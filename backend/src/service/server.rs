use std::collections::HashMap;

use rbatis::rbdc::datetime::FastDateTime;
use rbatis::rbdc::db::ExecResult;
use serde::Serialize;

use crate::global;
use crate::model;
use crate::ws::server::get_online_servers;
use crate::ws::server::ServerStatus;
use crate::{error::AppError, routes::server::CreateServerPayload};

#[derive(Serialize)]
pub struct ServerWithStatus {
    id: u32,
    ip: String,
    port: String,
    status: ServerStatus,
    online: bool,
}

pub async fn create_server(payload: CreateServerPayload) -> Result<ExecResult, AppError> {
    let new_server = model::server::Server {
        id: None,
        ip: payload.ip,
        port: payload.port,
        created_at: FastDateTime::now(),
    };
    model::server::Server::insert(&mut global::RB.clone(), &new_server)
        .await
        .map_err(|e| AppError::DatabaseError(e))
        .map(|res| Ok(res))?
}

pub async fn get_servers() -> Result<Vec<ServerWithStatus>, AppError> {
    let servers_in_db = model::server::Server::select_all(&mut global::RB.clone())
        .await
        .map_err(|e| AppError::DatabaseError(e))?;
    let online_servers = get_online_servers().await;
    let mut server_map = HashMap::new();
    for i in servers_in_db {
        server_map.insert(
            i.id.unwrap(),
            ServerWithStatus {
                id: i.id.unwrap(),
                ip: i.ip,
                port: i.port,
                status: ServerStatus::Idle,
                online: false,
            },
        );
    }
    for s in online_servers {
        if let Some(server) = server_map.get_mut(&s.id) {
            server.status = s.status;
            server.online = true;
        }
    }
    Ok(server_map.into_iter().map(|(_, v)| v).collect())
}
