use axum::{http::Request, middleware::Next, response::IntoResponse, extract::{RequestParts, Query}};
use serde::Deserialize;

use crate::{error::AppError, global, service::auth::decode_token};

pub async fn with_admin<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, AppError> {
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
        true => {
            req.extensions_mut().insert(token_data);
            Ok(next.run(req).await)
        }
        false => Err(AppError::Unauthorized),
    }
}

#[derive(Deserialize)]
struct TokenQueryString {
    token: String
}

pub async fn with_admin_qs<B: Send>(
        req: Request<B>,
        next: Next<B>,
        ) -> Result<impl IntoResponse, AppError> {
    let mut request_parts = RequestParts::new(req);
    let token_query = request_parts.extract::<Query<TokenQueryString>>().await.map_err(|_| AppError::Unauthorized)?;
    let token_data = decode_token(&token_query.token).map_err(|_| AppError::Unauthorized)?;
    let user = crate::model::user::select_by_steamid(
            &mut global::RB.clone(),
    token_data.steamid64.clone(),
    )
    .await
    .map_err(|_| AppError::Unauthorized)?
    .ok_or(AppError::Unauthorized)?;

    match user.is_admin {
        true => {
            request_parts.extensions_mut().insert(token_data);
            let request = request_parts.try_into_request().expect("body extracted");
            Ok(next.run(request).await)
        }
        false => Err(AppError::Unauthorized),
    }
}

pub async fn with_auth<B>(
    mut req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, AppError> {
    let header_token = req
        .headers()
        .get("authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(AppError::Unauthorized)?
        .replace("Bearer ", "");
    let token_data = decode_token(&header_token).map_err(|_| AppError::Unauthorized)?;
    req.extensions_mut().insert(token_data);
    Ok(next.run(req).await)
}
