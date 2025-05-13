use super::{Card, Rank};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Kind {
	HighCard,
	Pair,
	TwoPairs,
	ThreeOfAKind,
	Straight,
	Flush,
	FullHouse,
	FourOfAKind,
	StraightFlush,
	RoyalFlush,
}

#[derive(Debug, Eq, PartialOrd, Ord)]
pub struct Hand {
	pub kind: Kind,
	/// Sorted five cards, cards form the kind first, then high card first
	pub cards: Vec<Card>,
}

impl PartialEq for Hand {
	fn eq(&self, other: &Self) -> bool {
		if self.kind != other.kind {
			return false;
		}
		for i in 0..5 {
			if self.cards[i].rank != other.cards[i].rank {
				return false;
			}
		}

		true
	}
}

impl Hand {
	/// # Panics
	///
	/// Will panic if the length of `common` or `hand` is not right
	#[must_use]
	pub fn calc_best_hand(common: &[Card], hand: &[Card]) -> Self {
		assert_eq!(common.len(), 5);
		assert_eq!(hand.len(), 2);
		let mut best_hand = Self::new(common);
		let mut cards: Vec<_> = common.into();
		cards.extend(hand);

		for i in 0..cards.len() {
			for j in (i + 1)..cards.len() {
				let cards: Vec<_> = cards
					.iter()
					.enumerate()
					.filter_map(|(k, c)| if k != i && k != j { Some(*c) } else { None })
					.collect();
				let hand = Self::new(&cards);
				best_hand = best_hand.max(hand);
			}
		}

		best_hand
	}

	/// Create a new five cards hand
	///
	/// # Panics
	///
	/// Will panic if the length of `cards` is not 5
	#[must_use]
	pub fn new(cards: &[Card]) -> Self {
		assert!(cards.len() == 5);
		let mut cards: Vec<_> = cards.into();
		cards.sort();
		cards.reverse();
		let kind = Self::calc_kind(&cards);

		Self { kind, cards }.normalized()
	}

	/// Make cards easier to compare from first to last
	fn normalized(mut self) -> Self {
		if Self::is_lowest_straight(&self.cards) || Self::is_last_four_of_a_kind(&self.cards) {
			self.cards.swap(0, 4);
			self.cards[0..4].sort();
			self.cards[0..4].reverse();
		} else if self.kind == Kind::FullHouse {
			if self.cards[2].rank == self.cards[3].rank && self.cards[2].rank == self.cards[4].rank
			{
				self.cards.swap(0, 3);
				self.cards.swap(1, 4);
				self.cards[0..3].sort();
				self.cards[0..3].reverse();
			}
		} else if self.kind == Kind::ThreeOfAKind {
			// a a a b c
			// a b b b c
			// a b c c c
			for i in 0..2 {
				if self.cards[i].rank != self.cards[2].rank {
					self.cards.swap(i, 4 - i);
					break;
				}
			}
			self.cards[0..3].sort();
			self.cards[0..3].reverse();
			self.cards[3..5].sort();
			self.cards[3..5].reverse();
		} else if self.kind == Kind::TwoPairs {
			// abbcc
			// bbacc
			// bbcca
			if self.cards[0].rank != self.cards[1].rank {
				self.cards.swap(0, 4);
			} else if self.cards[2].rank != self.cards[1].rank {
				self.cards.swap(2, 4);
			}
			self.cards[0..4].sort();
			self.cards[0..4].reverse();
		} else if self.kind == Kind::Pair {
			// find pair
			for i in 0..4 {
				if self.cards[i].rank == self.cards[i + 1].rank {
					self.cards.swap(0, i);
					self.cards.swap(1, i + 1);
					break;
				}
			}
		}

		self
	}

	fn calc_kind(cards: &[Card]) -> Kind {
		let is_flush = Self::is_flush(cards);
		let is_straight = Self::is_straight(cards);
		let is_straight_flush = is_flush && is_straight;
		let is_royal_flush = is_straight_flush && cards[4].rank == Rank::Ten;

		if is_royal_flush {
			return Kind::RoyalFlush;
		} else if is_straight_flush {
			return Kind::StraightFlush;
		}

		let is_four_of_a_kind = Self::is_four_of_a_kind(cards);
		if is_four_of_a_kind {
			return Kind::FourOfAKind;
		}

		let is_three_of_a_kind = Self::is_three_of_a_kind(cards);
		let is_two_pairs = Self::is_two_pairs(cards);
		let is_full_house = is_three_of_a_kind && is_two_pairs;

		if is_full_house {
			return Kind::FullHouse;
		} else if is_flush {
			return Kind::Flush;
		} else if is_straight {
			return Kind::Straight;
		} else if is_three_of_a_kind {
			return Kind::ThreeOfAKind;
		} else if is_two_pairs {
			return Kind::TwoPairs;
		}

		if Self::is_pair(cards) {
			return Kind::Pair;
		}

		Kind::HighCard
	}

