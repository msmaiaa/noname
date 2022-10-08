use axum::{http::Request, middleware::Next, response::IntoResponse};

use crate::{error::AppError, global, service::auth::decode_token};

pub async fn with_admin<B>(req: Request<B>, next: Next<B>) -> Result<impl IntoResponse, AppError> {
    let header_token = req
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(AppError::Unauthorized)?
        .replace("Bearer ", "");
    let token_data = decode_token(&header_token).map_err(|_| AppError::Unauthorized)?;
    let user = crate::model::user::select_by_steamid(
        &mut global::RB.clone(),
        token_data.steamid64.clone(),
    )
    .await
    .map_err(|_| AppError::Unauthorized)?
    .ok_or(AppError::Unauthorized)?;

    match user.is_admin {
        true => Ok(next.run(req).await),
        false => Err(AppError::Unauthorized),
    }
}
