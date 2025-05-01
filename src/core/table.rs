use super::{Game, Player, User};

#[derive(Debug, Clone)]
pub struct Table {
	pub id: String,
	pub seats: [Option<User>; 10],
	pub sb: usize,
}

impl Table {
	pub fn new() -> Self {
		Self {
			id: "1".to_string(),
			seats: [const { None }; 10],
			sb: 0,
		}
	}

	pub fn with_user(user: &User) -> Self {
		let mut table = Self::new();
		table.seats[0] = Some(user.clone());
		table
	}

	/// Number of users
	pub fn count(&self) -> usize {
		self.seats.iter().filter(|i| i.is_some()).count()
	}

	pub fn insert(&mut self, user: User) {
		assert!(self.count() < 10);
		let mut empty = 0;
		for i in 0..10 {
			match &self.seats[i] {
				Some(u) => {
					if u.id == user.id {
						panic!();
					}
				}
				None => empty = i,
			}
		}
		self.seats[empty] = Some(user);
	}

	/// Mark the user ready
	pub fn ready(&mut self, user_id: &str) {
		for user in &mut self.seats {
			if let Some(user) = user.as_mut() {
				if user.id == user_id {
					user.ready = true;
					return;
				}
			}
		}
		panic!()
	}

	pub fn new_game(&mut self) -> Game {
		assert!(self.count() >= 2);

		let count = self.count();
		let mut sb = 0;
		for i in 0..count {
			sb = (self.sb + i) % count;
			if self.seats[sb].is_some() {
				break;
			}
		}
		self.sb = (sb + 1) % count;

		let players: Vec<_> = self
			.seats
			.iter()
			.enumerate()
			.filter_map(|(i, seat)| {
				seat.as_ref().map(|user| Player {
					seat: i,
					name: user.name.clone(),
					stack: user.stack,
					total_bet: 0,
					has_folded: false,
					hand: Vec::with_capacity(2),
				})
			})
			.collect();

		Game::new(players, sb)
	}
}
