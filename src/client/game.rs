use std::process::exit;

use serde::Deserialize;
use serde_json::json;

use crate::{Card, Game, GameResult, Room, client::ErrorResponse, sprintln};

use super::{Client, error::anyhow_error};

#[derive(Debug, Deserialize)]
pub struct RoomResponse {
	pub room: Room,
	pub game: Option<Game>,
}

#[derive(Debug, Deserialize)]
struct HandResponse {
	hand: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct CommonResponse {
	cards: Vec<Card>,
}

#[derive(Debug, Deserialize)]
struct ResultsResponse {
	results: Vec<GameResult>,
}

impl Client {
	/// Wait for self's turn or game over
	pub async fn wait_turn(&mut self) -> anyhow::Result<()> {
		let guest_id = self.guest.as_ref().unwrap().id;
		let room_id = self.room.as_ref().unwrap().id;

		loop {
			let mut response = self
				.awc
				.get(format!("{}/rooms/{}", self.server_addr, room_id))
				.send()
				.await
				.map_err(anyhow_error)?;

			if response.status().is_success() {
				let resp: RoomResponse = response.json().await?;
				assert!(resp.game.is_some());
				self.game = resp.game;
				let game = self.game.as_ref().unwrap();
				let mut should_break = game.is_over();
				// it's my turn now
				if resp.room.seats[game.position]
					.as_ref()
					.is_some_and(|s| s.guest.id == guest_id)
				{
					should_break = true;
				}

				self.room = Some(resp.room);
				if should_break {
					break;
				}
			} else {
				let resp: ErrorResponse = response.json().await?;
				sprintln!("failed to retrive game info: {}", resp);
			}

			Self::tick().await;
		}

		Ok(())
	}

	/// Wait for game to begin
	///
	/// # Note
	///
	/// After calling this, self.game should always be some
	pub async fn wait_game(&mut self) -> anyhow::Result<()> {
		let room_id = self.room.as_ref().unwrap().id;
		loop {
			let mut room_resp = self
				.awc
				.get(format!("{}/rooms/{}", self.server_addr, room_id))
				.send()
				.await
				.map_err(anyhow_error)?;

			if room_resp.status().is_success() {
				let resp: RoomResponse = room_resp.json().await?;
				self.room = Some(resp.room);
				self.game = resp.game;
				if let Some(game) = &self.game {
					let guest = self.guest.as_ref().unwrap();
					let mut hand_resp = self
						.awc
						.get(format!(
							"{}/games/{}/hands/{}",
							self.server_addr, game.id, guest.id
						))
						.bearer_auth(self.token.as_ref().unwrap())
						.send()
						.await
						.map_err(anyhow_error)?;
					if !game.is_over() && hand_resp.status().is_success() {
						let resp: HandResponse = hand_resp.json().await?;
						self.hand = resp.hand;
						break;
					}
				}
			} else {
				let resp: ErrorResponse = room_resp.json().await?;
				sprintln!("failed to retrive game info: {resp}");
			}

			Self::tick().await;
		}

		Ok(())
	}

	pub async fn play(&mut self) -> anyhow::Result<()> {
		loop {
			sprintln!("waiting...");
			self.wait_turn().await?;
			// game is over
			if self.game.as_ref().unwrap().is_over() {
				return Ok(());
			}
			sprintln!("it's your turn now");
			self.sync_game().await?;
			self.print_game_status();

			let command = Client::read_command()?;
			let command: Vec<_> = command.iter().map(std::string::String::as_str).collect();
			match command[..] {
				[] => {
					continue;
				}
				["help"] => print_help(),
				["status"] => {
					self.sync_game().await?;
					self.print_game_status();
				}
				["fold"] => {
					let result = self.fold().await;
					if let Err(err) = result {
						sprintln!("failed to fold: {err}");
					}
				}
				["check"] => {
					let result = self.check().await;
					if let Err(err) = result {
						sprintln!("failed to check: {err}");
					}
				}
				["call"] => {
					let result = self.call().await;
					if let Err(err) = result {
						sprintln!("failed to call: {err}");
					}
				}
				["raise", chips] => {
					let result = self.raise(chips.parse()?).await;
					if let Err(err) = result {
						sprintln!("failed to raise: {err}");
					}
				}
				["allin"] => {
					let result = self.allin().await;
					if let Err(err) = result {
						sprintln!("failed to allin: {err}");
					}
				}
				["exit"] => exit(0),
				_ => sprintln!("unknown command or wrong usage"),
			}

			if self.game.as_ref().unwrap().is_over() {
				break;
			}
		}

		Ok(())
	}

