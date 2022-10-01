use futures_util::{FutureExt, StreamExt};

use crate::model::user;
use crate::service::auth::extract_data_from_depot;
use salvo::extra::ws::{Message, WebSocket};
use salvo::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, RwLock};
use tokio_stream::wrappers::UnboundedReceiverStream;

use crate::global::ONLINE_USERS;
pub type UserList = RwLock<Vec<ConnectedUser>>;

#[derive(Serialize, Deserialize, Clone)]
pub struct ConnectedUser {
    pub steamid64: String,
    #[serde(skip, default = "default_conn")]
    pub conn: mpsc::UnboundedSender<Result<Message, salvo::Error>>,
}

fn default_conn() -> mpsc::UnboundedSender<Result<Message, salvo::Error>> {
    let (tx, _) = mpsc::unbounded_channel();
    tx
}

pub async fn get_online_users() -> Vec<ConnectedUser> {
    ONLINE_USERS.read().await.to_vec()
}

pub async fn authorize_user_connection(
    _: &mut Request,
    depot: &mut Depot,
    _: &mut Response,
) -> Result<user::User, StatusError> {
    let token_data = extract_data_from_depot(depot).ok_or(StatusError::unauthorized())?;
    let found_user = user::select_by_steamid(&mut crate::global::RB.clone(), token_data.steamid64)
        .await
        .map_err(|e| StatusError::internal_server_error())?
        .ok_or(StatusError::unauthorized())?;
    Ok(found_user)
}

pub async fn handle_user_connection(ws: WebSocket, connected_server: &user::User) {
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
    let _connected_user = connected_server.clone();
    let fut = async move {
        ONLINE_USERS.write().await.push(ConnectedUser {
            steamid64: _connected_user.steamid64.clone(),
            conn: tx,
        });

        while let Some(result) = user_ws_rx.next().await {
            let msg = match result {
                Ok(msg) => msg,
                Err(_) => {
                    break;
                }
            };
            on_user_message(&_connected_user, msg)
                .await
                .map_err(|e| {
                    tracing::error!(error = ?e, "Error while handling server message");
                })
                .ok();
        }

        on_user_disconnected(&_connected_user).await;
    };
    tokio::task::spawn(fut);
}

#[derive(Deserialize)]
enum UserAction {}

#[derive(Deserialize)]
struct UserMessageData {}

#[derive(Deserialize)]
struct UserMessage {
    action: UserAction,
    data: UserMessageData,
}

async fn on_user_message(user_data: &user::User, msg: Message) -> anyhow::Result<()> {
    if !msg.is_text() {
        return Ok(());
    }
    let msg = msg.to_str()?;
    let parsed_msg: UserMessage = serde_json::from_str(msg)?;

    Ok(())
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
