use rusqlite::types::FromSql;
use serde::Serialize;

use super::Room;
use crate::error::{Result, forbidden_error};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, PartialOrd, Ord)]
pub enum Round {
	PreFlop,
	Flop,
	Turn,
	River,
	Finish,
}

impl Round {
	/// Parse database representation
	pub fn parse(round: &str) -> Self {
		match round {
			"preflop" => Self::PreFlop,
			"flop" => Self::Flop,
			"turn" => Self::Turn,
			"river" => Self::River,
			"finish" => Self::Finish,
			_ => panic!("invalid round"),
		}
	}
}

impl FromSql for Round {
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		Ok(Self::parse(value.as_str().unwrap()))
	}
}

#[derive(Debug, Serialize)]
pub struct Game {
	pub id: usize,
	pub room_id: usize,
	pub round: Round,
	pub pot: usize,
	/// Current player's position
	pub position: usize,
	/// Position of the first raise player
	pub raise_position: usize,
}

impl Game {
	pub fn new(id: usize, room_id: usize, sb: usize) -> Self {
		Self {
			id,
			room_id,
			round: Round::PreFlop,
			pot: 0,
			position: sb,
			raise_position: sb,
		}
	}

	pub fn is_finished(&self) -> bool {
		self.round == Round::Finish
	}

	/// Correct player position
	pub fn correct(&mut self, room: &Room) -> Result<()> {
		let mut p;
		for i in 0..Room::MAX_SEATS {
			p = (self.position + i) % Room::MAX_SEATS;
			if let Some(seat) = &room.seats[p] {
				if seat.fold || seat.allin() {
					continue;
				}
				self.position = p;
				return Ok(());
			}
		}

		Err(forbidden_error("invalid operation"))
	}

	/// Pass control to the next player
	pub fn pass(&mut self, room: &Room) -> Result<()> {
		self.position += 1;
		self.correct(room)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_round_cmp() {
		let rounds = [
			Round::PreFlop,
			Round::Flop,
			Round::Turn,
			Round::River,
			Round::Finish,
		];
		for (i, a) in rounds.iter().enumerate() {
			for b in &rounds[(i + 1)..] {
				assert!(a < b);
			}
		}
	}
}
