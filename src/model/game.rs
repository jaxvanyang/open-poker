use serde::Serialize;

#[derive(Debug, PartialEq, Eq, Serialize)]
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
}
