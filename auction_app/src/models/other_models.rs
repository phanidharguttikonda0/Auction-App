use serde::{Deserialize, Serialize};

#[derive(Debug,Deserialize, sqlx::FromRow, Serialize)]
pub struct Username{
	pub username: String
}

#[derive(Debug,Deserialize, sqlx::FromRow,Serialize)]
pub struct ProfileGame{
	pub username: String,
	pub auction_id: String,
}