use serde::Serialize;

use super::Guest;

/// Guest and ready status
type RoomGuest = (Guest, bool);

impl From<Guest> for RoomGuest {
	fn from(guest: Guest) -> Self {
		(guest, false)
	}
}

#[derive(Debug, Clone, Serialize)]
pub struct Room {
	pub id: usize,
	pub seats: [Option<RoomGuest>; Self::MAX_SEATS],
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
		table.seats[0] = Some(guest.clone().into());
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

	/// Let the guest join the room
	///
	/// # Return
	///
	/// Seat position of the new guest, None if room is full or the user already in
	pub fn insert(&mut self, guest: Guest) -> Option<usize> {
		let mut empty = Self::MAX_SEATS;
		for i in (0..Self::MAX_SEATS).rev() {
			match &self.seats[i] {
				Some((g, _)) => {
					if g.id == guest.id {
						return None;
					}
				}
				None => empty = i,
			}
		}

		if empty == Self::MAX_SEATS {
			None
		} else {
			self.seats[empty] = Some(guest.into());
			Some(empty)
		}
	}

	/// Return if the guest is on the table
	pub fn has_guest(&mut self, guest_id: usize) -> bool {
		for guest in &self.seats {
			if let Some((guest, _)) = guest.as_ref() {
				if guest.id == guest_id {
					return true;
				}
			}
		}
		false
	}

	/// Whether the guest is ready
	///
	/// # Return
	///
	/// None if not found
	pub fn is_ready(&mut self, guest_id: usize) -> Option<bool> {
		for guest in &self.seats {
			if let Some((guest, ready)) = guest.as_ref() {
				if guest.id == guest_id {
					return Some(*ready);
				}
			}
		}
		None
	}

	/// Set the guest to ready status
	///
	/// # Return
	///
	/// Seat position of the guest, None if not found
	pub fn ready(&mut self, guest_id: usize) -> Option<usize> {
		for (i, guest) in &mut self.seats.iter_mut().enumerate() {
			if let Some((guest, ready)) = guest.as_mut() {
				if guest.id == guest_id {
					*ready = true;
					return Some(i);
				}
			}
		}
		None
	}

	/// Set the guest to unready status
	///
	/// # Return
	///
	/// Seat position of the guest, None if not found
	pub fn unready(&mut self, guest_id: usize) -> Option<usize> {
		for (i, guest) in &mut self.seats.iter_mut().enumerate() {
			if let Some((guest, ready)) = guest.as_mut() {
				if guest.id == guest_id {
					*ready = false;
					return Some(i);
				}
			}
		}
		None
	}

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
