use actix_web::{App, HttpServer, web};
use open_poker::server::*;
use open_poker::*;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
	let user_db = web::Data::new(UserDb::default());
	let table_db = web::Data::new(TableDb::default());

	let server = HttpServer::new(move || {
		App::new()
			.app_data(user_db.clone())
			.app_data(table_db.clone())
			.service(login)
			.service(new)
			.service(join)
	})
	.bind(("127.0.0.1", 12345))?;

	sprintln!("start running...");
	server.run().await
}
