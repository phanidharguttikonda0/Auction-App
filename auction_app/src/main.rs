use axum::{
    middleware,
    routing::{get, post},
    Router,
};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};
mod handlers;
use handlers::{authentication_handlers::*, rooms_handler::*, players_handlers::*};
mod middlewares;
mod models;
use middlewares::authentication_middleware::*;
use middlewares::rooms_middleware::{active_room_checks, room_id_check};
async fn home() -> String {
    String::from("true")
}

#[derive(Clone)]
struct AppState {
    database_connection: Pool<Postgres>, // where it internally uses Arc so no need to wrapping it inside Arc
}

impl AppState {
    fn new(database_connection: Pool<Postgres>) -> AppState {
        AppState {
            database_connection,
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
        )).route("/:stats_id", get(get_player).layer(middleware::from_fn(authorization_check)))
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
    let state = AppState::new(database_connection);
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
