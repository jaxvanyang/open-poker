use actix_web::{HttpResponse, post, web};
use actix_web_httpauth::extractors::bearer::BearerAuth;
use serde_json::json;
use tracing::info;

use crate::{
	db::{commit, guest_by_token, new_room, new_transaction, open_connection},
	error::unauthorized_error,
};

/// Create a new room
#[post("")]
pub async fn new(auth: BearerAuth) -> actix_web::Result<HttpResponse> {
	info!("processing new room");

	let mut conn = open_connection()?;
	let tx = new_transaction(&mut conn)?;

	let guest = guest_by_token(&tx, auth.token())?;
	let guest = guest.ok_or(unauthorized_error("wrong token"))?;
	let room = new_room(&tx, &guest)?;

	commit(tx)?;

	Ok(HttpResponse::Ok().json(json!({"room": room})))
}

pub fn room_api() -> actix_web::Scope {
	web::scope("/rooms").service(new)
}
