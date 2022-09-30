use rbatis::rbdc::datetime::FastDateTime;
use rbatis::rbdc::db::ExecResult;

use crate::global;
use crate::model;
use crate::model::server::Server;
use crate::{error::AppError, routes::server::CreateServerPayload};

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
