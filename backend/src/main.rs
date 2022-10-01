use dotenv::dotenv;
use salvo::{
    extra::{
        jwt_auth::{JwtAuth, JwtAuthDepotExt, JwtTokenExtractor},
        ws::WebSocketUpgrade,
    },
    prelude::*,
};
use service::auth::extract_data_from_depot;

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

#[handler]
async fn with_admin(
    _: &mut Request,
    depot: &mut Depot,
    _: &mut Response,
) -> Result<(), StatusError> {
    let token_data = extract_data_from_depot(depot).ok_or(StatusError::unauthorized())?;
    let user =
        model::user::select_by_steamid(&mut global::RB.clone(), token_data.steamid64.clone())
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

#[handler]
fn with_jwt(req: &mut Request, depot: &mut Depot, res: &mut Response) -> Result<(), StatusError> {
    let jwt_token = req
        .headers()
        .get("Authorization")
        .map(|v| v.to_str().unwrap().replace("Bearer ", ""))
        .ok_or(StatusError::unauthorized())?;
    service::auth::decode_token(&jwt_token)
        .map(|token_data| {
            depot.insert("token_data", token_data);
        })
        .map_err(|e| {
            res.render(e.to_string());
            StatusError::unauthorized()
        })?;
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
                .hoop(with_jwt)
                .hoop(with_admin)
                .get(routes::server::get_servers)
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
