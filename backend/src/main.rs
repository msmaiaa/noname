use dotenv::dotenv;
use salvo::{extra::ws::WebSocketUpgrade, prelude::*};

use crate::middleware::{with_admin, with_jwt};

pub mod driver;
pub mod error;
pub mod global;
pub mod middleware;
pub mod model;
pub mod routes;
pub mod service;
pub mod ws;

#[handler]
async fn on_ws_connection(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), StatusError> {
    match req.uri().to_string().as_str() {
        "/ws/server" => {
            let data = ws::server::authorize_server_connection(req, res).await?;
            WebSocketUpgrade::new()
                .upgrade(req, res, move |ws| async move {
                    ws::server::handle_server_connection(ws, &data).await
                })
                .await?;
        }
        "/ws/user" => {
            let data = ws::user::authorize_user_connection(req, depot, res).await?;
            WebSocketUpgrade::new()
                .upgrade(req, res, move |ws| async move {
                    ws::user::handle_user_connection(ws, &data).await
                })
                .await?;
        }
        _ => {
            return Err(StatusError::forbidden());
        }
    };

    Ok(())
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

    let router = Router::new()
        .push(Router::with_path("/api/auth/login").get(routes::auth::login))
        .push(Router::with_path("/api/auth/steam_callback").get(routes::auth::steam_callback))
        .push(
            Router::with_path("/api/servers")
                .hoop(with_admin)
                .post(routes::server::create_server),
        )
        .push(Router::with_path("/ws/server").handle(on_ws_connection))
        .push(
            Router::with_path("/ws/user")
                .hoop(with_jwt)
                .handle(on_ws_connection),
        );

    tracing::info!("Server started at http://{}/", listen_addr);

    Server::new(TcpListener::bind(&listen_addr))
        .serve(router)
        .await;
}
