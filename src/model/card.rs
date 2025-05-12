use std::fmt::Display;

use rand::seq::SliceRandom;
use rusqlite::{ToSql, types::FromSql};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
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
			4 => Some(Suit::Club),
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

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Numeral {
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Ten,
	A,
}

impl Display for Numeral {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		let numeral = match self {
			Self::A => 'A',
			Self::Two => '2',
			Self::Three => '3',
			Self::Four => '4',
			Self::Five => '5',
			Self::Six => '6',
			Self::Seven => '7',
			Self::Eight => '8',
			Self::Nine => '9',
			Self::Ten => 'T',
		};
		write!(f, "{numeral}")
	}
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Face {
	J,
	Q,
	K,
}

impl Display for Face {
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		write!(
			f,
			"{}",
			match self {
				Self::J => 'J',
				Self::Q => 'Q',
				Self::K => 'K',
			}
		)
	}
}

#[derive(Debug, Clone, Copy)]
pub enum Joker {
	Red,
	Black,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub enum Rank {
	Two,
	Three,
	Four,
	Five,
	Six,
	Seven,
	Eight,
	Nine,
	Ten,
	J,
	Q,
	K,
	A,
}

/// Iterate from lowest 2 to highest A
pub struct RankIter {
	i: u8,
}

impl Iterator for RankIter {
	type Item = Rank;
	fn next(&mut self) -> Option<Self::Item> {
		let out = match self.i {
			2 => Some(Rank::Two),
			3 => Some(Rank::Three),
			4 => Some(Rank::Four),
			5 => Some(Rank::Five),
			6 => Some(Rank::Six),
			7 => Some(Rank::Seven),
			8 => Some(Rank::Eight),
			9 => Some(Rank::Nine),
			10 => Some(Rank::Ten),
			11 => Some(Rank::J),
			12 => Some(Rank::Q),
			13 => Some(Rank::K),
			14 => Some(Rank::A),
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
		RankIter { i: 2 }
	}

	pub fn as_usize(&self) -> usize {
		match self {
			Rank::A => 1,
			Rank::Two => 2,
			Rank::Three => 3,
			Rank::Four => 4,
			Rank::Five => 5,
			Rank::Six => 6,
			Rank::Seven => 7,
			Rank::Eight => 8,
			Rank::Nine => 9,
			Rank::Ten => 10,
			Rank::J => 11,
			Rank::Q => 12,
			Rank::K => 13,
		}
	}

	/// Parse database representation
	pub fn parse(rank: char) -> Self {
		match rank {
			'A' => Self::A,
			'2' => Self::Two,
			'3' => Self::Three,
			'4' => Self::Four,
			'5' => Self::Five,
			'6' => Self::Six,
			'7' => Self::Seven,
			'8' => Self::Eight,
			'9' => Self::Nine,
			'T' => Self::Ten,
			'J' => Self::J,
			'Q' => Self::Q,
			'K' => Self::K,
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
				Self::A => 'A',
				Self::Two => '2',
				Self::Three => '3',
				Self::Four => '4',
				Self::Five => '5',
				Self::Six => '6',
				Self::Seven => '7',
				Self::Eight => '8',
				Self::Nine => '9',
				Self::Ten => 'T',
				Self::J => 'J',
				Self::Q => 'Q',
				Self::K => 'K',
			}
		)
	}
}

/// French-suited card
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct Card {
	pub rank: Rank,
	pub suit: Suit,
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
		for rank in Rank::iter() {
			for suit in Suit::iter() {
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

impl PartialOrd for Card {
	fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
		self.rank.partial_cmp(&other.rank)
	}
}

impl Ord for Card {
	fn cmp(&self, other: &Self) -> std::cmp::Ordering {
		self.rank.cmp(&other.rank)
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn test_unique() {
		let deck = Card::new_deck();
		for i in 0..deck.len() {
			for j in (i + 1)..deck.len() {
				assert_ne!(deck[i], deck[j]);
			}
		}
	}

	#[test]
	fn test_order() {
		let deck = Card::new_sorted();
		assert!(deck.is_sorted());
	}

	#[test]
	fn test_random() {
		let d1 = Card::new_deck();
		let d2 = Card::new_deck();
		assert_ne!(d1, d2);
	}
}
