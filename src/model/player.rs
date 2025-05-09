use super::Card;

#[derive(Debug, Clone)]
pub struct Player {
	pub id: usize,
	pub seat: usize,
	pub name: String,
	pub stack: usize,
	pub total_bet: usize,
	pub has_folded: bool,
	pub hand: Vec<Card>,
}

impl Player {
	pub fn bet(&mut self, chips: usize) -> usize {
		assert!(chips <= self.stack);
		self.stack -= chips;
		self.total_bet += chips;

		chips
	}
}
