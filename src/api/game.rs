use actix_web::{HttpResponse, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use crate::{
	db::{
		bet as execute_bet, game_by_id, guest_by_token, new_transaction, open_connection,
		room_by_id,
	},
	error::{Result, conflict_error, not_found_error, unauthorized_error},
};

#[derive(Deserialize)]
struct BetForm {
	chips: usize,
}

#[post("/{game_id}/bets")]
pub async fn bet(
	auth: BearerAuth,
	path: web::Path<usize>,
	form: web::Form<BetForm>,
) -> Result<HttpResponse> {
	let game_id = path.into_inner();
	info!("post: bet for game {game_id}");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let guest = guest_by_token(&tx, auth.token())?.ok_or(unauthorized_error("invalid token"))?;
	let mut game = game_by_id(&tx, game_id)?.ok_or(not_found_error("game not found"))?;

	if game.is_finished() {
		return Err(conflict_error("game is already finished"));
	}

	let mut room = room_by_id(&tx, game.room_id)?.unwrap();
	let player = room.get_player(game.position);

	if guest.id != player.id {
		return Err(conflict_error("it's not your turn, please wait"));
	}

	execute_bet(&tx, &mut room, &mut game, form.chips)?;

	tx.commit()?;

	Ok(HttpResponse::Created().json(json!({"room": room, "game": game})))
}

pub fn game_api() -> actix_web::Scope {
	web::scope("/games").service(bet)
}
