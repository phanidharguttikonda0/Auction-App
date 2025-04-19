use axum::{extract::ws::{Message, WebSocket, WebSocketUpgrade}, response::IntoResponse};

use crate::models::room_models::Roomid;

pub async fn handle_ws_upgrade(ws: WebSocketUpgrade) -> impl IntoResponse {
	ws.on_upgrade(handle_ws)
}

async fn handle_ws(mut socket: WebSocket){
	// Equivalent of `socket.onopen`
    println!("📡 Client connected");

    while let Some(Ok(msg)) = socket.recv().await {
    	match msg {
    		// nothing but socket.send() from the client
    		Message::Text(text) => {
				

    			
    		},
    		// equivalent to socket.close()
    		Message::Close(_) => {},
    		Message::Binary(_) =>{
    			// usefull for getting binary data like files and images or videos
    		},
    		Message::Ping(_) => {},
    		Message::Pong(_) => {}
    	}
    }
}

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
	participants_connections: [socket_connection: {
	team_selected, purse_remaining
	},...],
	current_player: {}, // current_bidding players
	last_bid: {}, // last bidded team with amount
	max_players: number // max amount of players should participate in auction
}

if once bid started , if with in 12 seconds if no bid was posted, then get's
unsold, if after a bid any bids were not posted within 12 seconds, player will
be sell to the last bid person and a new player will be returned and update the
current-player status and last_bid to null.



*/