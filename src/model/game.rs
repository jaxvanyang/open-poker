use crate::Room;

use super::{Action, Card, Deck, Player, Record, Records};

#[derive(Debug, PartialEq)]
pub enum Round {
	PreFlop,
	Flop,
	Turn,
	River,
	Finish,
}

#[derive(Debug)]
pub struct Game {
	pub id: usize,
	pub players: Vec<Player>,
	pub current: usize,
	pub max_bet: usize,
	pub pot: usize,
	pub round: Round,
	pub public: Vec<Card>,
	pub records: Records,
	deck: Deck,
}

impl Game {
	pub const MIN_STACK: usize = 10;

	pub fn new(players: Vec<Player>) -> Self {
		assert!(2 <= players.len() && players.len() <= Room::MAX_SEATS);
		Self {
			id: 1,
			players,
			current: 0,
			max_bet: 0,
			pot: 0,
			round: Round::PreFlop,
			public: Vec::with_capacity(5),
			deck: Card::new_deck(),
			records: vec![],
		}
	}

	/// Pass round to the next player
	pub fn pass(&mut self) {
		let count = self.count();
		for i in 1..count {
			let next = (self.current + i) % count;
			let player = &self.players[next];
			if !player.has_folded && player.stack > 0 {
				self.current = next;
				return;
			}
		}
	}

	/// Number of players
	pub fn count(&self) -> usize {
		self.players.len()
	}

	/// Number of players have folded
	pub fn fold_count(&self) -> usize {
		self.players
			.iter()
			.filter(|player| player.has_folded)
			.count()
	}

	/// Number of players have all in but haven't folded
	fn allin_count(&self) -> usize {
		self.players
			.iter()
			.filter(|p| !p.has_folded && p.stack == 0)
			.count()
	}

	/// Number of players have max bet but haven't folded or all in
	fn bet_count(&self) -> usize {
		self.players
			.iter()
			.filter(|p| !p.has_folded && p.stack != 0 && p.total_bet == self.max_bet)
			.count()
	}

	pub fn should_deal(&self) -> bool {
		self.round != Round::River
			&& self.count() == self.fold_count() + self.allin_count() + self.bet_count()
	}

	/// Get current player
	fn player(&mut self) -> &mut Player {
		&mut self.players[self.current]
	}

	/// Bet as current player
	fn bet(&mut self, chips: usize) {
		self.pot += self.player().bet(chips);
		self.max_bet = self.player().total_bet;

		self.records.push(Record {
			seat: self.current,
			action: Action::Bet(chips),
		});
		if self.should_deal() {
			self.deal();
		}

		self.pass();
	}

	/// Fold as current player
	pub fn fold(&mut self) {
		self.player().has_folded = true;

		self.records.push(Record {
			seat: self.current,
			action: Action::Fold,
		});

		self.pass();
	}

	/// Check as current player
	pub fn check(&mut self) {
		assert!(self.player().total_bet == self.max_bet);
		self.bet(0);
	}

	/// Call as current player
	pub fn call(&mut self) {
		let chips = self.max_bet.checked_sub(self.player().total_bet).unwrap();
		assert!(0 < chips);
		self.bet(chips);
	}

	/// Raise as current player
	pub fn raise(&mut self, chips: usize) {
		assert!(self.player().total_bet < self.max_bet);
		let chips = chips + self.max_bet - self.player().total_bet;
		self.bet(chips);
	}

	/// All in as current player
	pub fn allin(&mut self) {
		let chips = self.player().stack;
		assert!(self.max_bet <= self.player().total_bet + chips);
		self.bet(chips);
	}

	pub fn start(&mut self) {
		assert!(self.pot == 0);
		assert!(self.players.iter().all(|p| p.stack >= Self::MIN_STACK));
		for player in &mut self.players {
			player.hand.push(self.deck.pop().unwrap());
			player.hand.push(self.deck.pop().unwrap());
		}
		self.bet(1);
		self.bet(2);
	}

	pub fn deal(&mut self) {
		match self.round {
			Round::PreFlop => {
				for _ in 0..3 {
					self.public.push(self.deck.pop().unwrap());
				}
				self.round = Round::Flop;
			}
			Round::Flop => {
				self.public.push(self.deck.pop().unwrap());
				self.round = Round::Turn;
			}
			Round::Turn => {
				self.public.push(self.deck.pop().unwrap());
				self.round = Round::River;
			}
			_ => panic!(),
		}
		self.records.push(Record {
			seat: self.current,
			action: Action::Deal,
		});
	}
}
