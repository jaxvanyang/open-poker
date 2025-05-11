use std::process::exit;

use serde::Deserialize;

use crate::{Card, Game, Room, client::ErrorResponse, sprintln};

use super::{Client, error::anyhow_error};

#[derive(Debug, Deserialize)]
struct RoomResponse {
	room: Room,
	game: Option<Game>,
}

#[derive(Debug, Deserialize)]
struct HandResponse {
	hand: Vec<Card>,
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
				let mut should_break = false;
				let resp: RoomResponse = response.json().await?;
				self.game = resp.game;
				if let Some(game) = &self.game {
					if resp.room.seats[game.position]
						.as_ref()
						.is_some_and(|s| s.guest.id == guest_id)
					{
						should_break = true;
					}
				} else {
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
					if hand_resp.status().is_success() {
						let resp: HandResponse = hand_resp.json().await?;
						self.hand = resp.hand;
						break;
					}
				}
			} else {
				let resp: ErrorResponse = room_resp.json().await?;
				sprintln!("failed to retrive game info: {}", resp);
			}

			Self::tick().await;
		}

		Ok(())
	}

	pub async fn play(&mut self) -> anyhow::Result<()> {
		sprintln!("waiting...");
		self.wait_game().await?;
		sprintln!("game started");
		println!("You have {}, {}", self.hand[0], self.hand[1]);

		sprintln!("waiting...");
		self.wait_turn().await?;
		if self.game.is_none() {
			return Ok(());
		}
		sprintln!("it's your turn now");

		loop {
			let command = Client::read_command()?;
			let command: Vec<_> = command.iter().map(|s| s.as_str()).collect();
			match command[..] {
				[] => {
					continue;
				}
				["help"] => print_help(),
				["status"] => self.print_game_status(),
				["fold"] => {
					let result = self.fold().await;
					if let Err(err) = result {
						sprintln!("failed to fold: {err}");
					}
				}
				["check"] => todo!(),
				["call"] => todo!(),
				["raise", chips] => todo!(),
				["allin"] => todo!(),
				["exit"] => exit(0),
				_ => sprintln!("unknown command or wrong usage"),
			}

			if self.game.as_ref().is_some_and(|g| g.is_over()) {
				break;
			}
		}

		Ok(())
	}

	pub async fn fold(&mut self) -> anyhow::Result<()> {
		let game_id = self.game.as_ref().unwrap().id;
		let token = self.token.as_ref().unwrap();

		let mut response = self
			.awc
			.post(format!("{}/games/{}/fold", self.server_addr, game_id))
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
			sprintln!("failed fold in the room: {}", resp);
		}

		Ok(())
	}

	// pub async fn check(&mut self) -> Result<()> {
	// 	todo!()
	// }

	// pub async fn call(&mut self) -> Result<()> {
	// 	todo!()
	// }

	// pub async fn raise(&mut self, chips: usize) -> Result<()> {
	// 	todo!()
	// }

	// pub async fn allin(&mut self) -> Result<()> {
	// 	todo!()
	// }

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
				"<"
			} else if game.position == i {
				"..."
			} else {
				""
			};
			println!("{i}: {} {status} {mark}", seat.guest.name);
		}

		println!("round: {}, pot: {}", game.round, game.pot);
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
	)
}
