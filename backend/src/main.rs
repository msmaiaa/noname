use dotenv::dotenv;
use salvo::{extra::ws::WebSocketUpgrade, prelude::*};

pub mod auth;
pub mod driver;
pub mod global;
pub mod model;
pub mod routes;
pub mod ws;

#[handler]
async fn on_ws_connection(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    match req.uri().to_string().as_str() {
        "/ws/server" => {
            let data = ws::server::authorize_server_connection(req, res)?;
            WebSocketUpgrade::new()
                .upgrade(req, res, move |ws| async move {
                    ws::server::handle_server_connection(ws, data).await
                })
                .await?;
        }
        "/ws/admin" => {
            let data = ws::admin::authorize_admin_connection(req, res)?;
            WebSocketUpgrade::new()
                .upgrade(req, res, move |ws| async move {
                    ws::admin::handle_admin_connection(ws, data).await
                })
                .await?;
        }
        "/ws/client" => {}
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

    let host = std::env::var("HOST").unwrap_or("0.0.0.0".to_string());
    let port = std::env::var("PORT").unwrap_or("1337".to_string());
    let addr = format!("{}:{}", host, port);

    let router = Router::new()
        .push(Router::with_path("/auth/login").get(routes::auth::login))
        .push(Router::with_path("/auth/steam_callback").get(routes::auth::steam_callback))
        .push(Router::with_path("/servers").get(routes::server::get_servers))
        .push(Router::with_path("ws/server").handle(on_ws_connection))
        .push(Router::with_path("ws/admin").handle(on_ws_connection));

    tracing::info!("Server started at http://{}/", addr);

    Server::new(TcpListener::bind(format!("{}:{}", host, port).as_str()))
        .serve(router)
        .await;
}
