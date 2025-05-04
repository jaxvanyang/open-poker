use crate::{Result, sprintln};

use super::Client;

impl Client {
	pub async fn login(&mut self, name: &str) -> Result<()> {
		let request = serde_json::json!({
			"name": name,
		});

		let mut response = self
			.awc
			.post(format!("{}/user/login", self.server_addr))
			.send_json(&request)
			.await?;

		if response.status().is_success() {
			self.user = Some(response.json().await?);
			sprintln!("login success");
		} else {
			let reason = String::from_utf8(response.body().await?.into()).unwrap();
			sprintln!("login failed: {reason}");
		}

		Ok(())
	}

	pub async fn new_table(&mut self) -> Result<()> {
		let user = self.user.as_ref().unwrap();

		let mut response = self
			.awc
			.post(format!("{}/user/new/{}", self.server_addr, user.id))
			.send()
			.await?;

		if response.status().is_success() {
			let table_id: String = response.json().await?;
			self.table_id = Some(table_id.clone());
			sprintln!("created a new table: {}", table_id);
		} else {
			let reason = String::from_utf8(response.body().await?.into()).unwrap();
			sprintln!("failed to create a new table: {reason}");
		}

		Ok(())
	}

	pub async fn join(&mut self, table_id: &str) -> Result<()> {
		let user = self.user.as_ref().unwrap();

		let mut response = self
			.awc
			.post(format!(
				"{}/user/join/{}/{}",
				self.server_addr, user.id, table_id
			))
			.send()
			.await?;

		if response.status().is_success() {
			let table_id: String = response.json().await?;
			self.table_id = Some(table_id.clone());
			sprintln!("joined table {}", table_id);
		} else {
			let reason = String::from_utf8(response.body().await?.into()).unwrap();
			sprintln!("failed to join the table: {reason}");
		}

		Ok(())
	}
}
