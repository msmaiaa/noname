use std::net::SocketAddrV4;

use futures_util::{FutureExt, StreamExt};

use salvo::extra::ws::{Message, WebSocket, WebSocketUpgrade};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tokio_stream::wrappers::UnboundedReceiverStream;

use tokio::sync::{mpsc, RwLock};

use crate::global::ONLINE_SERVERS;

pub type ServerList = RwLock<Vec<Server>>;

#[derive(Serialize, Deserialize, Clone, Copy)]
enum ServerStatus {
    Idle,
    WaitingForPlayers,
    Starting,
    KnifeRound,
    Live,
    Ending,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Server {
    ip: String,
    port: String,
    status: ServerStatus,
    #[serde(skip, default = "default_conn")]
    conn: mpsc::UnboundedSender<Result<Message, salvo::Error>>,
}

#[derive(Deserialize)]
enum ServerMessageData {
    #[serde(rename = "status")]
    Status(ServerStatus),
}

fn default_conn() -> mpsc::UnboundedSender<Result<Message, salvo::Error>> {
    let (tx, _) = mpsc::unbounded_channel();
    tx
}

pub async fn get_online_servers() -> Vec<Server> {
    ONLINE_SERVERS.read().await.to_vec()
}

const AUTHORIZED_SERVERS: [&str; 1] = ["192.168.0.13"];

fn check_server_authorization(req: &mut Request) -> Result<SocketAddrV4, StatusError> {
    let socket_info = req
        .remote_addr()
        .ok_or(StatusError::unauthorized())?
        .as_ipv4()
        .ok_or(StatusError::unauthorized())?;
    if !AUTHORIZED_SERVERS.contains(&socket_info.ip().to_string().as_str()) {
        tracing::warn!(
            "Unauthorized server tried to connect: {}",
            socket_info.ip().to_string()
        );
        return Err(StatusError::unauthorized());
    }
    tracing::info!(
        "Authorized server connected: {}",
        socket_info.ip().to_string()
    );
    Ok(socket_info.clone())
}

#[handler]
pub async fn on_server_connection(
    req: &mut Request,
    res: &mut Response,
) -> Result<(), StatusError> {
    tracing::info!("A server is trying to connect");
    let socket_info = check_server_authorization(req)?;
    WebSocketUpgrade::new()
        .upgrade(req, res, move |ws| async move {
            handle_server_connection(ws, socket_info).await
        })
        .await
}

async fn handle_server_connection(ws: WebSocket, socket_info: SocketAddrV4) {
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
        ONLINE_SERVERS.write().await.push(Server {
            ip: socket_info.ip().to_string(),
            port: socket_info.port().to_string(),
            status: ServerStatus::Idle,
            conn: tx,
        });

        while let Some(result) = server_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!("websocket server error(uid={}): {}", socket_info.ip(), e);
                    break;
                }
            };
            on_server_message(socket_info, msg)
                .await
                .map_err(|e| {
                    tracing::error!(error = ?e, "Error while handling server message");
                })
                .ok();
        }

        on_server_disconnected(socket_info).await;
    };
    tokio::task::spawn(fut);
}

#[derive(Deserialize)]
enum ServerAction {
    #[serde(rename = "server_2_backend_update_status")]
    Server2BackendUpdateStatus,
}
#[derive(Deserialize)]
struct ServerMessage {
    action: ServerAction,
    data: ServerMessageData,
}

async fn on_server_message(server_data: SocketAddrV4, msg: Message) -> anyhow::Result<()> {
    if !msg.is_text() {
        return Ok(());
    }
    let msg = msg.to_str()?;
    let parsed_msg: ServerMessage = serde_json::from_str(msg)?;

    match parsed_msg.action {
        ServerAction::Server2BackendUpdateStatus => {
            let status = match parsed_msg.data {
                ServerMessageData::Status(status) => status,
            };
            for server in ONLINE_SERVERS.write().await.iter_mut() {
                if server.ip == server_data.ip().to_string() {
                    server.status = status;
                }
            }
        }
    }
    Ok(())
}

async fn on_server_disconnected(server_data: SocketAddrV4) {
    tracing::info!("Server {} disconnected", server_data.ip().to_string());
    ONLINE_SERVERS.write().await.retain(|server| {
        if server.ip == server_data.ip().to_string() {
            return false;
        }
        true
    });
}
