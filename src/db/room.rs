use std::cmp::Ordering;

use rusqlite::{OptionalExtension, Transaction};

use crate::db::{game_by_id, get_common, get_hand};
use crate::error::{Result, conflict_error};
use crate::{Game, Guest, Hand, Room, Seat};

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

/// Get current running game of the room
pub fn get_running_game(tx: &Transaction, id: usize) -> Result<Option<Game>> {
	let game = get_games(tx, id, false, 1, 0)?.pop();
	Ok(match game {
		Some(game) => {
			if game.is_over() {
				None
			} else {
				Some(game)
			}
		}
		None => None,
	})
}

/// Bet as the current player of the game
pub fn bet(tx: &Transaction, room: &mut Room, game: &mut Game, chips: usize) -> Result<()> {
	let max_bet = room.max_bet();
	let seat = room.seats[game.position].as_mut().unwrap();

	assert!(chips <= seat.stack);
	seat.stack -= chips;
	seat.bet += chips;
	game.pot += chips;
	if seat.bet > max_bet {
		game.raise_position = game.position;
		tx.execute(
			"update game set raise_position = ?1 where id = ?2",
			(game.raise_position, game.id),
		)?;
	} else if seat.bet != max_bet && !seat.allin() {
		return Err(conflict_error("should bet more"));
	}
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

/// Compute game result
///
/// # Note
///
/// Only use this function when the game is over
pub fn calc_result(tx: &Transaction, room: &mut Room, game: &Game) -> Result<()> {
	let ids: Vec<_> = room
		.seats
		.iter()
		.filter(|s| s.as_ref().is_some_and(|s| !s.fold))
		.filter_map(|s| s.as_ref().map(|s| s.guest.id))
		.collect();

	let mut win_i = 0;
	let common = get_common(tx, game)?;
	let mut hands = Vec::new();
	for id in &ids {
		hands.push(get_hand(tx, game.id, *id)?.unwrap());
	}
	for i in 1..hands.len() {
		let h1 = Hand::calc_best_hand(&common, &hands[win_i]);
		let h2 = Hand::calc_best_hand(&common, &hands[i]);
		match h1.cmp(&h2) {
			Ordering::Less => {
				win_i = i;
			}
			Ordering::Equal => todo!(),
			Ordering::Greater => (),
		}
	}

	let win_id = ids[win_i];

	// TODO: side pot logic
	for seat in room.seats.iter_mut().filter_map(|s| s.as_mut()) {
		let diff = if seat.guest.id == win_id {
			seat.stack += game.pot;
			tx.execute(
				"update seat set stack = ?1 where room_id = ?2 and guest_id = ?3",
				(seat.stack, room.id, win_id),
			)?;

			(game.pot - seat.bet) as isize
		} else {
			-(seat.bet as isize)
		};
		seat.guest.bankroll += diff;
		seat.ready = false;

		tx.execute(
			"insert into result (game_id, guest_id, diff) values (?1, ?2, ?3)",
			(game.id, seat.guest.id, diff),
		)?;
		tx.execute(
			"update guest set bankroll = ?1 where id = ?2",
			(seat.guest.bankroll, seat.guest.id),
		)?;
		tx.execute(
			"update seat set ready = false where room_id = ?1 and guest_id = ?2",
			(room.id, seat.guest.id),
		)?;
	}

	Ok(())
}
