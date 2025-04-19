use axum::{extract::{Path, State}, Json};

use crate::{models::{players_models::{Player, PlayerStats, PoolId, StatsId}, room_models::Roomid}, AppState};



pub async fn get_players(State(state): State<AppState>,Path(pool_id):Path<PoolId>)
-> Json<Vec<Player>>{

	let result = sqlx::query_as::<_,Player>("select 
	stats,player_name,age,country,role,capped
	 from players where pool = ($1)
		").bind(pool_id.pool_id).fetch_all(&state.database_connection).await ;
	match result {
	    Ok(res) => {
	    	println!("we got the result");
	    	Json(res)
	    },
	    Err(err) => {
	    	println!("error was {}", err) ;
	    	Json(vec![])
	    }
	}
}

pub async fn get_player(State(state):State<AppState>,Path(stats_id):Path<StatsId>)
-> Json<Option<PlayerStats>> {
	let result = sqlx::query_as::<_,PlayerStats>("
		select * from stats where stats_id = ($1)
		").bind(stats_id.stats_id).fetch_one(&state.database_connection).await;
	match result{
		Ok(res) => {
			println!("we got result");
			Json::from(Some(res))
		},
		Err(err) => {
			println!("Error was {}",err) ;
			Json::from(None)
		}
	}
}

pub async fn get_unsold_players(State(state):State<AppState>,Path(room):
Path<Roomid>) -> Json<Vec<Player>> {

	let result = sqlx::query_as::<_,Player>(
		"select p.stats,p.player_name,p.age,p.country,p.role,p.capped from 
		players_unsold pu Join players p ON p.player_id = pu.player_id where
		pu.room_id = ($1)"
		).bind(room.room_id).fetch_all(&state.database_connection).await;
	match result {
		Ok(res) => {
			println!("we found result as well") ;
			Json(res)
		},
		Err(err) => {
			println!("Error was {}", err) ;
			Json(vec![])
		}
	}

}