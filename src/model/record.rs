#[derive(Debug)]
pub enum Action {
	Bet(usize),
	Fold,
	Deal,
}

#[derive(Debug)]
pub struct Record {
	pub seat: usize,
	pub action: Action,
}

pub type Records = Vec<Record>;
