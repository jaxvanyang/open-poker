use std::error::Error;

use crate::{Game, User, sprintln};

pub struct Client {
	pub awc: awc::Client,
	pub server_addr: String,
	pub user: Option<User>,
	pub table_id: Option<String>,
	pub game: Option<Game>,
}

impl Default for Client {
	fn default() -> Self {
		Self {
			awc: awc::Client::default(),
			server_addr: "http://localhost:12345".to_string(),
			user: None,
			table_id: None,
			game: None,
		}
	}
}

impl Client {
	pub async fn login(&mut self, name: &str) -> Result<(), Box<dyn Error>> {
		let request = serde_json::json!({
			"name": name,
		});

		let mut response = self
			.awc
			.post(format!("{}/login", self.server_addr))
			.send_json(&request)
			.await?;

		if response.status().is_success() {
			self.user = Some(response.json().await?);
			sprintln!("login success");
		} else {
			sprintln!("login failed");
		}

		Ok(())
	}

	pub async fn new_game(&mut self) -> Result<(), Box<dyn Error>> {
		let user = self.user.as_ref().unwrap();

		let request = serde_json::json!({
			"user_id": user.id,
		});

		let mut response = self
			.awc
			.post(format!("{}/new", self.server_addr))
			.send_json(&request)
			.await?;

		if response.status().is_success() {
			let table_id: String = response.json().await?;
			self.table_id = Some(table_id.clone());
			sprintln!("created a new table: {}", table_id);
		} else {
			sprintln!("failed to create a new table");
		}

		Ok(())
	}

	pub async fn join(&mut self, table_id: &str) -> Result<(), Box<dyn Error>> {
		let user = self.user.as_ref().unwrap();

		let request = serde_json::json!({
			"user_id": user.id,
			"table_id": table_id,
		});

		let mut response = self
			.awc
			.post(format!("{}/join", self.server_addr))
			.send_json(&request)
			.await?;

		if response.status().is_success() {
			let table_id: String = response.json().await?;
			self.table_id = Some(table_id.clone());
			sprintln!("joined table {}", table_id);
		} else {
			sprintln!("failed to join the table");
		}

		Ok(())
	}

	pub async fn ready(&mut self) -> Result<(), Box<dyn Error>> {
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

	pub async fn fold(&mut self) -> Result<(), Box<dyn Error>> {
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

	pub async fn check(&mut self) -> Result<(), Box<dyn Error>> {
		todo!()
	}

	pub async fn call(&mut self) -> Result<(), Box<dyn Error>> {
		todo!()
	}

	pub async fn raise(&mut self, chips: usize) -> Result<(), Box<dyn Error>> {
		todo!()
	}

	pub async fn allin(&mut self) -> Result<(), Box<dyn Error>> {
		todo!()
	}
}
