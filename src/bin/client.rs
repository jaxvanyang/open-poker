use open_poker::{client::*, sprintln};

#[actix_web::main]
async fn main() {
	let mut client = Client::default();

	loop {
		let result = client.run().await;
		if let Err(err) = result {
			sprintln!("command failed: {err}");
		}
	}
}
