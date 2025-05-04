use crate::{Result, sprintln};

use super::Client;

impl Client {
	pub async fn ready(&mut self) -> Result<()> {
		let user = self.user.as_ref().unwrap();
		let table_id = self.table_id.as_ref().unwrap();

		let mut response = self
			.awc
			.post(format!(
				"{}/user/ready/{}/{}",
				self.server_addr, user.id, table_id
			))
			.send()
			.await?;

		if response.status().is_success() {
			sprintln!("ready to play");
		} else {
			let reason = String::from_utf8(response.body().await?.into()).unwrap();
			sprintln!("failed to be ready: {reason}");
		}

		Ok(())
	}

	/// Wait for game to start
	pub async fn wait_game(&self) -> Result<String> {
		let user = self.user.as_ref().unwrap();
		let table_id = self.table_id.as_ref().unwrap();
		loop {
			let mut response = self
				.awc
				.post(format!(
					"{}/table/game/{}/{}",
					self.server_addr, user.id, table_id
				))
				.send()
				.await?;

			if response.status().is_success() {
				let game_id = String::from_utf8(response.body().await?.into()).unwrap();
				sprintln!("game started: {game_id}");
				return Ok(game_id);
			} else {
				Self::tick().await;
			}
		}
	}
}
