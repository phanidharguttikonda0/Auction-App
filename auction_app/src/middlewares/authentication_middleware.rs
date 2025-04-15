use axum::{ extract::Request, http::StatusCode, middleware::Next, response::Response};

use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation,Algorithm} ;
use validator::ValidationError;

use crate::models::authentication_models::Claims;

pub fn authorization_header_check(header: &str) -> (bool, Claims) {
	println!("header came for checking {}", header) ;
	let mut validation = Validation::new(Algorithm::HS256);
    validation.validate_exp = false;
    validation.required_spec_claims.remove("exp");
	let result = decode::<Claims>(header, &DecodingKey::from_secret("secret".as_ref()), &validation);
	match result {
	    Ok(val) => {
	    	println!("success");

	    	(true,val.claims)
	    },
	    Err(err) => {
	    	println!("{}", err);
	    	println!("failed");
	    	(false, Claims::new(String::from(""), -1))
	    },
	}
}

pub fn validate_username(username: &str) -> Result<(), ValidationError> {

	if username.len() > 3 {
		return Err(ValidationError::new("Minimum of length 4 is required")) 
	}
	if !username.chars().all(|c| c.is_alphanumeric() || c == '_'){
		return Err(ValidationError::new("only contain alpha numeric characters only"))
	}
	Ok(())
}

pub fn generate_authorization_header(username: String, user_id: i32) -> String {
	encode(&Header::default(), &Claims::new(username,user_id), &EncodingKey::from_secret("secret".as_ref())).unwrap()
}


pub async fn authorization_check(mut req: Request, next: Next) -> Result<Response, StatusCode>{
	
	let header = req.headers().get("authorization").unwrap().to_str().unwrap() ;
	let val = authorization_header_check(header);
	if val.0{
		req.extensions_mut().insert(val.1);
		Ok(next.run(req).await)
	}else{
		Err(StatusCode::UNAUTHORIZED)
	}
	

}

//* why we need to reconstruct a new request, when we are working with body of an request
/*
as req.into_parts() returns the values , so the parts and body are moved out from req, 
so we need to construct a new request with the same parts and the body.
*/