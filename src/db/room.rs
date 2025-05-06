use rusqlite::Transaction;

use crate::error::Result;
use crate::{Guest, Room};

use super::max_id;

pub fn max_room_id(tx: &Transaction) -> Result<usize> {
	max_id(tx, "room")
}

pub fn new_room(tx: &Transaction, guest: &Guest) -> Result<Room> {
	let id = max_room_id(tx)? + 1;
	tx.execute("insert into room(id) values(?1)", (id,))?;
	tx.execute(
		"insert into seat(room_id, guest_id, number) values(?1, ?2, 0)",
		(id, guest.id),
	)?;
	let room = Room::with_guest(id, guest);
	Ok(room)
}
