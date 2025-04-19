use serde::{Deserialize, Serialize};


#[derive(Debug,Serialize,Deserialize,sqlx::FromRow)]
pub struct PoolId{
	pub pool_id: String
}

#[derive(Debug,Serialize, Deserialize)]
pub struct StatsId{
	pub stats_id: i32
}


#[derive(Debug,sqlx::FromRow,Serialize)]
pub struct Player{
	pub player_name:String,
	pub stats: i32,
	pub role: String,
	pub age: i32,
	pub country: String,
	pub capped: bool
}

#[derive(Debug,sqlx::FromRow, Serialize)]
pub struct PlayerStats{
	pub id: i32,
    pub matches: i32,
    pub total_runs : i32,
    pub average : f32,
    pub strike_rate : f32,
    pub fifties :i32,
    pub hundreads :i32,
    pub wickets : i32,
    pub three_wickets : i32,
    pub five_wickets: i32
}

