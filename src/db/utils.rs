use anyhow::anyhow;
use rusqlite::Connection;
use tracing::info;

/// Open connection to the database
pub fn open_connection() -> anyhow::Result<Connection> {
	if cfg!(test) {
		Connection::open("test.db3").map_err(|err| anyhow!("open db error: {}", err))
	} else {
		Connection::open("db.db3").map_err(|err| anyhow!("open db error: {}", err))
	}
}

pub fn init() -> anyhow::Result<()> {
	info!("initializing database");

	let conn = open_connection()?;

	conn.execute_batch(
		"begin;
		drop table if exists guest_token;
		drop table if exists guest;
		create table guest (
			id integer primary key autoincrement,
			name text not null check(
				3 <= length(name) and
				length(name) <= 32 and
				name not in ('system', 'server', 'client')
			),
			bankroll integer default 0
		) strict;
		create table guest_token (
			id integer references guest (id),
			token text not null unique check(length(token) = 64)
		) strict;
		commit;",
	)?;

	Ok(())
}
