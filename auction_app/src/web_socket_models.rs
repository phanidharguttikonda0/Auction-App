
use uuid::Uuid ;
use tokio::sync::mpsc::UnboundedSender;
use axum::extract::ws::Message;
use serde::{Serialize, Deserialize};

#[derive(Debug, Deserialize,Serialize)]
pub struct CreateConnection{ // for creating a new connection
	pub room_id: String,
	pub participant_id: i32,
	pub max_numbers: i32,
	pub team_selected: String
}

#[derive(Debug, Deserialize,Serialize)]
pub struct RoomConnection{
	pub room_id: String,
	pub participant_id: i32,
	pub team_selected: String
}

#[derive(Debug)]
pub struct Participant {
    pub participant_id: u32,
    pub sender: UnboundedSender<Message>, // Channel to send messages to this user
} // as it was the not able to use Serializable and Deserializable we are going to do it manually


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
    pub age: u32,
    pub stats: i32,
    pub base_price: i32,
    pub country: String,
    pub capped: bool
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bid {
    pub amount: f64,
    pub team_name: String,
    pub room_id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct LastBid {
	    pub amount: f64,
    	pub team_name: String,
}

#[derive(Debug,Deserialize)]
pub struct Ready{
	pub room_id: String
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

#[derive(Debug, Serialize, Deserialize)]
pub struct Sell{
	pub team_name: String,
	pub room_id: String,
	pub player_id: u32
}