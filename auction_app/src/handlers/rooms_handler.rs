use axum::{extract::State, response::IntoResponse, Json};
use sqlx::{Pool ,Postgres};
use crate::{models::room_models::{Accessability, CreateRoom, RoomState, Roomid}, AppState};
use uuid::Uuid;


pub async fn create_room(State(state):State<AppState>,Json(room): Json<CreateRoom>) -> Json<Roomid> {
	// creation of middle-ware, where there should be no ongoing or starting status for user
	// who wants to create and who wants to join
	println!(" the room was {:#?} ", room) ; 

	let result = sqlx::query_as::<_,Roomid>("insert into rooms (owner_id,accessibility,room_status) values ($1,$2,$3) RETURNING room_id")
	.bind(&room.user_id)
	.bind(match room.accessibility {
		Accessability::PUBLIC => "public",
		Accessability::PRIVATE => "private"
	})
	.bind(match room.state {
		RoomState::ONGOING => "ongoing" ,
		RoomState::STARTING => "waiting",
		RoomState::COMPLETED => "completed" 
	})
	.fetch_one(&state.database_connection).await ;
	println!("the result was {:#?}", Uuid::nil());
	match result {
		Ok(res) => {
			println!("result was {:#?}", res) ;
			Json::from(res)
			// here we need to add the user as the participant as well
		},
		Err(err) => {
			println!("Error was {}", err) ;
			Json::from(Roomid{room_id: Uuid::nil()})
			// returns 00000000-0000-0000-0000-000000000000
		}
	}

} // we can allow both public and private rooms to join with the code,
// but benfit of public room was , some other x-person can also join the room


pub async fn addParticipant(database_connection:&Pool<Postgres>,user_id: i32,room_id: i32) {}


pub async fn get_public_rooms(State(state): State<AppState>) -> impl IntoResponse{
	// returns the available public_rooms

}


pub async fn join_room(State(state): State<AppState>) -> impl IntoResponse{
	// room-joining
}