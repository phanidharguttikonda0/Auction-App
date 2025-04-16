use crate::{
    models::{
        authentication_models::Claims,
        room_models::{
            Accessability, CreateRoom, GetTeams, JoinRoomPath, Participant, PublicRoom, PublicRoomsReturn, RoomCreation, RoomState, Roomid, Teams
        },
    },
    AppState,
};
use axum::{
    extract::{Path, State},
    response::IntoResponse,
    Extension, Json,
};
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

pub async fn create_room(
    Extension(user): Extension<Claims>,
    State(state): State<AppState>,
    Json(room): Json<CreateRoom>,
) -> Json<RoomCreation> {
    // creation of middle-ware, where there should be no ongoing or starting status for user
    // who wants to create and who wants to join

    let mut tx = state.database_connection.begin().await.unwrap();
    let result = sqlx::query_as::<_,Roomid>("insert into rooms (owner_id,accessibility,room_status) values ($1,$2,$3) RETURNING room_id")
	.bind(user.userId)
	.bind(match room.accessibility {
		Accessability::PUBLIC => "public",
		Accessability::PRIVATE => "private"
	})
	.bind(match room.state {
		RoomState::ONGOING => "ongoing" ,
		RoomState::STARTING => "waiting",
		RoomState::COMPLETED => "completed" 
	})
	.fetch_one(&mut *tx).await ;

    match result {
        Ok(res) => {
            println!("result was {:#?}", res);
            let participant_id = add_participant(tx, user.userId, res.room_id, room.team).await;

            Json::from(RoomCreation {
                room_id: res.room_id,
                participant_id: participant_id,
            })
            // here we need to add the user as the participant as well
        }
        Err(err) => {
            println!("Error was {}", err);
            Json::from(RoomCreation {
                room_id: Uuid::nil(),
                participant_id: -1,
            })
            // returns 00000000-0000-0000-0000-000000000000
        }
    }
} // we can allow both public and private rooms to join with the code,
  // but benfit of public room was , some other x-person can also join the room

pub async fn add_participant(
    mut tx: Transaction<'static, Postgres>,
    user_id: i32,
    room_id: Uuid,
    team: Teams,
) -> i32 {
    let result = sqlx::query_as::<_, Participant>(
        "insert into participants (team_selected,user_id,room_id) values ($1,$2,$3)
		RETURNING participant_id
		",
    )
    .bind(team_match(team))
    .bind(user_id)
    .bind(room_id)
    .fetch_one(&mut *tx)
    .await;

    match result {
        Ok(res) => {
            println!("The Participant was {:#?}", res);
            tx.commit().await.unwrap();
            res.participant_id
        }
        Err(err) => {
            println!("Error was {}", err);
            tx.rollback().await.unwrap();
            0
        }
    }
}

pub async fn get_public_rooms(State(state): State<AppState>) -> Json<PublicRoomsReturn> {
    // we are going to return, usernames, selected-teams,room-id
    //only of waiting stage
    let result = sqlx::query_as::<_, PublicRoom>(
        "SELECT 
	    u.username,
	    p.team_selected,
	    r.room_id
		FROM 
		    participants p
		JOIN 
		    users u ON p.user_id = u.id
		JOIN 
		    rooms r ON p.room_id = r.room_id
		WHERE 
		    r.room_status = 'waiting';
		",
    )
    .fetch_all(&state.database_connection)
    .await;

    match result {
        Ok(res) => {
            println!("the first public room was {:#?}", res[0]);
            Json::from(PublicRoomsReturn { public_rooms: res })
        }
        Err(err) => {
            println!("Error in get public rooms were {}", err);
            Json::from(PublicRoomsReturn {
                public_rooms: vec![],
            })
        }
    }
}

pub fn team_match(team: Teams) -> String {
match team {
        Teams::MumbaiIndians => String::from("MumbaiIndians"),
        Teams::SunrisersHyderabad => String::from("SunrisersHyderabad"),
        Teams::PunjabKings => String::from("PunjabKings"),
        Teams::DelhiCapitals => String::from("DelhiCapitals"),
        Teams::ChennaiSuperKings => String::from("ChennaiSuperKings"),
        Teams::RajastanRoyals => String::from("RajastanRoyals"),
        Teams::GujaratTitans => String::from("GujaratTitans"),
        Teams::RoyalChallengesBengaluru => String::from("RoyalChallengesBengaluru"),
        Teams::KolkataKingKnightRiders => String::from("KolkataKingKnightRiders"),
        Teams::LucknowSuperGaints => String::from("LucknowSuperGaints"),
    }
}

// fails when room-id was invalid and not active in middleware stage itself
// fails if selected team id was set and joining again in the room
pub async fn join_room(
    Extension(user): Extension<Claims>,
    State(state): State<AppState>,
    Path(room): Path<JoinRoomPath>,
) -> impl IntoResponse {
    // room-joining
    println!("{:#?}", room);
    let participant_id = add_participant(
        state.database_connection.begin().await.unwrap(),
        user.userId,
        Uuid::parse_str(&room.room_id).unwrap(),
        room.team_name,
    )
    .await;
    participant_id.to_string()
}

pub async fn get_teams(State(state): State<AppState>, Path(room):Path<JoinRoomPath>) -> Json<Vec<GetTeams>>{
    // from the given room-id and the team selected we are going to 
    // get the  players bids

    let result = sqlx::query_as::<_,GetTeams>("select b.amount,p.player_name,p.role from bids b 
        JOIN players p ON p.player_id = b.player_id JOIN participants pt ON 
        pt.participant_id = b.participant_id where b.room_id = ($1) AND pt.team_selected = ($2)
        ").bind(room.room_id).bind(team_match(room.team_name))
        .fetch_all(&state.database_connection).await;
    match result {
        Ok(res) => {
            println!("successfully fetched result");
            Json(res)
        },
        Err(err) => {
            println!("Error was {}",err) ;
            Json(vec![])
        }
    }
}
