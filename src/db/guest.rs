use rand::{Rng, distr::Alphanumeric};
use rusqlite::{OptionalExtension, Transaction};

use crate::Guest;

use super::{max_id, open_connection};
use crate::error::Result;

pub fn has_guest_token(tx: &Transaction, token: &str) -> Result<bool> {
	let result = tx
		.query_row(
			"select * from guest_token where token = ?1",
			(token,),
			|_| Ok(()),
		)
		.optional()?
		.is_some();

	Ok(result)
}

pub fn max_guest_id(tx: &Transaction) -> Result<usize> {
	max_id(tx, "guest")
}

pub fn new_guest(tx: &Transaction, name: &str) -> Result<Guest> {
	let max_id = max_guest_id(tx)?;
	let id = max_id + 1;
	tx.execute("insert into guest(id, name) values(?1, ?2)", (id, name))?;

	Ok(Guest::new(id, name))
}

pub fn new_guest_token(tx: &Transaction, guest: &Guest) -> Result<String> {
	let mut rng = rand::rng();
	let mut token: String;
	loop {
		token = (0..64).map(|_| rng.sample(Alphanumeric) as char).collect();
		if !has_guest_token(&tx, &token)? {
			break;
		}
	}
	tx.execute(
		"insert into guest_token(id, token) values(?1, ?2)",
		(guest.id, &token),
	)?;

	Ok(token)
}

pub fn new_guest_and_token(name: &str) -> Result<(Guest, String)> {
	let mut conn = open_connection()?;
	let tx = conn.transaction()?;

	let guest = new_guest(&tx, name)?;
	let token = new_guest_token(&tx, &guest)?;

	tx.commit()?;

	Ok((guest, token))
}

/// Get guest by ID
///
/// # Return
///
/// None if guest not found
pub fn guest_by_id(tx: &Transaction, id: usize) -> Result<Option<Guest>> {
	Ok(tx
		.query_row(
			"select name, bankroll from guest where id = ?1",
			(id,),
			|row| {
				Ok(Guest {
					id,
					name: row.get(0)?,
					bankroll: row.get(1)?,
				})
			},
		)
		.optional()?)
}

/// Get guest by token
///
/// # Return
///
/// None if guest not found
pub fn guest_by_token(tx: &Transaction, token: &str) -> Result<Option<Guest>> {
	Ok(tx
		.query_row(
			"select g.id, name, bankroll from guest as g, guest_token as t
				where g.id = t.id and t.token = ?1",
			(token,),
			|row| {
				Ok(Guest {
					id: row.get(0)?,
					name: row.get(1)?,
					bankroll: row.get(2)?,
				})
			},
		)
		.optional()?)
}

#[cfg(test)]
mod tests {
	use crate::db;

	use super::*;

	#[test]
	fn login() {
		db::init();
		for _ in 0..10 {
			let (guest, token) = new_guest_and_token("bob").unwrap();
			let mut conn = open_connection().unwrap();
			let tx = conn.transaction().unwrap();
			assert_eq!(guest, guest_by_token(&tx, &token).unwrap().unwrap());
		}
	}

	#[test]
	fn name_too_short() {
		db::init();
		assert!(new_guest_and_token("a").is_err());
	}

	#[test]
	fn name_too_long() {
		db::init();
		let name: String = ['a'; 33].iter().collect();
		assert!(new_guest_and_token(&name).is_err());
	}
}
