use crate::error::Result;
use rusqlite::{Connection, OptionalExtension, Transaction};
use tracing::info;

/// Open connection to the database
pub fn open_connection() -> Result<Connection> {
	if cfg!(test) {
		Connection::open("test.db3").map_err(|err| err.into())
	} else {
		Connection::open("db.db3").map_err(|err| err.into())
	}
}

/// Convenience function to create a new transaction and map error
pub fn new_transaction(conn: &mut Connection) -> Result<Transaction> {
	conn.transaction().map_err(|err| err.into())
}

/// Convenience function to commit a transaction and map error
pub fn commit(tx: Transaction) -> Result<()> {
	tx.commit().map_err(|err| err.into())
}

pub fn init() -> Result<()> {
	info!("initializing database");

	let conn = open_connection()?;

	conn.execute_batch(
		"begin;
		drop table if exists seat;
		drop table if exists room;
		drop table if exists guest_token;
		drop table if exists guest;
		create table guest (
			id integer primary key autoincrement check(id > 0),
			name text not null check(
				3 <= length(name) and
				length(name) <= 32 and
				name not in ('system', 'server', 'client')
			),
			bankroll integer not null default 0
		) strict;
		create table guest_token (
			id integer references guest(id),
			token text not null unique check(length(token) = 64)
		) strict;
		create table room (
			id integer primary key autoincrement check(id > 0),
			sb integer not null default 0 check(0 <= sb and sb < 10)
		) strict;
		create table seat (
			room_id integer references room(id),
			guest_id integer references guest(id),
			number integer not null default 0 check(0 <= number and number < 10),
			unique (room_id, guest_id),
			unique (room_id, number)
		) strict;
		commit;",
	)?;

	if cfg!(debug_assertions) {
		conn.execute_batch(
			"begin;
			insert into guest(id, name) values(255, 'tester');
			insert into guest_token(id, token)
				values(255, '0123456789012345678901234567890123456789012345678901234567890123');
			commit;",
		)?;
	}

	Ok(())
}

pub(crate) fn max_id(tx: &Transaction, table: &str) -> Result<usize> {
	tx.query_row(
		format!("select id from {table} order by id desc limit 1;").as_str(),
		(),
		|row| row.get(0),
	)
	.optional()
	.map(|id| id.unwrap_or(0))
	.map_err(|err| err.into())
}
