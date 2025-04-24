use sqlx::{Pool, Postgres};
use uuid::Uuid;



pub async fn player_sold(room_id: String, player_id: u32, participant_id: u32, amount:u32, connection: &Pool<Postgres>) -> bool {
    // we need to convert room_id to Uuid
    let result = sqlx::query("insert into bids (room_id,player_id,participant_id,amount) values ($1,$2,$3,$4) ")
    .bind(Uuid::parse_str(&room_id).unwrap()).bind(player_id as i32).bind(participant_id as i32).bind(amount as i32).execute(connection).await ;

    match result {
        Ok(result) => {
            if result.rows_affected() > 0{
                println!("Bid Successfully inserted") ;
                true
            }else{
                println!("Bid was not inserted successfully") ;
                false
            }
        },
        Err(err) => {
            println!("Error Occured") ;
            false
        }
    }
}
