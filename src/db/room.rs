use rusqlite::{OptionalExtension, Transaction};

use crate::error::Result;
use crate::{Guest, Room};

use super::{guest_by_id, max_id};

pub fn max_room_id(tx: &Transaction) -> Result<usize> {
	max_id(tx, "room")
}

pub fn new_room(tx: &Transaction, guest: &Guest) -> Result<Room> {
	let id = max_room_id(tx)? + 1;
	tx.execute("insert into room(id) values(?1)", (id,))?;
	tx.execute(
		"insert into seat(room_id, position, guest_id) values(?1, 0, ?2)",
		(id, guest.id),
	)?;
	let room = Room::with_guest(id, guest);
	Ok(room)
}

/// Whether the guest is ready in the room
pub fn is_ready(tx: &Transaction, room_id: usize, guest_id: usize) -> Result<bool> {
	let ready = tx
		.query_row(
			"select count(*) from ready where room_id = ?1 and guest_id = ?2",
			(room_id, guest_id),
			|_| Ok(true),
		)
		.optional()?
		.unwrap_or(false);
	Ok(ready)
}

/// Get room by ID
///
/// # Return
///
/// None if room not found
pub fn room_by_id(tx: &Transaction, id: usize) -> Result<Option<Room>> {
	let mut room = Room::new(id);
	let sb: Option<usize> = tx
		.query_row("select sb from room where id = ?1", (id,), |row| row.get(0))
		.optional()?;
	match sb {
		None => return Ok(None),
		Some(sb) => room.sb = sb,
	}

	let mut stmt = tx.prepare("select position, guest_id from seat where room_id = ?1")?;
	let rows = stmt.query_map((id,), |row| {
		Ok((row.get::<usize, usize>(0)?, row.get::<usize, usize>(1)?))
	})?;

	for row in rows {
		let (position, guest_id) = row?;
		let ready = is_ready(tx, id, guest_id)?;
		room.seats[position] = guest_by_id(tx, guest_id)?.map(|g| (g, ready));
	}

	Ok(Some(room))
}
