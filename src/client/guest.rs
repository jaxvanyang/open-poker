use serde::Deserialize;
use serde_json::json;

use crate::{Guest, client::ErrorResponse, sprintln};

use super::{Client, error::anyhow_error};

#[derive(Debug, Deserialize)]
struct LoginResponse {
	guest: Guest,
	token: String,
}

impl Client {
	pub async fn login(&mut self, name: &str) -> anyhow::Result<()> {
		let mut response = self
			.awc
			.post(format!("{}/guests", self.server_addr))
			.send_form(&json!({"name": name}))
			.await
			.map_err(anyhow_error)?;

		if response.status().is_success() {
			let resp: LoginResponse = response.json().await?;
			self.guest = Some(resp.guest);
			self.token = Some(resp.token);
			sprintln!("login success");
		} else {
			let resp: ErrorResponse = response.json().await?;
			sprintln!("login failed: {}", resp);
		}

		Ok(())
	}

	// pub async fn join(&mut self, table_id: &str) -> Result<()> {
	// 	let user = self.user.as_ref().unwrap();

	// 	let mut response = self
	// 		.awc
	// 		.post(format!(
	// 			"{}/user/join/{}/{}",
	// 			self.server_addr, user.id, table_id
	// 		))
	// 		.send()
	// 		.await?;

	// 	if response.status().is_success() {
	// 		let table_id: String = response.json().await?;
	// 		self.table_id = Some(table_id.clone());
	// 		sprintln!("joined table {}", table_id);
	// 	} else {
	// 		let reason = String::from_utf8(response.body().await?.into()).unwrap();
	// 		sprintln!("failed to join the table: {reason}");
	// 	}

	// 	Ok(())
	// }
}
