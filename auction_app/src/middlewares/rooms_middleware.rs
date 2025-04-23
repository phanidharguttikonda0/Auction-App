use axum::{extract::{ Request, State}, middleware::Next,Extension, response::Response};
use hyper::StatusCode;
use sqlx::{Pool, Postgres};
use uuid::Uuid;
use crate::{models::{authentication_models::Claims, room_models::{Roomid, Users}}, AppState};


pub async fn get_user_id(participant_id: i32, database_connection: &Pool<Postgres>) -> i32 {
	let result = sqlx::query_as::<_,Users>("select user_id from participants where participant_id = ($1)").bind(&participant_id).fetch_one(database_connection).await ;
	match result {
		Ok(res) => res.user_id,
		Err(_) => -1
	}
}



pub async fn active_room_checks(State(state): State<AppState>,req: Request,next: Next) -> Result<Response, StatusCode>{
	// checking whether there are any active rooms that user have been participating
	let user = req.extensions().get::<Claims>().unwrap();
	let result = sqlx::query_as::<_,Roomid>("SELECT r.room_id
			FROM participants p
			JOIN rooms r ON p.room_id = r.room_id
			WHERE p.user_id = $1
			AND r.room_status IN ('ongoing', 'waiting')
			")
			.bind(user.userId)
			.fetch_one(&state.database_connection).await ;
	match result {
		Ok(res) => {
			println!("Existed room id was {}", res.room_id);
			println!("no error where result found means there is a room already exists with this participant ") ;
			Err(StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS) 
		}
		Err(_) => {
			println!("No room active room was found to be exists for the user");
			Ok(next.run(req).await)
		}
	}

}


pub async fn room_id_check(State(state): State<AppState>,req: Request,next: Next) -> Result<Response, StatusCode> {
	// where checks whether room_id was active or not and exists
	// it's a room-Id validation 

	let uri_path = req.uri().path();
	let segments: Vec<&str> = uri_path.trim_start_matches('/').split('/').collect();

	let room_id = Uuid::parse_str(segments[2]);

	let result = sqlx::query_as::<_,Roomid>("select room_id from rooms where room_id = ($1) and 
		room_status IN ('waiting')
		").bind(match room_id {
			Ok(res) => res,
			Err(err) => {
				println!("invalid uuid passed") ;
				println!("error was {}",err) ;
				Uuid::nil()
			}
		}).fetch_one(&state.database_connection).await ;
	match result {
		Ok(res) => {
			println!("Result was {:#?}", res);
			println!("Room exists with waiting state");
			Ok(next.run(req).await)
		},
		Err(err) => {
			println!("Error was {}", err) ;
			println!("May be invalid uuid was passed") ;
			Err(StatusCode::NOT_ACCEPTABLE)
		}
	}
}