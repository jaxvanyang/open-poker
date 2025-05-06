use serde::Serialize;

use super::Guest;

#[derive(Debug, Clone, Serialize)]
pub struct Room {
	pub id: usize,
	pub seats: [Option<Guest>; Self::MAX_SEATS],
	pub sb: usize,
}

impl Room {
	pub const MAX_SEATS: usize = 10;

	pub fn new(id: usize) -> Self {
		Self {
			id,
			seats: [const { None }; Self::MAX_SEATS],
			sb: 0,
		}
	}

	pub fn with_guest(id: usize, guest: &Guest) -> Self {
		let mut table = Self::new(id);
		table.seats[0] = Some(guest.clone());
		table
	}

	/// Number of users
	pub fn count(&self) -> usize {
		self.seats.iter().filter(|i| i.is_some()).count()
	}

	/// Are all players ready
	// pub fn should_start(&self) -> bool {
	// 	let mut count = 0;
	// 	for user in &self.seats {
	// 		if let Some(user) = user.as_ref() {
	// 			if user.ready {
	// 				count += 1;
	// 			} else {
	// 				return false;
	// 			}
	// 		}
	// 	}

	// 	count >= 2
	// }

	pub fn insert(&mut self, user: Guest) {
		assert!(self.count() < Self::MAX_SEATS);
		let mut empty = 0;
		for i in 0..Self::MAX_SEATS {
			match &self.seats[i] {
				Some(u) => {
					if u.id == user.id {
						panic!();
					}
				}
				None => empty = i,
			}
		}
		self.seats[empty] = Some(user);
	}

	/// Return if the user is on the table
	pub fn has_user(&mut self, user_id: usize) -> bool {
		for user in &self.seats {
			if let Some(user) = user.as_ref() {
				if user.id == user_id {
					return true;
				}
			}
		}
		false
	}

	// Mark the user ready
	// pub fn ready(&mut self, user_id: &str) {
	// 	for user in &mut self.seats {
	// 		if let Some(user) = user.as_mut() {
	// 			if user.id == user_id {
	// 				user.ready = true;
	// 				return;
	// 			}
	// 		}
	// 	}
	// 	panic!()
	// }

	// pub fn new_game(&mut self) -> Game {
	// 	assert!(self.should_start());

	// 	let mut sb = 0;
	// 	for i in 0..Self::MAX_SEATS {
	// 		sb = (self.sb + i) % Self::MAX_SEATS;
	// 		if self.seats[sb].is_some() {
	// 			break;
	// 		}
	// 	}
	// 	self.sb = (sb + 1) % Self::MAX_SEATS;

	// 	let mut players = Vec::new();
	// 	for i in 0..Self::MAX_SEATS {
	// 		let seat = (sb + i) % Self::MAX_SEATS;
	// 		if let Some(user) = &self.seats[seat] {
	// 			players.push(Player {
	// 				seat,
	// 				name: user.name.clone(),
	// 				stack: user.stack,
	// 				total_bet: 0,
	// 				has_folded: false,
	// 				hand: Vec::with_capacity(2),
	// 			})
	// 		}
	// 	}

	// 	Game::new(players)
	// }
}
