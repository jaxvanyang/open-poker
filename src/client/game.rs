use crate::sprintln;

use super::Client;

impl Client {
	/// Wait for self's turn
	pub async fn wait_turn(&self) -> anyhow::Result<()> {
		todo!()
	}

	pub async fn play(&mut self) -> anyhow::Result<()> {
		self.game_id = Some(self.wait_game().await?);
		self.wait_turn().await?;
		loop {
			match Client::read_command() {
				Err(err) => return Err(err),
				Ok(command) => match command.iter().map(|s| s.as_str()).collect::<Vec<_>>()[..] {
					[] => {
						continue;
					}
					["fold"] => self.fold().await?,
					["check"] => self.check().await?,
					["call"] => self.call().await?,
					["raise", chips] => self.raise(chips.parse::<usize>().unwrap()).await?,
					["allin"] => self.allin().await?,
					_ => sprintln!("unknown command!"),
				},
			}
		}
	}

	pub async fn fold(&mut self) -> anyhow::Result<()> {
		let user = self.user.as_ref().unwrap();
		let table_id = self.table_id.as_ref().unwrap();

		let request = serde_json::json!({
			"user_id": user.id,
			"table_id": table_id,
		});

		let response = self
			.awc
			.post(format!("{}/ready", self.server_addr))
			.send_json(&request)
			.await?;

		if response.status().is_success() {
			sprintln!("ready to play");
		} else {
			sprintln!("failed to be ready");
		}

		Ok(())
	}

	pub async fn check(&mut self) -> Result<()> {
		todo!()
	}

	pub async fn call(&mut self) -> Result<()> {
		todo!()
	}

	pub async fn raise(&mut self, chips: usize) -> Result<()> {
		todo!()
	}

	pub async fn allin(&mut self) -> Result<()> {
		todo!()
	}
}