	pub async fn print_game_result(&mut self) -> anyhow::Result<()> {
		let game = self.game.as_ref().unwrap();
		let results;
		loop {
			let mut resp = self
				.awc
				.get(format!("{}/games/{}/results", self.server_addr, game.id))
				.send()
				.await
				.map_err(anyhow_error)?;

			if resp.status().is_success() {
				let resp: ResultsResponse = resp.json().await?;
				results = resp.results;
				break;
			}
			println!("failed to get game result: ");
		}

		println!(
			"Game: {}, pot: {}, common: {}",
			game.id,
			game.pot,
			self.pretty_common()
		);
		let room = self.room.as_ref().unwrap();
		for (i, seat) in room
			.seats
			.iter()
			.enumerate()
			.filter_map(|(i, s)| s.as_ref().map(|s| (i, s)))
		{
			for result in &results {
				if seat.guest.id == result.guest_id {
					let hand = if seat.fold {
						"fold".to_string()
					} else {
						// TODO: find cards in showdown
						"not fold".to_string()
					};
					println!("{i}: {} ({hand}) {:+}", seat.guest.name, result.diff);
					break;
				}
			}
		}

		Ok(())
	}

	pub async fn fold(&mut self) -> anyhow::Result<()> {
		let game_id = self.game.as_ref().unwrap().id;
		let token = self.token.as_ref().unwrap();

		let mut response = self
			.awc
			.post(format!("{}/games/{game_id}/fold", self.server_addr))
			.bearer_auth(token)
			.send()
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: RoomResponse = response.json().await?;
			sprintln!("folded in the room: {}", resp.room.id);
			self.room = Some(resp.room);
			self.game = resp.game;
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("failed to fold in the room: {}", resp);
		}

		Ok(())
	}

	pub async fn check(&mut self) -> anyhow::Result<()> {
		let game_id = self.game.as_ref().unwrap().id;
		let token = self.token.as_ref().unwrap();

		let mut response = self
			.awc
			.post(format!("{}/games/{game_id}/bets", self.server_addr))
			.bearer_auth(token)
			.send_form(&json!({"chips": 0}))
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: RoomResponse = response.json().await?;
			sprintln!("checked in the room: {}", resp.room.id);
			self.room = Some(resp.room);
			self.game = resp.game;
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("failed to check in the room: {}", resp);
		}

