use actix_web::{HttpResponse, post, web};
use serde::Deserialize;
use serde_json::json;
use tracing::{error, info};

use crate::{db::new_guest_and_token, error::internal_server_error};

#[derive(Deserialize)]
struct LoginForm {
	name: String,
}

#[post("")]
pub async fn login(form: web::Form<LoginForm>) -> actix_web::Result<HttpResponse> {
	info!("processing login for guest '{}'", form.name);

	let (guest, token) = new_guest_and_token(&form.name).map_err(|err| {
		error!(name = form.name, "{err}");
		internal_server_error("failed to create guest, please retry")
	})?;

	Ok(HttpResponse::Created().json(json!({
		"guest": guest,
		"token": token,
	})))
}

pub fn guest_api() -> actix_web::Scope {
	web::scope("/guests").service(login)
}
