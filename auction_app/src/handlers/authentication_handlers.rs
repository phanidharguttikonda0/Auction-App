
use axum::extract::State;
use axum::{extract::Path,  Form, Json};


use crate::models::authentication_models::{Authentication, Authorization, Details, SignUp};
use crate::middlewares::authentication_middleware::generate_authorization_header ;
use crate::AppState;

// login taking 6.2ms of time to serve a request
pub async fn login(State(state): State<AppState>, Form(login_data): Form<Authentication>) -> Json<Result<Authorization, String>> {
	println!("login details : {:#?}", login_data) ;
	let database_connection = state.database_connection ;
	let result = sqlx::query_as::<_, Details>("Select username,id from users where username=($1) and password=($2)")
	.bind(&login_data.username)
	.bind(&login_data.password)
	.fetch_one(&database_connection).await ;
	match result {
		Ok(res) => {
			println!("Result for login was {:#?}", res) ;
			let authorization_header = generate_authorization_header(res.username, res.id) ;
			Json::from(Ok(Authorization::new(authorization_header)))
		},
		Err(err) => {
			println!("Error while login was {}", err) ;
			Json::from(Err(String::from("Error Occured due to no row found")))
		}
	}
	
}

pub async fn sign_up(State(state): State<AppState>,Form(new_user): Form<SignUp>) -> Json<Result<Authorization, String>> {
	println!("new_user : {:#?}", new_user) ;
	let database_connection = state.database_connection ;
	let result = sqlx::query_as::<_,Details>("insert into users (username,mail_id, password) values ($1,$2,$3)")
	.bind(&new_user.username)
	.bind(&new_user.mail_id)
	.bind(&new_user.password)
	.fetch_one(&database_connection).await;
	match result {
	    Ok(res) => {
	    	println!("The result was {:#?}", res) ;
	    	let authorization_header = generate_authorization_header(res.username,res.id) ;
			Json::from(Ok(Authorization::new(authorization_header)))
	    },
	    Err(err) => {
	    	println!("error occured : {}", err);
	    	Json::from(Err(String::from("User already exists with mail_id")))
	    }
	}
	
}

pub async fn forgot_credentials(State(state): State<AppState>,Path(mail_id): Path<String>) -> Json<String> {
	println!("mail_id was : {:#?}", mail_id) ;
	Json::from(String::from("true"))
}