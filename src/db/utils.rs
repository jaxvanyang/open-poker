use crate::error::Result;
use rusqlite::{Connection, OptionalExtension, Transaction};

/// Open connection to the database
pub fn open_connection() -> Result<Connection> {
	if cfg!(test) {
		Connection::open("db/test.db3").map_err(|err| err.into())
	} else {
		Connection::open("db/db.db3").map_err(|err| err.into())
	}
}

/// Convenience function to create a new transaction and map error
pub fn new_transaction(conn: &mut Connection) -> Result<Transaction> {
	conn.transaction().map_err(|err| err.into())
}

/// Convenience function to execute SQL and map error
pub fn execute<P>(tx: &Transaction, sql: &str, params: P) -> Result<usize>
where
	P: rusqlite::Params,
{
	tx.execute(sql, params).map_err(|err| err.into())
}

/// Convenience function to commit a transaction and map error
pub fn commit(tx: Transaction) -> Result<()> {
	tx.commit().map_err(|err| err.into())
}

#[cfg(test)]
pub fn init() {
	std::process::Command::new("./db/init.sh")
		.arg("test")
		.spawn()
		.unwrap()
		.wait()
		.unwrap();
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
