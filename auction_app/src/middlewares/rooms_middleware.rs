use axum::{extract::{Request, State}, middleware::Next, response::Response};
use hyper::StatusCode;
use crate::{authorization_header_check, models::room_models::{Roomid}, AppState};


pub async fn active_room_checks(State(state): State<AppState>,req: Request,next: Next) -> Result<Response, StatusCode>{
	// checking whether there are any active rooms that user have been participating
	let (_,claims) = authorization_header_check(req.headers().get("authorization").unwrap().to_str().unwrap()) ;

	let result = sqlx::query_as::<_,Roomid>("SELECT r.room_id
			FROM participants p
			JOIN rooms r ON p.room_id = r.room_id
			WHERE p.user_id = $1
			AND r.room_status IN ('ongoing', 'waiting')")
			.bind(claims.userId)
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