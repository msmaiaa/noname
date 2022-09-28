use salvo::{extra::ws::WebSocketUpgrade, prelude::*};
pub mod global;
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
        "/ws/admin" => {}
        "/ws/client" => {}
        _ => {
            return Err(StatusError::forbidden());
        }
    };

    Ok(())
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let router = Router::new()
        .push(Router::with_path("/servers").get(routes::server::get_servers))
        .push(Router::with_path("ws/server").handle(on_ws_connection))
        .push(Router::with_path("ws/admin").handle(on_ws_connection));
    tracing::info!("Listening at http://127.0.0.1:1337");
    Server::new(TcpListener::bind("0.0.0.0:1337"))
        .serve(router)
        .await;
}
