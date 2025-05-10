use anyhow::anyhow;
use serde::Deserialize;

use crate::{Room, client::ErrorResponse, sprintln};

use super::{Client, error::anyhow_error};

#[derive(Debug, Deserialize)]
struct RoomResponse {
	room: Room,
}

impl Client {
	pub async fn new_room(&mut self) -> anyhow::Result<()> {
		if self.token.is_none() {
			return Err(anyhow!("you should login first"));
		}
		let token = self.token.as_ref().unwrap();

		let mut response = self
			.awc
			.post(format!("{}/rooms", self.server_addr))
			.bearer_auth(token)
			.send()
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: RoomResponse = response.json().await?;
			sprintln!("created a new room: {}", resp.room.id);
			self.room = Some(resp.room);
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("failed to create a new room: {}", resp);
		}

		Ok(())
	}

	pub async fn join(&mut self, id: &str) -> anyhow::Result<()> {
		if self.token.is_none() {
			return Err(anyhow!("you should login first"));
		}
		let token = self.token.as_ref().unwrap();

		let mut response = self
			.awc
			.patch(format!("{}/rooms/{id}", self.server_addr))
			.bearer_auth(token)
			.send()
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: RoomResponse = response.json().await?;
			sprintln!("joined the room: {}", resp.room.id);
			self.room = Some(resp.room);
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("failed to join the room: {}", resp);
		}

		Ok(())
	}

	pub async fn ready(&mut self) -> anyhow::Result<()> {
		if self.token.is_none() {
			return Err(anyhow!("you should login first"));
		}
		let token = self.token.as_ref().unwrap();
		let room = self.room.as_ref().unwrap();

		let mut response = self
			.awc
			.put(format!("{}/rooms/{}/ready", self.server_addr, room.id))
			.bearer_auth(token)
			.send()
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: RoomResponse = response.json().await?;
			sprintln!("ready in the room: {}", resp.room.id);
			self.room = Some(resp.room);
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("failed be ready in the room: {}", resp);
		}

		Ok(())
	}

	// /// Wait for game to start
	// pub async fn wait_game(&self) -> Result<String> {
	// 	let user = self.user.as_ref().unwrap();
	// 	let table_id = self.table_id.as_ref().unwrap();
	// 	loop {
	// 		let mut response = self
	// 			.awc
	// 			.post(format!(
	// 				"{}/table/game/{}/{}",
	// 				self.server_addr, user.id, table_id
	// 			))
	// 			.send()
	// 			.await?;

	// 		if response.status().is_success() {
	// 			let game_id = String::from_utf8(response.body().await?.into()).unwrap();
	// 			sprintln!("game started: {game_id}");
	// 			return Ok(game_id);
	// 		} else {
	// 			Self::tick().await;
	// 		}
	// 	}
	// }
}
