use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
mod handlers;
use handlers::{authentication_handlers::*, rooms_handler::*, players_handlers::*, other_handlers::*};
mod middlewares;
mod models;
use middlewares::authentication_middleware::*;
use middlewares::rooms_middleware::{active_room_checks, room_id_check};
mod web_socket;
use web_socket::*;
mod web_socket_models;
use std::sync::{Arc};
use tokio::sync::RwLock;
use std::collections::HashMap;
use web_socket_models::Participant ;

async fn home() -> String {
    String::from("true")
}

#[derive(Clone)]
struct AppState {
    database_connection: Pool<Postgres>, // where it internally uses Arc so no need to wrapping it inside Arc
    redis_connection: redis::Client,
    websocket_connections: Arc<RwLock<HashMap<String, Vec<Participant>>>>,
}

impl AppState {
    fn new(database_connection: Pool<Postgres>, redis_connection: redis::Client, websocket_connections:Arc<RwLock<HashMap<String, Vec<Participant>>>>) -> AppState {
        AppState {
            database_connection,
            redis_connection,
            websocket_connections
        }
    }
}

fn rooms_router(state: AppState) -> Router<AppState> {
    Router::new()
        .route(
            "/create",
            post(create_room)
                .layer(middleware::from_fn(authorization_check))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    active_room_checks,
                )),
        )
        .route(
            "/",
            get(get_public_rooms).layer(middleware::from_fn(authorization_check)),
        )
        .route(
            "/join-room/:user_id/:room_id/:team_name",
            get(join_room)
                .layer(middleware::from_fn(authorization_check))
                .layer(middleware::from_fn_with_state(
                    state.clone(),
                    active_room_checks,
                ))
                .layer(middleware::from_fn_with_state(state.clone(), room_id_check)),
        )
        .route("/teams/:room_id/:team_name", get(get_teams).layer(
            middleware::from_fn(authorization_check)
            ))
}

fn players_routes() -> Router<AppState> {
    Router::new().route("/:pool_id", get(get_players).layer(
        middleware::from_fn(authorization_check)
        )).route("/player-stats/:stats_id", get(get_player).layer(middleware::from_fn(authorization_check)))
        .route("/unsold/:room_id", get(get_unsold_players)
            .layer(middleware::from_fn(authorization_check))
            )
}



#[tokio::main(flavor = "multi_thread", worker_threads = 8)] // it uses all the cores in system
async fn main() {
    // at max only 100 connections only psql opens at time , if more requests accessing them
    // then they need to wait for any other connections to be completed.
    let database_connection = PgPoolOptions::new()
        .max_connections(100) // max we specified only those many connections will open
        .min_connections(2) // min no of connections pool keeps alive even when idle
        .connect("postgresql://postgres:phani@localhost:5432/auction")
        .await
        .unwrap();
        let redis_client = redis::Client::open("redis://127.0.0.1:6379").unwrap() ;
        let websocket_connections = Arc::new(RwLock::new(HashMap::new()));
    let state = AppState::new(database_connection, redis_client,websocket_connections);
    let auth_router = Router::new()
        .route("/login", post(login))
        .route("/sign-up", post(sign_up))
        .route("/forgot-credentials/:mail_id", post(forgot_credentials));
    let app = Router::new()
        .nest("/authentication", auth_router)
        .route(
            "/home",
            get(home).layer(middleware::from_fn(authorization_check)),
        )
        .nest("/rooms", rooms_router(state.clone()))
        .nest("/players", players_routes())
        .route("/search/:username", get(get_username).layer(middleware::from_fn(authorization_check)))
        .route("/profile/:username", get(get_profile).layer(middleware::from_fn(authorization_check)))
        .route("/profile/:auction_id/:username", get(get_auction).layer(middleware::from_fn(authorization_check)))
        .route("/", get(handle_ws_upgrade))
        .with_state(state); // state must be specified at last
                            // here we are creating the tcp connection
    let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:9090")
        .await
        .unwrap();
    println!("server running on the port {:#?}", tcp_listener);
    println!(
        "Available Cores are {}",
        std::thread::available_parallelism().unwrap().get()
    );
    axum::serve(tcp_listener, app).await.unwrap();
}

/*
where using all threads doesn't benfit here any though because, any way all are
Io operations, since the databases connections are limited , there will not a advantage
of using the multiple threads for these io operations, there is no big difference, but
if the data in the database were more than 10k rows or 20k rows then there will be a difference
in using multiple threads, but any way the task will be blocked and another task will be taken
by the tokio main thread and until finds a value it goes for another task, so even this case
as well not been that much usefull.
*/
