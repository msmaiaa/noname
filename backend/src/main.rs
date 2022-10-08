use std::net::SocketAddr;

use crate::middleware::with_admin;
use axum::{
    body::{self, Body},
    extract::{ws::WebSocketUpgrade, ConnectInfo},
    http::{header, Request},
    response::IntoResponse,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use error::AppError;
use service::auth::TokenData;
use tower::ServiceBuilder;
use tower_http::{
    cors::{Any, CorsLayer},
    ServiceBuilderExt,
};

//use crate::middleware::{with_admin, with_jwt};

pub mod driver;
pub mod error;
pub mod extractor;
pub mod global;
pub mod middleware;
pub mod model;
pub mod response;
pub mod routes;
pub mod service;
pub mod ws;

async fn on_server_connection(
    ws: WebSocketUpgrade,
    req: Request<Body>,
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
) -> Result<impl IntoResponse, AppError> {
    let data = ws::server::authorize_server_connection(req, addr).await?;
    Ok(ws.on_upgrade(move |ws| ws::server::handle_server_connection(ws, data)))
}
async fn on_user_connection(
    token_data: TokenData,
    ws: WebSocketUpgrade,
    req: Request<Body>,
) -> Result<impl IntoResponse, AppError> {
    let data = ws::user::authorize_user_connection(req, token_data).await?;
    Ok(ws.on_upgrade(move |ws| ws::user::handle_user_connection(ws, data)))
}

#[tokio::main]
async fn main() {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    global::RB
        .init(rbdc_pg::driver::PgDriver {}, global::DATABASE_URL.as_str())
        .expect("Failed to initialize database connection");
    driver::db::migrate(driver::db::DbKind::Postgres).await;

    let listen_addr = format!("0.0.0.0:{}", *global::PORT);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE]);

    let router = Router::new()
        .route("/api/auth/login", get(routes::auth::login))
        .route(
            "/api/auth/steam_callback",
            get(routes::auth::steam_callback),
        )
        .route(
            "/api/servers",
            post(routes::server::create_server).route_layer(axum::middleware::from_fn(with_admin)),
        )
        .route("/ws/server", get(on_server_connection))
        .route("/ws/user", get(on_user_connection))
        .layer(cors);

    tracing::info!("Server started at http://{}/", listen_addr);

    axum::Server::bind(&SocketAddr::from((
        [0, 0, 0, 0],
        global::PORT.parse::<u16>().unwrap(),
    )))
    .serve(router.into_make_service())
    .await
    .unwrap();
}
