use actix_web::{App, HttpServer};
use open_poker::api::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	tracing_subscriber::fmt::init();

	let server = HttpServer::new(move || App::new().service(guest_api()).service(room_api()))
		.bind(("127.0.0.1", 12345))?;

	server.run().await
}
