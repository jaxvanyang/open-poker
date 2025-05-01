use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct User {
	pub id: String,
	pub name: String,
	pub token: String,
	pub stack: usize,
	pub ready: bool,
}

impl User {
	pub fn buy_in(&mut self, chips: usize) {
		self.stack = self.stack.checked_add(chips).unwrap();
	}

	pub fn new(name: &str) -> Option<Self> {
		match name {
			"client" | "server" => None,
			name => Some(User {
				id: name.split_whitespace().collect::<Vec<_>>().join("_"),
				name: name.to_string(),
				token: "token".to_string(),
				stack: 1000,
				ready: false,
			}),
		}
	}
}
