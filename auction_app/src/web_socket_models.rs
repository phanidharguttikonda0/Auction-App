
use uuid::Uuid ;


#[derive(Debug)]
pub struct CreateConnection{ // for creating a new connection
	pub room_id: Uuid,
	pub participant_id: i32,
	pub max_numbers: i32
}

// Room Creation struct for adding a participant