use actix_web::{App, HttpServer};
use open_poker::api::*;
use open_poker::db;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	tracing_subscriber::fmt::init();

	db::init().map_err(|err| std::io::Error::other(err))?;

	let server =
		HttpServer::new(move || App::new().service(guest_api())).bind(("127.0.0.1", 12345))?;

	server.run().await
}
