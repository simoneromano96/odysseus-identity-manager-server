use log::info;
use wither::{
	mongodb::{Client, Database},
	Model,
};

use crate::{models::User, settings::APP_SETTINGS};

pub async fn init_database() -> Database {
	let db = Client::with_uri_str(&APP_SETTINGS.mongo.uri)
		.await
		.expect("Cannot connect to the db")
		.database(&APP_SETTINGS.mongo.database);

	info!("Mongo database initialised");

	User::sync(&db).await.expect("Failed syncing indexes");

	db
}
