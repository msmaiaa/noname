use dotenv::dotenv;
use salvo::{
    extra::{
        jwt_auth::{JwtAuth, JwtAuthDepotExt, JwtTokenExtractor},
        ws::WebSocketUpgrade,
    },
    prelude::*,
};

pub mod driver;
pub mod error;
pub mod global;
pub mod middleware;
pub mod model;
pub mod routes;
pub mod service;
pub mod ws;

#[handler]
async fn on_ws_connection(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    match req.uri().to_string().as_str() {
        "/ws/server" => {
            let data = ws::server::authorize_server_connection(req, res).await?;
            WebSocketUpgrade::new()
                .upgrade(req, res, move |ws| async move {
                    ws::server::handle_server_connection(ws, &data).await
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

struct JwtExtractor;

#[async_trait]
impl JwtTokenExtractor for JwtExtractor {
    async fn token(&self, req: &mut Request) -> Option<String> {
        req.headers()
            .get("Authorization")
            .map(|v| v.to_str().unwrap().replace("Bearer ", ""))
    }
}

#[handler]
async fn with_admin(
    _: &mut Request,
    depot: &mut Depot,
    _: &mut Response,
) -> Result<(), StatusError> {
    let token_data = depot
        .jwt_auth_data::<service::auth::TokenData>()
        .ok_or(StatusError::unauthorized())?;
    let user = model::user::select_by_steamid(
        &mut global::RB.clone(),
        token_data.claims.steamid64.clone(),
    )
    .await
    .map_err(|_| StatusError::unauthorized())?;
    match user {
        Some(user) => {
            if user.is_admin {
                Ok(())
            } else {
                Err(StatusError::unauthorized())
            }
        }
        None => Err(StatusError::unauthorized()),
    }
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

    let auth_handler: JwtAuth<service::auth::TokenData> = JwtAuth::new(global::JWT_KEY.to_string())
        .with_extractors(vec![Box::new(JwtExtractor {})])
        .with_response_error(true);

    let router = Router::new()
        .push(Router::with_path("/api/auth/login").get(routes::auth::login))
        .push(Router::with_path("/api/auth/steam_callback").get(routes::auth::steam_callback))
        .push(
            Router::with_path("/api/servers")
                .hoop(auth_handler)
                .hoop(with_admin)
                .get(routes::server::get_servers)
                .post(routes::server::create_server),
        )
        .push(Router::with_path("/ws/server").handle(on_ws_connection))
        .push(Router::with_path("/ws/admin").handle(on_ws_connection));

    tracing::info!("Server started at http://{}/", listen_addr);

    Server::new(TcpListener::bind(&listen_addr))
        .serve(router)
        .await;
}
