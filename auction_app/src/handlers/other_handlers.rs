use axum::{extract::{Json, Path, State}, response::IntoResponse};

use crate::{models::other_models::{ProfileGame, Username}, AppState};



pub async fn get_username(State(state):State<AppState>, Path(username):Path<Username>) -> Json<Vec<Username>>{

	let result = sqlx::query_as::<_,Username>("select username from users where username LIKE ($1)%")
	.bind(username.username).fetch_all(&state.database_connection).await; 

	match result {
		Ok(res) => {
			Json(res)
		},
		Err(err) => {
			println!("Error was {}",err) ;
			Json(vec![])
		}
	}

}

//GET /profile/:username
pub async fn get_profile(State(state): State<AppState>,Path(username):Path<Username>) -> impl IntoResponse{}

//GET /profile/:auction_id/:username -> returns all the teams buyed players details
pub async fn get_auction(State(state): State<AppState>,Path(game_play):Path<ProfileGame>) -> impl IntoResponse{}