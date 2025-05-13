use serde::{Deserialize, Serialize};

use super::{Game, Guest, Seat};

#[derive(Debug, Serialize, Deserialize)]
pub struct Room {
	pub id: usize,
	pub seats: [Option<Seat>; Self::MAX_SEATS],
	/// The position of small blind
	pub sb: usize,
}

impl Room {
	pub const MAX_SEATS: usize = 10;

	#[must_use]
	pub fn new(id: usize) -> Self {
		Self {
			id,
			seats: [const { None }; Self::MAX_SEATS],
			sb: 0,
		}
	}

	#[must_use]
	pub fn with_guest(id: usize, guest: &Guest) -> Self {
		let mut table = Self::new(id);
		table.seats[0] = Some(guest.clone().into());
		table
	}

	#[must_use]
	pub fn get_guest(&self, position: usize) -> Option<&Guest> {
		self.seats[position].as_ref().map(|s| &s.guest)
	}

	/// Correct the sb position
	///
	/// # Panics
	///
	/// Will panic if there is no player in this room
	pub fn correct(&mut self) {
		let mut sb;
		for i in 0..Self::MAX_SEATS {
			sb = (self.sb + i) % Self::MAX_SEATS;
			if self.seats[sb].is_some() {
				self.sb = sb;
				return;
			}
		}

		panic!("no player in the room");
	}

	/// Pass sb to the next guest
	pub fn pass_sb(&mut self) {
		self.sb += 1;
		self.correct();
	}

	/// Number of users
	#[must_use]
	pub fn count(&self) -> usize {
		self.seats.iter().filter(|i| i.is_some()).count()
	}

	/// Number of unfold players
	#[must_use]
	pub fn player_count(&self) -> usize {
		self.seats
			.iter()
			.filter(|s| s.as_ref().is_some_and(|s| !s.fold))
			.count()
	}

	/// Whether all remaining players allin
	#[must_use]
	pub fn all_allin(&self) -> bool {
		self.seats
			.iter()
			.filter(|s| s.as_ref().is_some_and(|s| !s.fold && !s.allin()))
			.count() == 0
	}

	/// Are all players ready
	#[must_use]
	pub fn all_ready(&self) -> bool {
		for seat in &self.seats {
			if let Some(seat) = &seat {
				if !seat.ready {
					return false;
				}
			}
		}

		true
	}

	/// All guests are ready and count >= 2
	#[must_use]
	pub fn should_start(&self) -> bool {
		self.all_ready() && self.count() >= 2
	}

	/// Let the guest join the room
	///
	/// # Return
	///
	/// Seat position of the new guest, None if room is full or the user already in
	pub fn insert(&mut self, guest: Guest) -> Option<usize> {
		let mut empty = Self::MAX_SEATS;
		for i in (0..Self::MAX_SEATS).rev() {
			match &self.seats[i] {
				Some(seat) => {
					if seat.guest.id == guest.id {
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
		for seat in &self.seats {
			if let Some(seat) = &seat {
				if seat.guest.id == guest_id {
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
	#[must_use]
	pub fn is_ready(&self, guest_id: usize) -> Option<bool> {
		for seat in &self.seats {
			if let Some(seat) = &seat {
				if seat.guest.id == guest_id {
					return Some(seat.ready);
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
	///
	/// # Panics
	///
	/// Will panic if the guest's stack less than 10
	pub fn ready(&mut self, guest_id: usize) -> Option<usize> {
		for (i, seat) in &mut self.seats.iter_mut().enumerate() {
			if let Some(seat) = seat.as_mut() {
				if seat.guest.id == guest_id {
					assert!(seat.stack >= 10);
					seat.ready = true;
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
		for (i, seat) in &mut self.seats.iter_mut().enumerate() {
			if let Some(seat) = seat.as_mut() {
				if seat.guest.id == guest_id {
					seat.ready = false;
					return Some(i);
				}
			}
		}
		None
	}

	pub fn new_game(&mut self, game_id: usize) -> Game {
		let mut sb = 0;
		for i in 0..Self::MAX_SEATS {
			sb = (self.sb + i) % Self::MAX_SEATS;
			if self.seats[sb].is_some() {
				break;
			}
		}
		self.sb = (sb + 1) % Self::MAX_SEATS;

		Game::new(game_id, self.id, self.sb)
	}

	#[must_use]
	pub fn max_bet(&self) -> usize {
		let mut max_bet = 0;
		for i in 0..Self::MAX_SEATS {
			if let Some(seat) = &self.seats[i] {
				if seat.fold {
					continue;
				}
				max_bet = max_bet.max(seat.bet);
			}
		}

		max_bet
	}
}
