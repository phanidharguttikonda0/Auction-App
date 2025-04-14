use serde::{Deserialize, Serialize};
use validator::{ Validate};

use crate::middlewares::authentication_middleware::validate_username ;

#[derive(sqlx::FromRow,Serialize, Deserialize, Debug, Validate)]
pub struct Authentication {
	#[validate(email)]
	pub mail_id: String,
	#[validate(length(min=8, message="min 8 characters should contain"))]
	pub password: String
}


#[derive(sqlx::FromRow, Serialize, Deserialize, Debug, Validate)]
pub struct SignUp{
	#[validate(custom(function = "validate_username"))]
	pub username: String,
	#[validate(email)]
	pub mail_id: String,
	#[validate(length(min=8, message="min 8 characters should contain"))]
	pub password: String,
}



// where validator helps in validating these with out need to middlewares
// so there is no need of middlewares for input validation 

#[derive(Serialize)]
pub struct  Authorization{
	header: String,
}

impl Authorization {
	pub fn new(header: String) -> Authorization {
		Authorization { header }
	}
}

#[derive(Deserialize,Serialize)]
pub struct Claims {
	pub mail_id: String,
}

impl Claims {
	pub fn new(mail_id: String) -> Claims {
		Claims { mail_id }
	}
}