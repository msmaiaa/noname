use std::net::SocketAddr;

use axum::{
    body::Body,
    extract::ws::{Message, WebSocket},
    http::Request,
};
use futures_util::{FutureExt, StreamExt};

use crate::{error::AppError, model::server};
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::global::ONLINE_SERVERS;
pub type ServerList = RwLock<Vec<ConnectedServer>>;

#[derive(Serialize, Deserialize, Clone, Copy)]
pub enum ServerStatus {
    Idle,
    WaitingForPlayers,
    Starting,
    KnifeRound,
    Live,
    Ending,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectedServer {
    pub id: u32,
    pub ip: String,
    pub port: String,
    pub status: ServerStatus,
    #[serde(skip, default = "default_conn")]
    pub conn: mpsc::UnboundedSender<Result<Message, axum::Error>>,
}

#[derive(Deserialize)]
enum ServerMessageData {
    #[serde(rename = "status")]
    Status(ServerStatus),
}

fn default_conn() -> mpsc::UnboundedSender<Result<Message, axum::Error>> {
    let (tx, _) = mpsc::unbounded_channel();
    tx
}

pub async fn get_online_servers() -> Vec<ConnectedServer> {
    ONLINE_SERVERS.read().await.to_vec()
}

pub async fn authorize_server_connection(
    req: Request<Body>,
    addr: SocketAddr,
) -> Result<server::Server, AppError> {
    let port: String = req
        .headers()
        .get("PORT")
        .map(|p| p.to_str().unwrap().to_string())
        .ok_or(AppError::Unauthorized)?;

    let found_server =
        server::select_by_full_ip(&mut crate::global::RB.clone(), addr.ip().to_string(), port)
            .await
            .map_err(|e| AppError::DatabaseError(e))?
            .ok_or({
                tracing::info!("Unauthorized server tried to connect: {}", addr.ip());
                AppError::Unauthorized
            })?;
    Ok(found_server)
}

pub async fn handle_server_connection(ws: WebSocket, connected_server: server::Server) {
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
    let _connected_server = connected_server.clone();
    let fut = async move {
        ONLINE_SERVERS.write().await.push(ConnectedServer {
            id: _connected_server.id.unwrap(),
            ip: _connected_server.ip.clone(),
            port: _connected_server.port.clone(),
            status: ServerStatus::Idle,
            conn: tx,
        });

        while let Some(result) = server_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(e) => {
                    eprintln!(
                        "websocket server error(uid={}): {}",
                        _connected_server.ip, e
                    );
                    break;
                }
            };
            on_server_message(&_connected_server, msg)
                .await
                .map_err(|e| {
                    tracing::error!(error = ?e, "Error while handling server message");
                })
                .ok();
        }

        on_server_disconnected(&_connected_server).await;
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

async fn on_server_message(server_data: &server::Server, msg: Message) -> anyhow::Result<()> {
    let msg = msg.to_text()?;
    let parsed_msg: ServerMessage = serde_json::from_str(msg)?;

    match parsed_msg.action {
        ServerAction::Server2BackendUpdateStatus => {
            let status = match parsed_msg.data {
                ServerMessageData::Status(status) => status,
            };
            for server in ONLINE_SERVERS.write().await.iter_mut() {
                if server.ip == server_data.ip && server.port == server_data.port {
                    server.status = status;
                }
            }
        }
    }
    Ok(())
}

async fn on_server_disconnected(server_data: &server::Server) {
    tracing::info!("Server {} disconnected", server_data.ip);
    ONLINE_SERVERS.write().await.retain(|server| {
        if server.ip == server_data.ip && server.port == server_data.port {
            return false;
        }
        true
    });
}
