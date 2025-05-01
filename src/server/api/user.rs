use std::collections::hash_map;

use actix_web::{HttpResponse, Responder, post, web};
use serde::Deserialize;

use crate::{
	Table, User,
	server::{TableDb, UserDb},
	sprintln,
};

#[derive(Deserialize)]
struct LoginArg {
	name: String,
}

#[post("/login")]
pub async fn login(arg: web::Json<LoginArg>, user_db: web::Data<UserDb>) -> impl Responder {
	sprintln!("login: {}", arg.name);

	let mut users = user_db.users.lock().unwrap();
	let user = User::new(arg.name.as_str());

	match user {
		None => HttpResponse::BadRequest().body("reserved name"),
		Some(user) => {
			if users.contains_key(&user.id) {
				HttpResponse::BadRequest().body("name already in use")
			} else {
				users.insert(user.id.clone(), user.clone());
				HttpResponse::Ok().json(user)
			}
		}
	}
}

#[derive(Deserialize)]
struct NewArg {
	user_id: String,
}

#[post("/new")]
pub async fn new(
	arg: web::Json<NewArg>,
	user_db: web::Data<UserDb>,
	table_db: web::Data<TableDb>,
) -> impl Responder {
	sprintln!("new: {}", arg.user_id);

	let user = &user_db.users.lock().unwrap()[&arg.user_id];
	let table = Table::with_user(user);
	let mut tables = table_db.tables.lock().unwrap();

	let entry = tables.entry(table.id.clone());

	match entry {
		hash_map::Entry::Occupied(_) => HttpResponse::BadRequest().body("table ID repeat"),
		hash_map::Entry::Vacant(entry) => {
			entry.insert(table.clone());
			HttpResponse::Ok().json(table.id)
		}
	}
}

#[derive(Deserialize)]
struct JoinArg {
	user_id: String,
	table_id: String,
}

#[post("/join")]
pub async fn join(
	arg: web::Json<JoinArg>,
	user_db: web::Data<UserDb>,
	table_db: web::Data<TableDb>,
) -> impl Responder {
	sprintln!("join: {}", arg.user_id);

	let user = &user_db.users.lock().unwrap()[&arg.user_id];
	let mut tables = table_db.tables.lock().unwrap();

	match tables.entry(arg.table_id.clone()) {
		hash_map::Entry::Occupied(mut entry) => {
			entry.get_mut().insert(user.clone());
			HttpResponse::Ok().json(&arg.table_id)
		}
		hash_map::Entry::Vacant(_) => HttpResponse::BadRequest().body("table not found"),
	}
}

type ReadyArg = JoinArg;

#[post("/ready")]
pub async fn ready(arg: web::Json<ReadyArg>, table_db: web::Data<TableDb>) -> impl Responder {
	sprintln!("ready: {}", arg.user_id);

	let mut tables = table_db.tables.lock().unwrap();
	let table = tables.get_mut(&arg.table_id).unwrap();
	table.ready(&arg.user_id);

	HttpResponse::Ok().body("get ready")
}
