
use uuid::Uuid ;


#[derive(Debug, Deserialize,Serialize)]
pub struct CreateConnection{ // for creating a new connection
	pub room_id: Uuid,
	pub participant_id: i32,
	pub max_numbers: i32,
	pub team_selected: String
}

#[derive(Debug, Deserialize,Serialize)]
pub struct Participant{
	pub room_id: Uuid,
	pub participant_id: i32,
	pub team_selected: String
}

use serde::{Serialize, Deserialize};
use std::collections::{HashMap, HashSet};

#[derive(Serialize, Deserialize)]
pub struct ParticipantConnection {
    pub team_name: String,
    pub purse_remaining: f64,
}

#[derive(Serialize, Deserialize)]
pub struct Player {
    pub player_id: u32,
    pub player_name: String,
    pub role: String,
    pub base_price: f64,
    pub age: u32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LastBid {
    pub amount: f64,
    pub team_name: String,
}

#[derive(Serialize, Deserialize)]
pub struct Room {
    pub participants: HashMap<u32, (u32, String)>, // remaining purse and participants id's and team-selected
    pub current_player: Option<Player>,
    pub last_bid: Option<LastBid>,
    pub max_players: u32,
}

impl Room {
	pub fn new(participant_id: u32, team_name: String,max_people: u32) -> Room {
		let mut participants = HashMap::new() ;
		participants.insert(participant_id, (12000,team_name)) ;
		Room {
			participants,
			current_player: None,
			last_bid: None,
			max_players: max_people
		}
	}
}
