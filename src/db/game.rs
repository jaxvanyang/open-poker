use crate::{Card, Game, GameResult, Room, error::Result};
use rusqlite::{OptionalExtension, Transaction};

use super::{bet, max_id};

pub fn max_game_id(tx: &Transaction) -> Result<usize> {
	max_id(tx, "game")
}

pub fn new_game(tx: &Transaction, room: &mut Room) -> Result<Game> {
	// correct room first
	room.correct();
	tx.execute("update room set sb = ?1 where id = ?2", (room.sb, room.id))?;

	let id = max_game_id(tx)? + 1;
	tx.execute(
		"insert into game (id, room_id, position) values (?1, ?2, ?3)",
		(id, room.id, room.sb),
	)?;

	let mut deck = Card::new_deck();

	for seat in &mut room.seats {
		match seat {
			Some(seat) => {
				seat.bet = 0;
				seat.fold = false;
				tx.execute(
					"update seat set (bet, fold) = (0, false) where room_id = ?1 and guest_id = ?2",
					(room.id, seat.guest.id),
				)?;
				let (c1, c2) = (deck.pop().unwrap(), deck.pop().unwrap());
				tx.execute(
					"insert into hand (game_id, guest_id, c1, c2) values (?1, ?2, ?3, ?4)",
					(id, seat.guest.id, c1, c2),
				)?;
			}
			None => continue,
		}
	}

	tx.execute(
		"insert into flop (game_id, c1, c2, c3) values (?1, ?2, ?3, ?4)",
		(
			id,
			deck.pop().unwrap(),
			deck.pop().unwrap(),
			deck.pop().unwrap(),
		),
	)?;
	tx.execute(
		"insert into turn (game_id, card) values (?1, ?2)",
		(id, deck.pop().unwrap()),
	)?;
	tx.execute(
		"insert into river (game_id, card) values (?1, ?2)",
		(id, deck.pop().unwrap()),
	)?;

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
			"select room_id, round, pot, position, raise_position from game where id = ?1",
			(id,),
			|row| {
				Ok(Game {
					id,
					room_id: row.get(0)?,
					round: row.get(1)?,
					pot: row.get(2)?,
					position: row.get(3)?,
					raise_position: row.get(4)?,
				})
			},
		)
		.optional()?)
}

pub fn get_hand(tx: &Transaction, game_id: usize, guest_id: usize) -> Result<Option<Vec<Card>>> {
	Ok(tx
		.query_row(
			"select c1, c2 from hand where game_id = ?1 and guest_id = ?2",
			(game_id, guest_id),
			|row| Ok(vec![row.get(0)?, row.get(1)?]),
		)
		.optional()?)
}

pub fn get_flop(tx: &Transaction, game_id: usize) -> Result<Option<Vec<Card>>> {
	Ok(tx
		.query_row(
			"select c1, c2, c3 from flop where game_id = ?1",
			(game_id,),
			|row| Ok(vec![row.get(0)?, row.get(1)?, row.get(2)?]),
		)
		.optional()?)
}

pub fn get_turn(tx: &Transaction, game_id: usize) -> Result<Option<Card>> {
	Ok(tx
		.query_row(
			"select card from turn where game_id = ?1",
			(game_id,),
			|row| row.get(0),
		)
		.optional()?)
}

pub fn get_river(tx: &Transaction, game_id: usize) -> Result<Option<Card>> {
	Ok(tx
		.query_row(
			"select card from river where game_id = ?1",
			(game_id,),
			|row| row.get(0),
		)
		.optional()?)
}

pub fn update_round(tx: &Transaction, room: &Room, game: &mut Game) -> Result<bool> {
	let result = game.update(room);

	if result {
		tx.execute(
			"update game set round = ?1 where id = ?2",
			(game.round, game.id),
		)?;
	}

	Ok(result)
}

pub fn get_results(tx: &Transaction, game_id: usize) -> Result<Vec<GameResult>> {
	let mut stmt = tx.prepare("select guest_id, diff from result where game_id = ?1")?;
	let mut results = Vec::new();
	for result in stmt.query_map((game_id,), |row| {
		Ok(GameResult::new(game_id, row.get(0)?, row.get(1)?))
	})? {
		results.push(result?);
	}

	Ok(results)
}
