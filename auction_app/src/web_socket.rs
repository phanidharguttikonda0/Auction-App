use axum::{extract::{ws::{Message, WebSocket, WebSocketUpgrade}, State}, response::IntoResponse};
use futures::{StreamExt, SinkExt}; // Needed for `.next()` and `.send()`
use crate::{ web_socket_models::{Bid,Sell, CreateConnection, LastBid, Ready, Room, RoomConnection,Player}, AppState};
use tokio::sync::mpsc::unbounded_channel;
use crate::Participant;


/*
this following function does 2 things one is websocket hand shake and other is calls the handle_ws inside a spawned async task
Because handle_ws() is async and is passed to on_upgrade, Tokio spawns it internally as a task that will run independently for each client.
on_upgrade(|socket| handle_ws(socket, state))
it's basically as ,
tokio::spawn(async move {
    handle_ws(socket, state).await;
}); // for each client  a new task will be executed

*/
pub async fn handle_ws_upgrade(ws: WebSocketUpgrade, State(connections): State<AppState>) -> impl IntoResponse {
	ws.on_upgrade(move |socket| handle_ws(socket, connections))
}



// redis allows atomicity
async fn handle_ws(mut socket: WebSocket, connections:AppState){ // for each new websocket connection this will be executed once if 10k web socket connections were created then 10k times it will be called
	// Equivalent of `socket.onopen`
    println!("📡 Client connected");
    // Create an unbounded channel to send messages to this client
    let (tx, mut rx) = unbounded_channel::<Message>() ; // for each websocket connection this will be created once

    // we split the websocket in to 2 parts because , we are sending data in a different task and recieving in a different task both are not in a single task
    let (mut sender, mut receiver) = socket.split() ; // for each websocket connection this will be created once
    tokio::spawn(async move { // we will send messages
        while let Some(msg) = rx.recv().await { // rx is an unbounded reciever where we actually send the data through it , so when recieves data we will send the data via the sender socket
            if let Err(e) = sender.send(msg).await {
                println!("❌ Failed to send WS message to client: {:?}", e);
                break;
            }
        } // this loop never ends keeps on waiting until sender socket was disconnected
    }); // this task also never ends because of it's loop, where it keeps on waiting if there are any messages that are brodcasted such that it will actually sent to the client



    // this loop will never end until the client disconnects, the loop will be running continously
    while let Some(Ok(msg)) = receiver.next().await { // we recieve messages from the client

    	match msg {
    		Message::Text(text) => {
    			
    			let room_creation = serde_json::from_str::<CreateConnection>(&text);
    			let room_join = serde_json::from_str::<RoomConnection>(&text) ;
    			let bid = serde_json::from_str::<Bid>(&text) ;
    			let ready = serde_json::from_str::<Ready>(&text) ;
                let sell = serde_json::from_str::<Sell>(&text) ;
    			if let Ok(create) = room_creation {

    				let participant = Participant {
    					participant_id: create.participant_id as u32,
    					sender: tx.clone()
    				};

    				// we are going to store this transaction to the connections
    				let mut state = connections.websocket_connections.write().await;
    				state.entry(create.room_id.clone()).or_default().push(participant);
                    drop(state);
    				// now we need to add to redis,after storing the socket, if the user
    				// disconnects and join again, then we need to check whether the redis already
    				// contains or not, if it contains then continue, with out adding to redis again

    				// we need to store it in the redis

    				//sending data to all the members in the room
    				broadcast_message(&connections, Message::Text(
    					serde_json::to_string::<CreateConnection>(&create).unwrap()
    					), create.room_id).await;

    			}else if let Ok(room_join) = room_join {
    				let participant = Participant {
    					participant_id: room_join.participant_id as u32,
    					sender: tx.clone()
    				};

    				// we are going to store this transaction to the connections
    				let mut state = connections.websocket_connections.write().await;
    				state.entry(room_join.room_id.clone()).or_default().push(participant);
                    drop(state);
    				// we need to store it in the redis

    				//sending data to all the members in the room, when a new member joined
    				broadcast_message(&connections, Message::Text(
    					serde_json::to_string::<RoomConnection>(&room_join).unwrap()
    				), room_join.room_id).await;

    				// checking whether max members joined or not if , then we will start the auction

    			}else if let Ok(bid) = bid {
    				// now we need to add the last bid to the redis and broadcast the message to all the users in the room

                    broadcast_message(&connections, Message::Text(
                        serde_json::to_string::<LastBid>(&LastBid{
                            amount: bid.amount,
                            team_name: bid.team_name
                        }).unwrap()
                        ), bid.room_id).await ;

    			}else if let Ok(ready) = ready { // this can be called only by the creator of the  room, we need to set it up in the front-end itself
    				// getting ready
    				broadcast_message(&connections, Message::Text(
    					serde_json::to_string::<Player>(&get_next_player(1).await).unwrap()
    					), ready.room_id).await ;
    			}
                else if let Ok(sell) = sell {
                    // we are going to sell this player
                    broadcast_message(&connections, Message::Text(serde_json::to_string::<Sell>(&sell).unwrap()),
                        sell.room_id.clone()
                        ).await;
                    // now we need to add this player to psql database

                    // we need to update the purse in the redis

                    // we need to  broadcast next player to the specific room
                    broadcast_message(&connections, Message::Text(serde_json::to_string::<Player>(
                        &get_next_player((sell.player_id+1) as i32).await).unwrap()
                        ), sell.room_id).await ;
                }
                else{
    				println!("It's neither of the above");
    			}



    		},
    		Message::Binary(_) => {},
    		Message::Ping(_) => {},
    		Message::Pong(_) => {},
    		Message::Close(_close) => {
                println!("when this occurs we are going to close the connection for that socket");

            }
    	}


    }

// room creation handling completed
// adding participant handling completed
// getting started with the auction has been completed
// each bid handling has been completed,
// sending next player after the bid selling has been completed
    
}


async fn broadcast_message(
    connections: &AppState,
    message: Message,
    room_id: String,
) {
    // Acquire read lock on the shared websocket_connections map
    let guard = connections.websocket_connections.read().await;
	let room_connections = guard.get(&room_id);
    //println!("room_id was ->{}->", room_id) ;

    match room_connections {
        Some(participants) => {
            // Iterate over participants and send the message
            participants.iter().for_each(|participant| {
                // Potential panic here if channel is closed!
                println!("executing {:#?}", participant );
                if let Err(e) = participant.sender.send(message.clone()) { // here we are actually sending the data to the unbounderSender that will be recieved by the unboundedReciever
                    println!(
                        "❌ Failed to send message to participant {}: {:?}",
                        participant.participant_id, e
                    );
                }
            });
        },
        None => {
            println!("⚠️ No room_id '{}' exists in the websocket_connections map", room_id);
        }
    }
}



async fn create_room(room: Room) -> bool {
	true
}




async fn get_next_player(player_id: i32) -> Player {
    Player {
        player_name: String::from("guttikonda phanidhar reddy"),
        stats: 1,
        role: String::from("all-rounder"),
        age : 22,
        country : String::from("India"),
        capped : true,
        player_id: 1,
        base_price: 200, // in lakhs
    }
}



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