	fn is_pair(cards: &[Card]) -> bool {
		for i in 1..5 {
			if cards[i].rank == cards[i - 1].rank {
				return true;
			}
		}

		false
	}

	fn is_two_pairs(cards: &[Card]) -> bool {
		let mut pair_cnt = 0;
		let mut l = 0;
		let mut r;
		while l < 5 {
			r = l;
			while r < 5 && cards[r].rank == cards[l].rank {
				r += 1;
			}
			if r - l > 1 {
				pair_cnt += 1;
			}

			l = r;
		}

		pair_cnt == 2
	}

	fn is_three_of_a_kind(cards: &[Card]) -> bool {
		for i in 0..3 {
			if cards[i].rank == cards[i + 1].rank && cards[i].rank == cards[i + 2].rank {
				return true;
			}
		}

		false
	}

	fn is_four_of_a_kind(cards: &[Card]) -> bool {
		if Self::is_last_four_of_a_kind(cards) {
			return true;
		}
		for i in 1..4 {
			if cards[i].rank != cards[0].rank {
				return false;
			}
		}
		true
	}

	fn is_last_four_of_a_kind(cards: &[Card]) -> bool {
		for i in 2..5 {
			if cards[i].rank != cards[1].rank {
				return false;
			}
		}
		true
	}

	fn is_straight(cards: &[Card]) -> bool {
		if Self::is_lowest_straight(cards) {
			return true;
		}

		let mut first = cards[0].rank.as_usize();
		if first == 1 {
			first = 14;
		}
		for (i, card) in cards.iter().enumerate().skip(1) {
			if card.rank.as_usize() != first - i {
				return false;
			}
		}

		true
	}

	fn is_lowest_straight(cards: &[Card]) -> bool {
		if cards[0].rank != Rank::A {
			return false;
		}
		for (i, card) in cards.iter().enumerate().skip(1) {
			if card.rank.as_usize() != (6 - i) {
				return false;
			}
		}

		true
	}

