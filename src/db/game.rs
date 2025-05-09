use crate::{Game, Room, Round, error::Result};
use rusqlite::{OptionalExtension, Transaction};

use super::{bet, max_id};

pub fn max_game_id(tx: &Transaction) -> Result<usize> {
	max_id(tx, "game")
}

pub fn new_game(tx: &Transaction, room: &mut Room) -> Result<Game> {
	let id = max_game_id(tx)? + 1;
	tx.execute(
		"insert into game (id, room_id, position) values (?1, ?2, ?3)",
		(id, room.id, room.sb),
	)?;

	for seat in &room.seats {
		match seat {
			Some(seat) => {
				tx.execute(
					"update seat set (bet, fold) = (0, false) where room_id = ?1 and guest_id = ?2",
					(room.id, seat.guest.id),
				)?;
			}
			None => continue,
		}
	}

	let mut game = Game::new(id, room.id, room.sb);
	bet(tx, room, &mut game, 1)?;
	bet(tx, room, &mut game, 2)?;

	Ok(game)
}

/// Get game by ID
///
/// # Return
///
/// None if game not found
pub fn game_by_id(tx: &Transaction, id: usize) -> Result<Option<Game>> {
	Ok(tx
		.query_row(
			"select room_id, round, pot, position from game where id = ?1",
			(id,),
			|row| {
				Ok(Game {
					id,
					room_id: row.get(0)?,
					round: Round::parse(row.get::<usize, String>(1)?.as_str()),
					pot: row.get(2)?,
					position: row.get(3)?,
				})
			},
		)
		.optional()?)
}
