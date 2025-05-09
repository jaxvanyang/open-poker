use actix_web::{HttpResponse, get, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde::Deserialize;
use serde_json::json;
use tracing::info;

use crate::{
	Round,
	db::{
		bet as execute_bet, fold as execute_fold, game_by_id, get_flop, get_hand, get_river,
		get_turn, guest_by_id, guest_by_token, new_transaction, open_connection, room_by_id,
	},
	error::{Result, bad_request_error, forbidden_error, not_found_error, unauthorized_error},
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
		return Err(forbidden_error("game is already finished"));
	}

	let mut room = room_by_id(&tx, game.room_id)?.unwrap();
	let player = room.get_guest(game.position).unwrap();

	if guest.id != player.id {
		return Err(forbidden_error("it's not your turn, please wait"));
	}

	execute_bet(&tx, &mut room, &mut game, form.chips)?;

	tx.commit()?;

	Ok(HttpResponse::Created().json(json!({"room": room, "game": game})))
}

#[post("/{game_id}/fold")]
pub async fn fold(auth: BearerAuth, path: web::Path<usize>) -> Result<HttpResponse> {
	let game_id = path.into_inner();
	info!("post: fold for game {game_id}");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let guest = guest_by_token(&tx, auth.token())?.ok_or(unauthorized_error("invalid token"))?;
	let mut game = game_by_id(&tx, game_id)?.ok_or(not_found_error("game not found"))?;

	if game.is_finished() {
		return Err(forbidden_error("game is already finished"));
	}

	let mut room = room_by_id(&tx, game.room_id)?.unwrap();
	let player = room.get_guest(game.position).unwrap();

	if guest.id != player.id {
		return Err(forbidden_error("it's not your turn, please wait"));
	}

	execute_fold(&tx, &mut room, &mut game)?;

	tx.commit()?;

	Ok(HttpResponse::Created().json(json!({"room": room, "game": game})))
}

#[get("/{game_id}/hands/{guest_id}")]
pub async fn hand(auth: BearerAuth, path: web::Path<(usize, usize)>) -> Result<HttpResponse> {
	let (game_id, guest_id) = path.into_inner();

	let mut conn = open_connection()?;
	let tx = conn.transaction()?;

	// let mut game = game_by_id(&tx, game_id)?.ok_or(not_found_error("game not found"))?;
	let guest = guest_by_token(&tx, auth.token())?.ok_or(unauthorized_error("invalid token"))?;
	let request_guest =
		guest_by_id(&tx, guest_id)?.ok_or(not_found_error("request guest not found"))?;

	if guest.id != request_guest.id {
		return Err(bad_request_error(
			"check other's hand is not supported for now",
		));
	}

	let hand = get_hand(&tx, game_id, request_guest.id)?.unwrap();

	tx.commit()?;

	Ok(HttpResponse::Ok().json(json!({"hand": hand})))
}

#[get("/{game_id}/flop")]
pub async fn flop(path: web::Path<usize>) -> Result<HttpResponse> {
	let game_id = path.into_inner();

	let mut conn = open_connection()?;
	let tx = conn.transaction()?;

	let game = game_by_id(&tx, game_id)?.ok_or(not_found_error("game not found"))?;

	if game.round < Round::Flop {
		return Err(forbidden_error("game is still before flop, please wait"));
	}

	let flop = get_flop(&tx, game_id)?.unwrap();

	tx.commit()?;

	Ok(HttpResponse::Ok().json(json!({"flop": flop})))
}

#[get("/{game_id}/turn")]
pub async fn turn(path: web::Path<usize>) -> Result<HttpResponse> {
	let game_id = path.into_inner();

	let mut conn = open_connection()?;
	let tx = conn.transaction()?;

	let game = game_by_id(&tx, game_id)?.ok_or(not_found_error("game not found"))?;

	if game.round < Round::Turn {
		return Err(forbidden_error("game is still before turn, please wait"));
	}

	let turn = get_turn(&tx, game_id)?.unwrap();

	tx.commit()?;

	Ok(HttpResponse::Ok().json(json!({"turn": turn})))
}

#[get("/{game_id}/river")]
pub async fn river(path: web::Path<usize>) -> Result<HttpResponse> {
	let game_id = path.into_inner();

	let mut conn = open_connection()?;
	let tx = conn.transaction()?;

	let game = game_by_id(&tx, game_id)?.ok_or(not_found_error("game not found"))?;

	if game.round < Round::River {
		return Err(forbidden_error("game is still before river, please wait"));
	}

	let river = get_river(&tx, game_id)?.unwrap();

	tx.commit()?;

	Ok(HttpResponse::Ok().json(json!({"river": river})))
}

pub fn game_api() -> actix_web::Scope {
	web::scope("/games")
		.service(bet)
		.service(fold)
		.service(hand)
		.service(flop)
		.service(turn)
		.service(river)
}
