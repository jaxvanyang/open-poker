use rand::{Rng, distr::Alphanumeric};
use rusqlite::{OptionalExtension, Transaction};

use crate::Guest;

use super::open_connection;

pub fn has_guest_token(tx: &Transaction, token: &str) -> anyhow::Result<bool> {
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

pub fn new_guest(tx: &Transaction, name: &str) -> anyhow::Result<Guest> {
	let max_id: usize = tx
		.query_row(
			"select id from guest order by id desc limit 1;",
			(),
			|row| row.get(0),
		)
		.optional()?
		.unwrap_or(0);
	let id = max_id + 1;
	tx.execute("insert into guest(id, name) values(?1, ?2)", (id, name))?;

	Ok(Guest::new(id, name))
}

pub fn new_guest_token(tx: &Transaction, guest: &Guest) -> anyhow::Result<String> {
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

pub fn new_guest_and_token(name: &str) -> anyhow::Result<(Guest, String)> {
	let mut conn = open_connection()?;
	let tx = conn.transaction()?;

	let guest = new_guest(&tx, name)?;
	let token = new_guest_token(&tx, &guest)?;

	tx.commit()?;

	Ok((guest, token))
}

#[cfg(test)]
mod tests {
	use crate::db;

	use super::*;

	#[test]
	fn login() {
		db::init().unwrap();
		for _ in 0..10 {
			new_guest_and_token("bob").unwrap();
		}
	}

	#[test]
	fn name_too_short() {
		db::init().unwrap();
		assert!(new_guest_and_token("a").is_err());
	}

	#[test]
	fn name_too_long() {
		db::init().unwrap();
		let name: String = ['a'; 33].iter().collect();
		assert!(new_guest_and_token(&name).is_err());
	}
}
