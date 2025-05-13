use std::fmt::Display;

use rusqlite::{ToSql, types::FromSql};
use serde::{Deserialize, Serialize};

use super::Room;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, PartialOrd, Ord)]
pub enum Round {
	PreFlop,
	Flop,
	Turn,
	River,
	Over,
}

impl Round {
	/// Parse database representation
	///
	/// # Panics
	///
	/// Will panic if `round` is not valid
	#[must_use]
	pub fn parse(round: &str) -> Self {
		match round {
			"preflop" => Self::PreFlop,
			"flop" => Self::Flop,
			"turn" => Self::Turn,
			"river" => Self::River,
			"finish" => Self::Over,
			_ => panic!("invalid round"),
		}
	}

	/// Return the next round
	///
	/// # Panics
	///
	/// Will panic if self is over
	#[must_use]
	pub fn next_round(&self) -> Self {
		match self {
			Round::PreFlop => Round::Flop,
			Round::Flop => Round::Turn,
			Round::Turn => Round::River,
			Round::River => Round::Over,
			Round::Over => panic!("no next round"),
		}
	}
}

impl Display for Round {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::PreFlop => "preflop",
				Self::Flop => "flop",
				Self::Turn => "turn",
				Self::River => "river",
				Self::Over => "finish",
			}
		)
	}
}

impl FromSql for Round {
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		Ok(Self::parse(value.as_str().unwrap()))
	}
}

impl ToSql for Round {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		Ok(self.to_string().into())
	}
}

#[derive(Debug, Serialize, Deserialize)]
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
	#[must_use]
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

	#[must_use]
	pub fn is_over(&self) -> bool {
		self.round == Round::Over
	}

	/// Correct player position
	pub fn correct(&mut self, room: &Room) {
		let mut p;
		for i in 0..Room::MAX_SEATS {
			p = (self.position + i) % Room::MAX_SEATS;
			if let Some(seat) = &room.seats[p] {
				if seat.fold || seat.allin() {
					continue;
				}
				self.position = p;
				return;
			}
		}
	}

	/// Pass control to the next player
	pub fn pass(&mut self, room: &Room) {
		self.position += 1;
		self.correct(room);
	}

	/// Update round if condition meet
	///
	/// # Return
	///
	/// Return ture if round changed
	pub fn update(&mut self, room: &Room) -> bool {
		// all fold except one or all remaining allin
		if room.player_count() == 1 || room.all_allin() {
			self.round = Round::Over;
			return true;
		}

		// deal logic
		if self.position == self.raise_position {
			self.round = self.round.next_round();
			return true;
		}

		false
	}
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GameResult {
	pub game_id: usize,
	pub guest_id: usize,
	pub diff: isize,
}

impl GameResult {
	#[must_use]
	pub fn new(game_id: usize, guest_id: usize, diff: isize) -> Self {
		Self {
			game_id,
			guest_id,
			diff,
		}
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
			Round::Over,
		];
		for (i, a) in rounds.iter().enumerate() {
			for b in &rounds[(i + 1)..] {
				assert!(a < b);
			}
		}
	}
}
