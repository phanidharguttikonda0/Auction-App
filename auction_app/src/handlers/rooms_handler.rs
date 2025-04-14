use axum::{extract::State, response::IntoResponse, Json};

use crate::{models::room_models::{Accessability, CreateRoom, RoomCreation, RoomId, RoomState}, AppState};



pub async fn create_room(State(state):State<AppState>,Json(room): Json<CreateRoom>) -> Json<RoomCreation> {
	// creation of middle-ware, where there should be no ongoing or starting status for user
	// who wants to create and who wants to join
	println!(" the room was {:#?} ", room) ; 

	let result = sqlx::query_as::<_,RoomId>("insert into rooms (owner_id,accessibility,room_status) values ($1,$2,$3)")
	.bind(&room.user_id)
	.bind(match room.accessibility {
		Accessability::PUBLIC => "public",
		Accessability::PRIVATE => "private"
	})
	.bind(match room.state {
		RoomState::ONGOING => { "ongoing" },
		RoomState::STARTING => { "starting"},
		RoomState::COMPLETED => { "completed" }
	})
	.fetch_one(&state.database_connection).await ;
	
	match result {
		Ok(res) => {
			println!("result was {:#?}", res) ;
			Json::from(RoomCreation::new(res.room_id))
		},
		Err(err) => {
			println!("Error was {}", err) ;
			Json::from(RoomCreation::new(String::from(""))) // when length of Room-Id was zero
			// then something went wrong
		}
	}

} // we can allow both public and private rooms to join with the code,
// but benfit of public room was , some other x-person can also join the room

pub async fn get_public_rooms(State(state): State<AppState>) -> impl IntoResponse{
	// returns the available public_rooms

}