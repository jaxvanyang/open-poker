use rusqlite::{OptionalExtension, Transaction};

use crate::db::game_by_id;
use crate::error::{Result, conflict_error};
use crate::{Game, Guest, Room, Seat};

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
	let ready = tx.query_row(
		"select ready from seat where room_id = ?1 and guest_id = ?2",
		(room_id, guest_id),
		|row| row.get(0),
	)?;
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

	let mut stmt = tx.prepare(
		"select position, guest_id, ready, stack, bet, fold from seat where room_id = ?1",
	)?;
	let rows = stmt.query_map((id,), |row| {
		Ok((
			row.get::<usize, usize>(0)?,
			row.get(1)?,
			row.get(2)?,
			row.get(3)?,
			row.get(4)?,
			row.get(5)?,
		))
	})?;

	for row in rows {
		let (position, guest_id, ready, stack, bet, fold) = row?;
		room.seats[position] = Some(Seat {
			guest: guest_by_id(tx, guest_id)?.unwrap(),
			ready,
			stack,
			bet,
			fold,
		});
	}

	Ok(Some(room))
}

pub fn get_games(
	tx: &Transaction,
	id: usize,
	asc: bool,
	limit: usize,
	offset: usize,
) -> Result<Vec<Game>> {
	let mut stmt = tx.prepare(
		format!(
			"select id from game where room_id = ?1 order by id {} limit ?2 offset ?3",
			if asc { "ASC" } else { "DESC" }
		)
		.as_str(),
	)?;
	let rows = stmt.query_map((id, limit, offset), |row| row.get::<usize, usize>(0))?;

	let mut games = Vec::new();
	for game_id in rows {
		games.push(game_by_id(tx, game_id?)?.unwrap());
	}

	Ok(games)
}

/// Bet as the current player of the game
pub fn bet(tx: &Transaction, room: &mut Room, game: &mut Game, chips: usize) -> Result<()> {
	let max_bet = room.max_bet();
	let seat = room.seats[game.position].as_mut().unwrap();

	assert!(chips <= seat.stack);
	seat.stack -= chips;
	seat.bet += chips;
	if seat.bet > max_bet {
		game.raise_position = game.position;
	} else if !seat.allin() {
		return Err(conflict_error("should bet more"));
	}
	game.pot += chips;
	tx.execute(
		"update seat set (stack, bet) = (?1, ?2) where room_id = ?3 and guest_id = ?4",
		(seat.stack, seat.bet, room.id, seat.guest.id),
	)?;
	tx.execute(
		"update game set pot = ?1 where id = ?2",
		(game.pot, game.id),
	)?;

	game.pass(&room)?;
	tx.execute(
		"update game set position = ?1 where id = ?2",
		(game.position, game.id),
	)?;

	Ok(())
}

/// Fold as the current player of the game
pub fn fold(tx: &Transaction, room: &mut Room, game: &mut Game) -> Result<()> {
	let seat = room.seats[game.position].as_mut().unwrap();
	assert_eq!(seat.fold, false);
	seat.fold = true;
	tx.execute(
		"update seat set fold = true where room_id = ?1 and guest_id = ?2",
		(room.id, seat.guest.id),
	)?;

	game.pass(&room)?;
	tx.execute(
		"update game set position = ?1 where id = ?2",
		(game.position, game.id),
	)?;

	Ok(())
}
