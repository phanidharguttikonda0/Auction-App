use axum::{extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State}, response::IntoResponse};

use crate::{models::players_models::Player, web_socket_models::{CreateConnection, LastBid, Room}, AppState};


pub async fn handle_ws_upgrade(ws: WebSocketUpgrade, State(connections): State<AppState>) -> impl IntoResponse {
	ws.on_upgrade(move |socket| handle_ws(socket, connections))

}



// redis allows atomicity
async fn handle_ws(mut socket: WebSocket, connections:AppState){
	// Equivalent of `socket.onopen`
    println!("📡 Client connected");

    
}


async fn create_room(room: Room) -> bool {
	true
}




async fn get_next_player(player_id: i32) -> Player {}



/* 
BroadCasts the messages to the participants in that specific room
async fn broadcast_to_room(
    room_id: &str,
    msg: &str,
    connections: &Connections,
    sender_id: &str,
) {
    let map = connections.read().await;
    if let Some(participants) = map.get(room_id) {
        for participant in participants {
            if participant.participant_id != sender_id {
                let _ = participant.sender.send(Message::Text(msg.to_string()));
            }
        }
    }
}


*/



/* 

When every one joins an room the front-end creates and websocket connection
and sends the websocket along with room-id, and also when room was created
intially then along with the room-id and web-socket connection the max members
limit participating in the auction will also be sent, so in the redis, it should
store the room-id as the key, and the values are :
Once participants_connections reaches the max_players we are going to send an
message does we start auction now. In response we need Yes. Now we will start 
auction.

room_id : {
	participants: {participant_ids} // set this will be usefull after unfortunately if an user goes back and comes again to join , checking whether user was already exists or not
	participants_connections: [socket_connection: [
	team_selected(String), purse_remaining(float)
	],...],
	current_player: {}, // current_bidding players
	last_bid: {}, // last bidded team with amount
	max_players: number // max amount of players should participate in auction
}

if once bid started , if with in 12 seconds if no bid was posted, then get's
unsold, if after a bid any bids were not posted within 12 seconds, player will
be sell to the last bid person and a new player will be returned and update the
current-player status and last_bid to null.



*/


// firstly take CreateConnection
// secondly add a new participant using Room-Creation, check whether reached to the max players
// thirdly recieve the bids and sends back the response as next user along with sold to team name
// it will not send response until and unless the current bid has ended
// if all teams reaches 18 players, then allow them to choose
// the players they want , such that they can send the players they want
//  from then the socket sends only those players
// if a player leaves, in middle he can join back again