use std::net::SocketAddr;

use axum::{
    http::header,
    routing::{get, post},
    Router,
};
use dotenv::dotenv;
use noname::{global, middleware::with_admin, routes, ws};
use tower_http::cors::{Any, CorsLayer};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    tracing_subscriber::fmt::init();

    noname::driver::db::init_and_migrate().await;

    let listen_addr = format!("0.0.0.0:{}", *global::PORT);

    let ws_router = Router::new()
        .route(
            "/user",
            get(ws::user::on_user_connection).route_layer(axum::middleware::from_fn(with_admin)),
        )
        .route("/server", get(ws::server::on_server_connection));

    let auth_router = Router::new()
        .route("/login", get(routes::auth::login))
        .route("/steam_callback", get(routes::auth::steam_callback));

    let server_router = Router::new().route(
        "/",
        post(routes::server::create_server).route_layer(axum::middleware::from_fn(with_admin)),
    );

    let router = Router::new()
        .nest(
            "/api",
            Router::new()
                .nest("/auth", auth_router)
                .nest("/servers", server_router),
        )
        .nest("/ws", ws_router)
        .layer(build_cors());

    tracing::info!("Server started at http://{}/", listen_addr);

    start_server(router).await?;
    Ok(())
}

async fn start_server(router: Router) -> anyhow::Result<()> {
    axum::Server::bind(&SocketAddr::from((
        [0, 0, 0, 0],
        global::PORT.parse::<u16>().unwrap(),
    )))
    .serve(router.into_make_service_with_connect_info::<SocketAddr>())
    .await?;

    Ok(())
}

fn build_cors() -> CorsLayer {
    CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(vec![header::AUTHORIZATION, header::CONTENT_TYPE])
}
