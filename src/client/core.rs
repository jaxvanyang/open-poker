use std::{fmt::Display, io::Write, process::exit, time::Duration};

use actix_web::rt::time::sleep;
use serde::Deserialize;

use crate::{Card, Game, Guest, Room, sprintln};

use super::{error::anyhow_error, game::RoomResponse};

#[derive(Debug, Deserialize)]
pub struct ErrorResponse {
	pub error: String,
}

impl Display for ErrorResponse {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		self.error.fmt(f)
	}
}

pub struct Client {
	pub awc: awc::Client,
	pub server_addr: String,
	pub guest: Option<Guest>,
	pub token: Option<String>,
	pub room: Option<Room>,
	pub game: Option<Game>,
	pub hand: Vec<Card>,
	pub common: Vec<Card>,
}

impl Default for Client {
	fn default() -> Self {
		Self {
			awc: awc::Client::default(),
			server_addr: "http://localhost:12345".to_string(),
			guest: None,
			token: None,
			room: None,
			game: None,
			hand: vec![],
			common: vec![],
		}
	}
}

impl Client {
	pub const TPS: f32 = 20.0;
	pub const TICK: f32 = 1.0 / Self::TPS;

	/// Read input into an array of strings
	pub fn read_command() -> anyhow::Result<Vec<String>> {
		let mut input = String::new();
		print!("> ");
		std::io::stdout().flush().unwrap();
		std::io::stdin().read_line(&mut input)?;

		if input.is_empty() {
			println!();
			exit(0);
		}
		let command = input
			.split_whitespace()
			.map(std::string::ToString::to_string)
			.collect();

		Ok(command)
	}

	/// Sync status with the server
	pub async fn sync(&mut self) -> anyhow::Result<()> {
		if self.room.is_none() {
			return Ok(());
		}
		let room_id = self.room.as_ref().unwrap().id;
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
		} else {
			let resp: ErrorResponse = room_resp.json().await?;
			sprintln!("failed to sync with the server: {resp}");
		}

		Ok(())
	}

	/// Wait input and execute command
	///
	/// # Return
	///
	/// Err if command failed
	pub async fn run(&mut self) -> anyhow::Result<()> {
		let command = Client::read_command()?;
		let command: Vec<_> = command.iter().map(std::string::String::as_str).collect();
		match command[..] {
			[] => (),
			["help"] => print_help(),
			["status"] => {
				self.sync().await?;
				self.print_status();
			}
			["login", name] => {
				self.login(name).await?;
			}
			["new"] => {
				self.new_room().await?;
			}
			["join", id] => {
				self.join(id).await?;
			}
			["ready"] => {
				self.ready().await?;

				sprintln!("waiting...");
				self.wait_game().await?;
				sprintln!("game started");
				println!("Your hand: {}", self.pretty_hand());

				loop {
					let result = self.play().await;
					if let Err(err) = result {
						sprintln!("error: {err}");
					} else {
						break;
					}
				}
				sprintln!("game is over");
				self.print_game_result().await?;
			}
			["exit"] => {
				exit(0);
			}
			_ => sprintln!("unknown command or wrong usage"),
		}

		Ok(())
	}

	/// Sleep for a tick
	pub async fn tick() {
		sleep(Duration::from_secs_f32(Self::TICK)).await;
	}

	fn print_status(&self) {
		if self.guest.is_none() {
			println!("not login");
			return;
		}
		let guest = self.guest.as_ref().unwrap();

		if self.room.is_none() {
			println!("logined as {}, not in a room", guest.name);
			return;
		}

		let room = self.room.as_ref().unwrap();
		println!("Room: {}", room.id);
		println!("seat: name (stack) (bankroll) status");
		println!("------------------------------------");
		for (i, seat) in room.seats.iter().enumerate() {
			if seat.is_none() {
				continue;
			}
			let seat = seat.as_ref().unwrap();
			let ready = if seat.ready { "ready" } else { "not ready" };
			let mark = if seat.guest.id == guest.id {
				"(you)"
			} else {
				""
			};
			println!(
				"{i}: {} ({}) ({}) {ready} {mark}",
				seat.guest.name, seat.stack, seat.guest.bankroll
			);
		}
	}
}

fn print_help() {
	println!(
		"Command list:
		help
		status
		login <name>
		new
		join <room_id>
		ready
		exit"
	);
}
