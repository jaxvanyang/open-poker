use actix_web::{HttpResponse, patch, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde_json::json;
use tracing::info;

use crate::{
	db::{commit, execute, guest_by_token, new_room, new_transaction, open_connection, room_by_id},
	error::{conflict_error, not_found_error, unauthorized_error},
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
pub async fn join(auth: BearerAuth, path: web::Path<usize>) -> actix_web::Result<HttpResponse> {
	let room_id = path.into_inner();
	info!("patch: join room {room_id}");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let guest = guest_by_token(&tx, auth.token())?;
	let guest = guest.ok_or(unauthorized_error("invalid token"))?;
	let mut room = room_by_id(&tx, room_id)?.ok_or(not_found_error("room not found"))?;
	let position = room
		.insert(guest.clone())
		.ok_or(conflict_error("room full or already in"))?;

	execute(
		&tx,
		"insert into seat(room_id, position, guest_id) values(?1, ?2, ?3)",
		(room_id, position, guest.id),
	)?;

	commit(tx)?;

	Ok(HttpResponse::Ok().json(json!({"room": room})))
}

pub fn room_api() -> actix_web::Scope {
	web::scope("/rooms").service(new).service(join)
}
