use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct Guest {
	pub id: usize,
	pub name: String,
	pub bankroll: isize,
}

impl Guest {
	pub fn new(id: usize, name: &str) -> Self {
		Self {
			id,
			name: name.to_string(),
			bankroll: 0,
		}
	}
}
