use std::fmt::Display;

use rand::seq::SliceRandom;
use rusqlite::{ToSql, types::FromSql};
use serde::Serialize;

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Suit {
	Spade,
	Heart,
	Diamond,
	Club,
}

pub struct SuitIter {
	i: u8,
}

impl Iterator for SuitIter {
	type Item = Suit;
	fn next(&mut self) -> Option<Self::Item> {
		let out = match self.i {
			1 => Some(Suit::Spade),
			2 => Some(Suit::Heart),
			3 => Some(Suit::Diamond),
			4 => Some(Suit::Diamond),
			_ => None,
		};

		if out.is_some() {
			self.i += 1;
		}

		out
	}
}

impl Suit {
	pub fn iter() -> SuitIter {
		SuitIter { i: 1 }
	}

	/// Parse database representation
	pub fn parse(suit: char) -> Self {
		match suit {
			'S' => Self::Spade,
			'H' => Self::Heart,
			'C' => Self::Club,
			'D' => Self::Diamond,
			_ => panic!("invalid suit"),
		}
	}
}

impl Display for Suit {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let suit = match self {
			Suit::Spade => 'S',
			Suit::Heart => 'H',
			Suit::Diamond => 'D',
			Suit::Club => 'C',
		};
		write!(f, "{suit}")
	}
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Numeral {
	A = 1,
	Two = 2,
	Three = 3,
	Four = 4,
	Five = 5,
	Six = 6,
	Seven = 7,
	Eight = 8,
	Nine = 9,
	Ten = 10,
}

impl Display for Numeral {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let numeral = match self {
			Numeral::A => 'A',
			Numeral::Two => '2',
			Numeral::Three => '3',
			Numeral::Four => '4',
			Numeral::Five => '5',
			Numeral::Six => '6',
			Numeral::Seven => '7',
			Numeral::Eight => '8',
			Numeral::Nine => '9',
			Numeral::Ten => 'T',
		};
		write!(f, "{numeral}")
	}
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Face {
	J = 11,
	Q = 12,
	K = 13,
}

impl Display for Face {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Face::J => 'J',
				Face::Q => 'Q',
				Face::K => 'K',
			}
		)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Joker {
	Red,
	Black,
}

#[derive(Debug, Clone, Copy, Serialize)]
pub enum Rank {
	Numeral(Numeral),
	Face(Face),
}

pub struct RankIter {
	i: u8,
}

impl Iterator for RankIter {
	type Item = Rank;
	fn next(&mut self) -> Option<Self::Item> {
		let out = match self.i {
			1 => Some(Rank::Numeral(Numeral::A)),
			2 => Some(Rank::Numeral(Numeral::Two)),
			3 => Some(Rank::Numeral(Numeral::Three)),
			4 => Some(Rank::Numeral(Numeral::Four)),
			5 => Some(Rank::Numeral(Numeral::Five)),
			6 => Some(Rank::Numeral(Numeral::Six)),
			7 => Some(Rank::Numeral(Numeral::Seven)),
			8 => Some(Rank::Numeral(Numeral::Eight)),
			9 => Some(Rank::Numeral(Numeral::Nine)),
			10 => Some(Rank::Numeral(Numeral::Ten)),
			11 => Some(Rank::Face(Face::J)),
			12 => Some(Rank::Face(Face::Q)),
			13 => Some(Rank::Face(Face::K)),
			_ => None,
		};

		if out.is_some() {
			self.i += 1;
		}

		out
	}
}

impl Rank {
	pub fn iter() -> RankIter {
		RankIter { i: 1 }
	}

	/// Parse database representation
	pub fn parse(rank: char) -> Self {
		match rank {
			'A' => Self::Numeral(Numeral::A),
			'2' => Self::Numeral(Numeral::Two),
			'3' => Self::Numeral(Numeral::Three),
			'4' => Self::Numeral(Numeral::Four),
			'5' => Self::Numeral(Numeral::Five),
			'6' => Self::Numeral(Numeral::Six),
			'7' => Self::Numeral(Numeral::Seven),
			'8' => Self::Numeral(Numeral::Eight),
			'9' => Self::Numeral(Numeral::Nine),
			'T' => Self::Numeral(Numeral::Ten),
			'J' => Self::Face(Face::J),
			'Q' => Self::Face(Face::Q),
			'K' => Self::Face(Face::K),
			_ => panic!("invalid rank"),
		}
	}
}

impl Display for Rank {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Rank::Numeral(numeral) => numeral.to_string(),
				Rank::Face(face) => face.to_string(),
			}
		)
	}
}

/// French-suited card
#[derive(Debug, Clone, Copy, Serialize)]
pub struct Card {
	pub suit: Suit,
	pub rank: Rank,
}

/// Modern card
#[derive(Debug, Clone, Copy)]
pub enum ModernCard {
	Card(Card),
	Joker(Joker),
}

pub type Deck = Vec<Card>;

impl Card {
	/// Create a new sorted deck
	pub fn new_sorted() -> Deck {
		let mut deck = Deck::with_capacity(52);
		for suit in Suit::iter() {
			for rank in Rank::iter() {
				deck.push(Card { suit, rank });
			}
		}

		deck
	}

	/// Create a new shulled deck
	pub fn new_deck() -> Deck {
		let mut deck = Self::new_sorted();
		deck.shuffle(&mut rand::rng());

		deck
	}

	/// Parse database representation
	pub fn parse(card: &str) -> Self {
		assert!(card.len() == 2);
		Self {
			suit: Suit::parse(card.chars().nth(0).unwrap()),
			rank: Rank::parse(card.chars().nth(1).unwrap()),
		}
	}
}

impl Display for Card {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(f, "{}{}", self.suit, self.rank)
	}
}

impl ToSql for Card {
	fn to_sql(&self) -> rusqlite::Result<rusqlite::types::ToSqlOutput<'_>> {
		Ok(self.to_string().into())
	}
}

impl FromSql for Card {
	fn column_result(value: rusqlite::types::ValueRef<'_>) -> rusqlite::types::FromSqlResult<Self> {
		Ok(Self::parse(value.as_str()?))
	}
}
