use actix_web::{HttpResponse, delete, get, patch, post, put, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde_json::json;
use tracing::info;

use crate::{
	db::{
		commit, execute, get_games, get_running_game, guest_by_token, new_game, new_room,
		new_transaction, open_connection, room_by_id,
	},
	error::{
		Result, conflict_error, forbidden_error, internal_server_error, not_found_error,
		unauthorized_error,
	},
};

/// Create a new room
#[post("")]
pub async fn new(auth: BearerAuth) -> actix_web::Result<HttpResponse> {
	info!("post: create a new room");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let guest = guest_by_token(&tx, auth.token())?;
	let guest = guest.ok_or(unauthorized_error("invalid token"))?;
	let room = new_room(&tx, &guest)?;

	commit(tx)?;

	Ok(HttpResponse::Ok().json(json!({"room": room})))
}

/// Join a room
#[patch("/{room_id}")]
pub async fn join(auth: BearerAuth, path: web::Path<usize>) -> Result<HttpResponse> {
	let room_id = path.into_inner();
	info!("patch: join room {room_id}");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let guest = guest_by_token(&tx, auth.token())?.ok_or(unauthorized_error("invalid token"))?;
	let mut room = room_by_id(&tx, room_id)?.ok_or(not_found_error("room not found"))?;
	let position = room
		.insert(guest.clone())
		.ok_or(forbidden_error("room full or already in"))?;

	let game = get_running_game(&tx, room_id)?;
	if game.is_some() {
		return Err(forbidden_error("there is a game running, please wait"));
	}

	execute(
		&tx,
		"insert into seat(room_id, position, guest_id) values(?1, ?2, ?3)",
		(room_id, position, guest.id),
	)?;

	commit(tx)?;

	Ok(HttpResponse::Ok().json(json!({"room": room})))
}

/// Get the room and the last game
#[get("/{room_id}")]
pub async fn get_room(path: web::Path<usize>) -> Result<HttpResponse> {
	let room_id = path.into_inner();

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let room = room_by_id(&tx, room_id)?.ok_or(not_found_error("room not found"))?;

	let games = get_games(&tx, room_id, false, 1, 0)?;
	let game = games.get(0);

	commit(tx)?;

	Ok(HttpResponse::Ok().json(json!({"room": room, "game": game})))
}

/// Set the guest to be ready
#[put("/{room_id}/ready")]
pub async fn ready(auth: BearerAuth, path: web::Path<usize>) -> actix_web::Result<HttpResponse> {
	let room_id = path.into_inner();
	info!("put: ready in room {room_id}");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let guest = guest_by_token(&tx, auth.token())?;
	let guest = guest.ok_or(unauthorized_error("invalid token"))?;
	let mut room = room_by_id(&tx, room_id)?.ok_or(not_found_error("room not found"))?;

	if !room.has_guest(guest.id) {
		return Err(conflict_error("guest not in the room").into());
	}

	if room.is_ready(guest.id).unwrap() {
		return Err(conflict_error("guest is already ready").into());
	} else {
		room.ready(guest.id).unwrap();
	}

	if execute(
		&tx,
		"update seat set ready = true where room_id = ?1 and guest_id = ?2",
		(room_id, guest.id),
	)? != 1
	{
		return Err(internal_server_error("failed to set ready, please retry").into());
	}

	let mut game = None;
	if room.should_start() {
		game = Some(new_game(&tx, &mut room)?);
	}

	commit(tx)?;

	Ok(HttpResponse::Ok().json(json!({"room": room, "game": game})))
}

/// Set the guest to be unready
#[delete("/{room_id}/ready")]
pub async fn unready(auth: BearerAuth, path: web::Path<usize>) -> actix_web::Result<HttpResponse> {
	let room_id = path.into_inner();
	info!("delete: unready in room {room_id}");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let guest = guest_by_token(&tx, auth.token())?;
	let guest = guest.ok_or(unauthorized_error("invalid token"))?;
	let mut room = room_by_id(&tx, room_id)?.ok_or(not_found_error("room not found"))?;

	if !room.has_guest(guest.id) {
		return Err(conflict_error("guest not in the room").into());
	}

	if !room.is_ready(guest.id).unwrap() {
		return Err(conflict_error("guest is not ready").into());
	} else {
		room.unready(guest.id).unwrap();
	}

	if execute(
		&tx,
		"update seat set ready = false where room_id = ?1 and guest_id = ?2",
		(room_id, guest.id),
	)? != 1
	{
		return Err(internal_server_error("failed to set unready, please retry").into());
	}

	commit(tx)?;

	Ok(HttpResponse::Ok().json(json!({"room": room})))
}

/// Current game
#[get("{room_id}/game")]
pub async fn current_game(path: web::Path<usize>) -> actix_web::Result<HttpResponse> {
	let room_id = path.into_inner();
	info!("get: current game of room {room_id}");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let games = get_games(&tx, room_id, false, 1, 0)?;
	let game = games
		.get(0)
		.ok_or(not_found_error("no game has been played in this room"))?;

	Ok(HttpResponse::Ok().json(json!({"game": game})))
}

pub fn room_api() -> actix_web::Scope {
	web::scope("/rooms")
		.service(new)
		.service(join)
		.service(ready)
		.service(unready)
		.service(current_game)
		.service(get_room)
}
