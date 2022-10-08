use anyhow::anyhow;
use axum::{
    body::Body,
    extract::ws::{Message, WebSocket},
    http::Request,
};
use futures_util::{FutureExt, StreamExt};

use crate::model::user;
use crate::{error::AppError, service::auth::TokenData};

use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::global::ONLINE_USERS;
pub type UserList = RwLock<Vec<ConnectedUser>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectedUser {
    pub steamid64: String,
    #[serde(skip, default = "default_conn")]
    pub conn: mpsc::UnboundedSender<Result<Message, axum::Error>>,
}

fn default_conn() -> mpsc::UnboundedSender<Result<Message, axum::Error>> {
    let (tx, _) = mpsc::unbounded_channel();
    tx
}

pub async fn get_online_users() -> Vec<ConnectedUser> {
    ONLINE_USERS.read().await.to_vec()
}

pub async fn authorize_user_connection(
    _: Request<Body>,
    token_data: TokenData,
) -> Result<user::User, AppError> {
    let found_user = user::select_by_steamid(&mut crate::global::RB.clone(), token_data.steamid64)
        .await
        .map_err(|e| AppError::DatabaseError(e))?
        .ok_or(AppError::Unauthorized)?;
    Ok(found_user)
}

pub async fn handle_user_connection(ws: WebSocket, connected_user: user::User) {
    //adds the server to the list
    let (user_ws_tx, mut user_ws_rx) = ws.split();

    let (tx, rx) = mpsc::unbounded_channel();
    let rx = UnboundedReceiverStream::new(rx);
    let fut = rx.forward(user_ws_tx).map(|result| {
        if let Err(e) = result {
            tracing::error!(error = ?e, "websocket send error");
        }
    });
    tokio::task::spawn(fut);
    let _connected_user = connected_user.clone();
    let fut = async move {
        ONLINE_USERS.write().await.push(ConnectedUser {
            steamid64: _connected_user.steamid64.clone(),
            conn: tx,
        });

        let mut close_stream = false;
        while let Some(result) = user_ws_rx.next().await {
            if close_stream {
                break;
            }
            let msg = match result {
                Ok(msg) => msg,
                Err(_) => {
                    break;
                }
            };
            on_user_message(&_connected_user, msg)
                .await
                .map_err(|e| {
                    close_stream = true;
                    tracing::error!(error = ?e, "Error while handling server message");
                })
                .ok();
        }

        on_user_disconnected(&_connected_user).await;
    };
    tokio::task::spawn(fut);
}

async fn on_user_message(user_data: &user::User, msg: Message) -> anyhow::Result<()> {
    let msg = msg.to_text()?;
    if msg.starts_with("user") {
        handle_user_message(user_data, msg)
            .await
            .map_err(|_| anyhow!("Invalid user message"))?
    } else if msg.starts_with("admin") {
        handle_admin_message(user_data, msg)
            .await
            .map_err(|_| anyhow!("Invalid admin message"))?
    } else {
        tracing::warn!("Unknown message {} from user {}", msg, user_data.steamid64);
    }

    Ok(())
}

async fn handle_user_message(user_data: &user::User, msg: &str) -> Result<(), AppError> {
    match msg {
        "user_ping" => send_message_to_user(&user_data.steamid64, "pong".to_string()).await,
        _ => {
            tracing::warn!("Unknown message from user {}", user_data.steamid64);
        }
    }
    Ok(())
}

async fn handle_admin_message(user_data: &user::User, msg: &str) -> Result<(), AppError> {
    match user_data.is_admin {
        true => {}
        false => {
            tracing::warn!(
                "Unauthorized user {} is trying to send privileged messages",
                user_data.steamid64
            );
            ONLINE_USERS
                .write()
                .await
                .retain(|u| u.steamid64 != user_data.steamid64);
            return Err(AppError::Unauthorized);
        }
    }

    match msg {
        "admin_get_servers" => {
            let servers = crate::service::server::get_servers().await?;
            let servers =
                serde_json::to_string(&servers).map_err(|e| AppError::JsonParseError(e))?;
            send_message_to_user(&user_data.steamid64, servers).await;
        }
        _ => {
            tracing::warn!("Unknown message {} from user {}", msg, user_data.steamid64);
        }
    }

    Ok(())
}

async fn send_message_to_user(steamid64: &String, message: String) {
    let online_users = ONLINE_USERS.read().await;
    let user = online_users
        .iter()
        .find(|user| user.steamid64 == *steamid64);
    if let Some(user) = user {
        user.conn
            .send(Ok(Message::Text(message)))
            .map_err(|e| tracing::error!(error = ?e, "Error while sending message to user"))
            .ok();
    }
}

async fn on_user_disconnected(user_data: &user::User) {
    tracing::info!("Server {} disconnected", user_data.steamid64);
    ONLINE_USERS.write().await.retain(|user| {
        if user.steamid64 == user_data.steamid64 {
            return false;
        }
        true
    });
}
