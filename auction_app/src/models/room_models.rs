use serde::{Deserialize, Serialize};
use uuid::Uuid ;

#[derive(Debug,Deserialize)]
pub enum Accessability {
    PUBLIC,
    PRIVATE,
}

#[derive(Debug, Deserialize)]
pub enum RoomState{
	STARTING,
	ONGOING,
	COMPLETED
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
pub struct CreateRoom{
	pub user_id: i32, // creator user id
	pub accessibility: Accessability,
	pub state: RoomState
}


#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Roomid{
	pub room_id: Uuid,
}

