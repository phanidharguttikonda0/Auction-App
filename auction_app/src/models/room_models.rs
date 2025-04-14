use serde::{Deserialize, Serialize};


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
	pub user_id: String, // creator user id
	pub accessibility: Accessability,
	pub state: RoomState
}

#[derive(Serialize, Debug)]
pub struct RoomCreation{
	room_id: String,
}

impl RoomCreation{
	pub fn new(room_id:String) -> RoomCreation {
		RoomCreation { room_id }
	}
}

#[derive(sqlx::FromRow, Debug)]
pub struct RoomId{
	pub room_id: String
}