use serde::Serialize;

use super::Guest;

#[derive(Debug, Serialize)]
pub struct Seat {
	pub guest: Guest,
	pub ready: bool,
	pub stack: usize,
	pub bet: usize,
	pub fold: bool,
}

impl Seat {
	/// Whether the guest has allined
	pub fn allin(&self) -> bool {
		self.stack == 0
	}
}

impl From<Guest> for Seat {
	fn from(guest: Guest) -> Self {
		Self {
			guest,
			ready: false,
			stack: 1000,
			bet: 0,
			fold: false,
		}
	}
}
