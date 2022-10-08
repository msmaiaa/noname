use axum::Json;
use rbatis::rbdc::db::ExecResult;
use serde::Deserialize;

use crate::error::AppError;
use crate::response::AppResponse;
use crate::service::auth::TokenData;
use crate::service::server;
#[derive(Deserialize)]
pub struct CreateServerPayload {
    pub ip: String,
    pub port: String,
}

pub async fn create_server(
    Json(body): Json<CreateServerPayload>,
    token_data: TokenData,
) -> Result<AppResponse<ExecResult>, AppError> {
    tracing::info!("Creating server for user {}", token_data.steamid64);
    let created_server = server::create_server(body).await?;
    Ok(AppResponse::created(created_server))
}
