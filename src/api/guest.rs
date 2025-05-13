use actix_web::{HttpResponse, post, web};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info};

use crate::db::new_guest_and_token;
use crate::error::Result;

#[derive(Deserialize)]
struct LoginForm {
	name: String,
}

#[post("")]
pub async fn login(form: web::Form<LoginForm>) -> Result<HttpResponse> {
	info!("processing login for guest '{}'", form.name);

	let (guest, token) = new_guest_and_token(&form.name).map_err(|err| {
		error!(name = form.name, "{err}");
		err
	})?;

	Ok(HttpResponse::Created().json(json!({
		"guest": guest,
		"token": token,
	})))
}

#[must_use]
pub fn guest_api() -> actix_web::Scope {
	web::scope("/guests").service(login)
}