	fn is_flush(cards: &[Card]) -> bool {
		for i in 1..5 {
			if cards[i].suit != cards[0].suit {
				return false;
			}
		}
		true
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	fn parse_cards(cards: &[&str]) -> Vec<Card> {
		cards.iter().map(|s| Card::parse(s)).collect()
	}

	fn parse_hand(cards: [&str; 5]) -> Hand {
		let cards = parse_cards(&cards);
		Hand::new(&cards)
	}

	#[test]
	fn test_best_hand() {
		let common = parse_cards(&["SA", "C3", "D3", "S4", "C4"]);
		let hand = parse_cards(&["CA", "H6"]);
		let h1 = parse_hand(["CA", "SA", "D4", "S4", "H6"]);
		let h2 = Hand::calc_best_hand(&common, &hand);
		assert_eq!(h1, h2);
	}

	#[test]
	fn compare_high_card() {
		let h1 = parse_hand(["C3", "D5", "D6", "DT", "DA"]);
		let h2 = parse_hand(["S2", "H5", "H6", "HT", "HA"]);
		assert_eq!(h1.kind, Kind::HighCard);
		assert_eq!(h2.kind, Kind::HighCard);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_pair() {
		let h1 = parse_hand(["S3", "H5", "HT", "ST", "HA"]);
		let h2 = parse_hand(["SA", "H5", "H2", "CT", "CT"]);
		assert_eq!(h1.kind, Kind::Pair);
		assert_eq!(h2.kind, Kind::Pair);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_two_pairs() {
		let h1 = parse_hand(["SA", "H2", "S2", "S3", "H3"]);
		let h2 = parse_hand(["D2", "C2", "C3", "D3", "HK"]);
		dbg!(&h1, &h2);
		assert_eq!(h1.kind, Kind::TwoPairs);
		assert_eq!(h2.kind, Kind::TwoPairs);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_three_of_a_kind() {
		let h1 = parse_hand(["S3", "H5", "CA", "SA", "HA"]);
		let h2 = parse_hand(["SK", "HK", "CK", "S9", "H8"]);
		assert_eq!(h1.kind, Kind::ThreeOfAKind);
		assert_eq!(h2.kind, Kind::ThreeOfAKind);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_straight() {
		let h1 = parse_hand(["S2", "H3", "H4", "S5", "H6"]);
		let h2 = parse_hand(["SA", "H5", "H4", "S3", "H2"]);
		assert_eq!(h1.kind, Kind::Straight);
		assert_eq!(h2.kind, Kind::Straight);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_flush() {
		let h1 = parse_hand(["S2", "S3", "SK", "S5", "S7"]);
		let h2 = parse_hand(["C9", "C5", "C4", "C3", "C2"]);
		assert_eq!(h1.kind, Kind::Flush);
		assert_eq!(h2.kind, Kind::Flush);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_full_house() {
		let h1 = parse_hand(["S5", "H5", "C5", "S2", "H2"]);
		let h2 = parse_hand(["S3", "H3", "C3", "S6", "H6"]);
		assert_eq!(h1.kind, Kind::FullHouse);
		assert_eq!(h2.kind, Kind::FullHouse);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_four_of_a_kind() {
		let h1 = parse_hand(["S6", "H6", "C6", "D6", "CK"]);
		let h2 = parse_hand(["SA", "H5", "C5", "S5", "D5"]);
		assert_eq!(h1.kind, Kind::FourOfAKind);
		assert_eq!(h2.kind, Kind::FourOfAKind);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_straight_flush() {
		let h1 = parse_hand(["S9", "SK", "SQ", "SJ", "ST"]);
		let h2 = parse_hand(["CT", "CJ", "CQ", "C8", "C9"]);
		assert_eq!(h1.kind, Kind::StraightFlush);
		assert_eq!(h2.kind, Kind::StraightFlush);
		assert!(h1 > h2);
	}

	#[test]
	fn compare_royal_flush() {
		let h1 = parse_hand(["SA", "SK", "SQ", "SJ", "ST"]);
		let h2 = parse_hand(["CT", "CJ", "CQ", "CK", "CA"]);
		assert_eq!(h1.kind, Kind::RoyalFlush);
		assert_eq!(h2.kind, Kind::RoyalFlush);
		assert_eq!(h1, h2);
	}

	#[test]
	fn compare_kind() {
		let high_card = parse_hand(["SA", "HQ", "HJ", "HT", "H9"]);
		let pair = parse_hand(["SA", "HA", "HJ", "HT", "H9"]);
		let two_pairs = parse_hand(["SA", "HA", "HK", "HK", "H9"]);
		let three_of_a_kind = parse_hand(["SA", "HA", "CA", "HT", "H9"]);
		let lowest_straight = parse_hand(["S5", "H4", "C3", "H2", "HA"]);
		let straight = parse_hand(["SA", "HK", "CQ", "HJ", "HT"]);
		let flush = parse_hand(["SA", "SQ", "SJ", "ST", "S9"]);
		let full_house = parse_hand(["SA", "CA", "HA", "SK", "CK"]);
		let four_of_a_kind = parse_hand(["SA", "CA", "HA", "DA", "CK"]);
		let straight_flush = parse_hand(["SA", "S5", "S4", "S3", "S2"]);
		let royal_flush = parse_hand(["SA", "SK", "SQ", "SJ", "ST"]);

		assert_eq!(high_card.kind, Kind::HighCard);
		assert_eq!(pair.kind, Kind::Pair);
		assert_eq!(two_pairs.kind, Kind::TwoPairs);
		assert_eq!(three_of_a_kind.kind, Kind::ThreeOfAKind);
		assert_eq!(lowest_straight.kind, Kind::Straight);
		assert_eq!(straight.kind, Kind::Straight);
		assert_eq!(flush.kind, Kind::Flush);
		assert_eq!(full_house.kind, Kind::FullHouse);
		assert_eq!(four_of_a_kind.kind, Kind::FourOfAKind);
		assert_eq!(straight_flush.kind, Kind::StraightFlush);
		assert_eq!(royal_flush.kind, Kind::RoyalFlush);

		let hands = [
			high_card,
			pair,
			two_pairs,
			three_of_a_kind,
			lowest_straight,
			straight,
			flush,
			full_house,
			four_of_a_kind,
			straight_flush,
			royal_flush,
		];

		for i in 0..hands.len() {
			for j in (i + 1)..hands.len() {
				assert!(hands[i] < hands[j]);
			}
		}
	}
}
