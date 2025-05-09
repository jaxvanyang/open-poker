use serde::Serialize;

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

#[derive(Debug, Serialize)]
pub struct Game {
	pub id: usize,
	pub room_id: usize,
	pub round: Round,
	pub pot: usize,
	pub position: usize,
}

impl Game {
	pub fn new(id: usize, room_id: usize, sb: usize) -> Self {
		Self {
			id,
			room_id,
			round: Round::PreFlop,
			pot: 0,
			position: sb,
		}
	}

	pub fn is_finished(&self) -> bool {
		self.round == Round::Finish
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
