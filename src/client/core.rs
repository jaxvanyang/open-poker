use std::{io::Write, process::exit, time::Duration};

use actix_web::rt::time::sleep;

use crate::User;

pub struct Client {
	pub awc: awc::Client,
	pub server_addr: String,
	pub user: Option<User>,
	pub table_id: Option<String>,
	pub game_id: Option<String>,
}

impl Default for Client {
	fn default() -> Self {
		Self {
			awc: awc::Client::default(),
			server_addr: "http://localhost:12345".to_string(),
			user: None,
			table_id: None,
			game_id: None,
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
		let command = input.split_whitespace().map(|s| s.to_string()).collect();

		Ok(command)
	}

	/// Sleep for a tick
	pub async fn tick() {
		sleep(Duration::from_secs_f32(Self::TICK)).await;
	}
}
