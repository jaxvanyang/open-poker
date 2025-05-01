use std::{collections::HashMap, sync::Mutex};

use crate::{Table, User};

#[derive(Default)]
pub struct UserDb {
	pub users: Mutex<HashMap<String, User>>,
}

#[derive(Default)]
pub struct TableDb {
	pub tables: Mutex<HashMap<String, Table>>,
}
