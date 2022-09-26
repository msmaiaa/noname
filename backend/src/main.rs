use futures_util::{FutureExt, StreamExt};
use once_cell::sync::Lazy;
use salvo::extra::ws::{Message, WebSocket, WebSocketUpgrade};
use salvo::prelude::*;
use std::collections::HashMap;
use tokio_stream::wrappers::UnboundedReceiverStream;

use tokio::sync::{mpsc, RwLock};

type Servers = RwLock<HashMap<String, mpsc::UnboundedSender<Result<Message, salvo::Error>>>>;

static ONLINE_SERVERS: Lazy<Servers> = Lazy::new(Servers::default);

#[handler]
async fn hello_world() -> &'static str {
    "Hello, world!"
}

const AUTHORIZED_SERVERS: [&str; 1] = ["192.168.0.13"];

fn check_server_authorization(req: &mut Request) -> Result<String, StatusError> {
    let socket_ip = req
        .remote_addr()
        .ok_or(StatusError::unauthorized())?
        .as_ipv4()
        .ok_or(StatusError::unauthorized())?
        .ip()
        .to_string();
    if !AUTHORIZED_SERVERS.contains(&socket_ip.as_str()) {
        tracing::warn!("Unauthorized server tried to connect: {}", socket_ip);
        return Err(StatusError::unauthorized());
    }
    tracing::info!("Authorized server connected: {}", socket_ip);
    Ok(socket_ip)
}

#[handler]
async fn server_connected(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    tracing::info!("A server is trying to connect");
    let socket_ip = check_server_authorization(req)?;
    WebSocketUpgrade::new()
        .upgrade(req, res, |ws| async move {
            handle_server_connection(ws, socket_ip).await
        })
        .await
}

async fn handle_server_connection(ws: WebSocket, ip: String) {
    //adds the server to the list
    let (server_ws_tx, mut server_ws_rx) = ws.split();

    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    let fut = rx.forward(server_ws_tx).map(|result| {
        if let Err(e) = result {
            tracing::error!(error = ?e, "websocket send error");
        }
    });
    tokio::task::spawn(fut);
    let fut = async move {
        ONLINE_SERVERS.write().await.insert(ip.clone(), tx);

        while let Some(result) = server_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("websocket server error(uid={}): {}", ip, e);
                    break;
                }
            };
            on_server_message(ip.clone(), msg).await;
        }

        on_server_disconnected(ip).await;
    };
    tokio::task::spawn(fut);
}

async fn on_server_message(ip: String, msg: Message) {
    tracing::info!("Server {} sent a message: {:?}", ip, msg);
}

async fn on_server_disconnected(ip: String) {
    tracing::info!("Server {} disconnected", ip);
    ONLINE_SERVERS.write().await.remove(&ip);
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let router = Router::new()
        .get(hello_world)
        .push(Router::with_path("ws/server").handle(server_connected));
    tracing::info!("Listening at http://127.0.0.1:1337");
    Server::new(TcpListener::bind("0.0.0.0:1337"))
        .serve(router)
        .await;
}
