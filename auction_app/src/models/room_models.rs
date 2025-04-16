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

#[derive(Debug, Deserialize)]
pub enum Teams{
	MumbaiIndians,
	ChennaiSuperKings,
	RoyalChallengesBengaluru,
	SunrisersHyderabad,
	KolkataKingKnightRiders,
	PunjabKings,
	DelhiCapitals,
	RajastanRoyals,
	LucknowSuperGaints,
	GujaratTitans
}

#[derive(Deserialize, Debug, sqlx::FromRow)]
pub struct CreateRoom{
	pub accessibility: Accessability,
	pub state: RoomState,
	pub team: Teams
}


#[derive(sqlx::FromRow, Debug, Serialize, Deserialize)]
pub struct Roomid{
	pub room_id: Uuid,
}

#[derive(Debug, Serialize)]
pub struct RoomCreation{
	pub room_id: Uuid,
	pub participant_id: i32
}


#[derive(Debug,sqlx::FromRow)]
pub struct Participant{
	pub participant_id: i32
}

#[derive(Debug, Deserialize)]
pub struct JoinRoomPath{
	pub room_id: String,
	pub team_name: Teams
}

#[derive(Debug,Deserialize,sqlx::FromRow, Serialize)]
pub struct PublicRoom{
	pub username: String,
	pub team_selected: String,
	pub room_id: Uuid
}

#[derive(Debug, Serialize)]
pub struct PublicRoomsReturn{
	pub public_rooms: Vec<PublicRoom>
}

#[derive(Debug, sqlx::FromRow, Serialize)]
pub struct GetTeams{
	pub amount: i32,
	pub player_name: String,
	pub role: String
}