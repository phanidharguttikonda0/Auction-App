use std::{collections::HashSet, time::Duration};

use axum::{extract::{ws::{Message, WebSocket, WebSocketUpgrade}, Path, State}, response::IntoResponse, Error};
use futures::{StreamExt, SinkExt};
use redis::aio;
use serde::de::value;
use crate::{ middlewares::rooms_middleware::get_user_id, web_socket_models::{Bid, CreateConnection, LastBid, Player, Ready, Room, RoomConnection, Sell}, AppState};
use tokio::sync::{broadcast, mpsc::unbounded_channel};
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
pub async fn handle_ws_upgrade(ws: WebSocketUpgrade, State(connections): State<AppState>,Path((room_id,participant_id)): Path<(String,u32)>) -> impl IntoResponse {
	ws.on_upgrade(move |socket| handle_ws(socket, connections,room_id, participant_id))
}


// we are using mpsc(multiple producers and single consumers) where there are multiple producers but there is only a single task consume it (recieves it), where in that reciever only we are sending the data to the client
// it's like a mail box where many people can put letters in box (producer) and only one person opens it (reciever).
// let (tx, rx) = mpsc::channel(100); // Bounded channel with buffer of 100, only 100 it's the limit
// where unbounded channel means where there is no limit on how many messages it gonna store, no limit on the queue size
// redis allows atomicity
async fn handle_ws(mut socket: WebSocket, connections:AppState, room_id: String, participant_id: u32){ // for each new websocket connection this will be executed once if 10k web socket connections were created then 10k times it will be called
	// Equivalent of `socket.onopen`
    println!("📡 Client connected");
    // Create an unbounded channel to send messages to this client
    let (tx, mut rx) = unbounded_channel::<Message>() ; // for each websocket connection this will be created once
    let second_connection = connections.clone() ;
    // we split the websocket in to 2 parts because , we are sending data in a different task and recieving in a different task both are not in a single task
    let (mut sender, mut receiver) = socket.split() ; // for each websocket connection this will be created once
    tokio::spawn(async move { // we will send messages
        while let Some(msg) = rx.recv().await { // rx is an unbounded reciever where we actually send the data through it , so when recieves data we will send the data via the sender socket
            if tokio::time::timeout(Duration::from_secs(5), sender.send(msg)).await.is_err(){
                // we need to clean up the user , such that user need to join again
                if let Err(e) = sender.close().await {
                    println!("Error closing sender: {:?}", e);
                }
                let read_state = second_connection.websocket_connections.read().await ;
                let l = read_state.get(&room_id).unwrap() ;
                let mut index = 0 ;
                for participant in l{
                    if participant.participant_id == participant_id {
                        drop(read_state) ;
                        let mut state = second_connection.websocket_connections.write().await ;
                        let mut l:&mut Vec<Participant> = state.get_mut(&room_id).unwrap() ;
                        l.remove(index);
                        drop(state) ;
                        break;
                    }
                    index += 1 ;
                } // we are removing the connection of the user, telling him to rejoin again

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
                let mut redis_connection = connections.redis_connection.get_async_connection().await.unwrap() ;// return redis::aio::Connection

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
                    // we need to check whether the room-exists
                    let room = redis::cmd("GET").arg(&create.room_id).query_async::<aio::Connection,String>(&mut redis_connection).await ;
                    match room {
                        Ok(room) => {
                            println!("room exists already, so it's an invalid room-id") ;
                            broadcast_message(&connections, Message::Text(String::from("Invalid Room-Id")), create.room_id).await ;
                        },
                        Err(err) => {
                            println!("No room exists , so we can create the room") ;
                            // we need to store it in the redis
                            let serialized = serde_json::to_string(&Room::new(
                                create.participant_id as u32,
                            create.team_selected.clone(),
                            create.max_numbers as u32
                                )).unwrap() ;
                            redis::cmd("SET").arg(&create.room_id).arg(&serialized).query_async::<aio::Connection,()>(&mut redis_connection).await.unwrap() ;
            				//sending data to all the members in the room
            				broadcast_message(&connections, Message::Text(
            					serde_json::to_string::<CreateConnection>(&create).unwrap()
            					), create.room_id).await;
                        }
                    }


    			}else if let Ok(room_join) = room_join {
    				let participant = Participant {
    					participant_id: room_join.participant_id as u32,
    					sender: tx.clone()
    				};
                    // we need check whether the room is full or not , if not if the user already exists or not, if room_full no opportunity
                    // if room not full but user exists then we will just store the connection in the in memory instead of storing data again
                    // one should not have the same team name as well.
                    let results = redis::cmd("GET").arg(&room_join.room_id).query_async::<_, String>(&mut redis_connection).await.unwrap() ;

                    match serde_json::from_str::<Room>(&results) {
                        Ok(mut room) =>{
                            // room available

                                    let mut set:HashSet<i32> = HashSet::new() ; // adding user-ids
                                    let mut b = true ;

                                    for x in room.participants.values() {
                                        let value = get_user_id(x.0 as i32, &connections.database_connection).await ;
                                        if set.contains(&value ) {
                                            b = false ; break;
                                        }else{
                                            set.insert(value) ;
                                        }
                                    }

                                    // we are going to store this transaction to the connections
                                    let mut state = connections.websocket_connections.write().await;
                        	       			state.entry(room_join.room_id.clone()).or_default().push(participant);
                                               drop(state);
                                    if b && (room.participants.len() as u32) < room.max_players {
                                        // we need to check whether the team was already selected or not
                                        for x in room.participants.keys() {
                                            if room_join.team_selected == x.to_string(){
                                                b = false;
                                                break;
                                            }
                                        }

                                        if b {

                                            room.participants.insert(room_join.team_selected.clone(), (room_join.participant_id as u32, 12000));

                                            // we need to add it in the redis
                                            let serialized = serde_json::to_string(&room).unwrap() ;

                                            redis::cmd("SET").arg(&room_join.room_id).arg(&serialized).query_async::<_,()>(&mut redis_connection).await.unwrap() ;

                                            //sending data to all the members in the room, when a new member joined or old member has been re-joined
                            				broadcast_message(&connections, Message::Text(
                            					serde_json::to_string::<RoomConnection>(&room_join).unwrap()
                            				), room_join.room_id).await;

                                        }else{
                                            //sending data to all the members in the room, when a new member joined or old member has been re-joined
                            				broadcast_message(&connections, Message::Text(String::from("Select a different team")), room_join.room_id).await;
                                        }

                                    }else{
                                        //sending data to all the members in the room, when a new member joined or old member has been re-joined
                            				broadcast_message(&connections, Message::Text(
                            					serde_json::to_string::<RoomConnection>(&room_join).unwrap()
                            				), room_join.room_id).await;
                                    }


                        },
                        Err(err) => {
                            println!("Room doesn't exists {}", err) ;
                        }
                    }

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

        // So, if multiple parts of your app want to send data to the same client, they can’t all just call .send() directly on the WebSocket — it would cause race conditions or borrowing issues.


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