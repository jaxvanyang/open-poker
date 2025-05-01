use std::{error::Error, io::Write};

use open_poker::{client::Client, *};

#[actix_web::main]
async fn main() -> Result<(), Box<dyn Error>> {
	let mut client = Client::default();
	loop {
		let mut input = String::new();
		print!("> ");
		std::io::stdout().flush()?;
		std::io::stdin().read_line(&mut input).unwrap();

		if input.is_empty() {
			println!();
			return Ok(());
		}
		let command = input.split_whitespace().collect::<Vec<_>>();

		match command[..] {
			[] => {
				continue;
			}
			["exit" | ""] => break,
			["login", name] => client.login(name).await?,
			["new"] => client.new_game().await?,
			["join", table_id] => client.join(table_id).await?,
			["ready"] => client.ready().await?,
			["fold"] => client.fold().await?,
			["check"] => client.check().await?,
			["call"] => client.call().await?,
			["raise", chips] => client.raise(chips.parse::<usize>().unwrap()).await?,
			["allin"] => client.allin().await?,
			_ => cprintln!("unknown command!"),
		}
	}

	Ok(())
}