		Ok(())
	}

	pub async fn call(&mut self) -> anyhow::Result<()> {
		let game_id = self.game.as_ref().unwrap().id;
		let token = self.token.as_ref().unwrap();
		let guest_id = self.guest.as_ref().unwrap().id;
		let room = self.room.as_ref().unwrap();
		let mut chips = 0;
		for seat in room.seats.iter().filter_map(|s| s.as_ref()) {
			if seat.guest.id == guest_id {
				chips = room.max_bet() - seat.bet;
				break;
			}
		}

		let mut response = self
			.awc
			.post(format!("{}/games/{game_id}/bets", self.server_addr))
			.bearer_auth(token)
			.send_form(&json!({"chips": chips}))
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: RoomResponse = response.json().await?;
			sprintln!("called in the room: {}", resp.room.id);
			self.room = Some(resp.room);
			self.game = resp.game;
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("failed to call in the room: {}", resp);
		}

		Ok(())
	}

	pub async fn raise(&mut self, mut chips: usize) -> anyhow::Result<()> {
		let game_id = self.game.as_ref().unwrap().id;
		let token = self.token.as_ref().unwrap();
		let guest_id = self.guest.as_ref().unwrap().id;
		let room = self.room.as_ref().unwrap();
		for seat in room.seats.iter().filter_map(|s| s.as_ref()) {
			if seat.guest.id == guest_id {
				chips += room.max_bet() - seat.bet;
				break;
			}
		}

		let mut response = self
			.awc
			.post(format!("{}/games/{game_id}/bets", self.server_addr))
			.bearer_auth(token)
			.send_form(&json!({"chips": chips}))
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: RoomResponse = response.json().await?;
			sprintln!("raised in the room: {}", resp.room.id);
			self.room = Some(resp.room);
			self.game = resp.game;
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("failed to raise in the room: {}", resp);
		}

		Ok(())
	}

	pub async fn allin(&mut self) -> anyhow::Result<()> {
		let game_id = self.game.as_ref().unwrap().id;
		let token = self.token.as_ref().unwrap();
		let guest_id = self.guest.as_ref().unwrap().id;
		let room = self.room.as_ref().unwrap();
		let mut chips = 0;
		for seat in room.seats.iter().filter_map(|s| s.as_ref()) {
			if seat.guest.id == guest_id {
				chips = seat.stack;
				break;
			}
		}

		let mut response = self
			.awc
			.post(format!("{}/games/{game_id}/bets", self.server_addr))
			.bearer_auth(token)
			.send_form(&json!({"chips": chips}))
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: RoomResponse = response.json().await?;
			sprintln!("allined in the room: {}", resp.room.id);
			self.room = Some(resp.room);
			self.game = resp.game;
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("failed to allin in the room: {}", resp);
		}

		Ok(())
	}

	/// Convert cards to string for printing
	#[must_use]
	pub fn pretty_cards(cards: &[Card]) -> String {
		let mut s = String::new();
		if cards.is_empty() {
			return s;
		}
		s.push_str(&cards[0].to_string());
		for card in cards.iter().skip(1) {
			s.push_str(&format!(", {card}"));
		}

		s
	}

	#[must_use]
	pub fn pretty_hand(&self) -> String {
		Self::pretty_cards(&self.hand)
	}

	#[must_use]
	pub fn pretty_common(&self) -> String {
		Self::pretty_cards(&self.common)
	}

	/// Sync game status with the server
	pub async fn sync_game(&mut self) -> anyhow::Result<()> {
		self.sync().await?;

		let game_id = self.game.as_ref().unwrap().id;
		let mut resp = self
			.awc
			.get(format!("{}/games/{game_id}/common", self.server_addr))
			.send()
			.await
			.map_err(anyhow_error)?;

		if resp.status().is_success() {
			let resp: CommonResponse = resp.json().await?;
			self.common = resp.cards;
		} else {
			let resp: ErrorResponse = resp.json().await?;
			sprintln!("failed to sync with the server: {resp}");
		}

		Ok(())
	}

	fn print_game_status(&self) {
		let guest = self.guest.as_ref().unwrap();
		let room = self.room.as_ref().unwrap();
		let game = self.game.as_ref().unwrap();

		for (i, seat) in room.seats.iter().enumerate() {
			if seat.is_none() {
				continue;
			}
			let seat = seat.as_ref().unwrap();
			let status = if seat.fold {
				"fold".to_string()
			} else if seat.stack == 0 {
				format!("allin {}", seat.bet)
			} else {
				format!("bet {}", seat.bet)
			};
			let mark = if seat.guest.id == guest.id {
				format!("({}) (you)", self.pretty_hand())
			} else if game.position == i {
				"...".to_string()
			} else {
				String::new()
			};
			println!(
				"{i}: {} {status} ({}) ({}) {mark}",
				seat.guest.name, seat.stack, seat.guest.bankroll
			);
		}

		println!("----------------------------------------");
		println!(
			"round: {}, pot: {}, common: ({})",
			game.round,
			game.pot,
			self.pretty_common()
		);
	}
}

fn print_help() {
	println!(
		"Command list:
		help
		status
		fold
		check
		call
		raise
		allin
		exit"
	);
}
