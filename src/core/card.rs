use rand::seq::SliceRandom;

#[derive(Debug, Clone, Copy)]
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
}

#[derive(Debug, Clone, Copy)]
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

#[derive(Debug, Clone, Copy)]
pub enum Face {
	J = 11,
	Q = 12,
	K = 13,
}

#[derive(Debug, Clone, Copy)]
pub enum Joker {
	Red,
	Black,
}

#[derive(Debug, Clone, Copy)]
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
}

/// French-suited card
#[derive(Debug, Clone, Copy)]
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
}
