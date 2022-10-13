use axum::{
    extract::{Query, RequestParts},
    http::Request,
    middleware::Next,
    response::IntoResponse,
};
use serde::Deserialize;

use crate::{
    error::AppError,
    global,
    service::auth::{decode_token, TokenData},
};

///	Extracts the authorization header and checks if the access should be granted
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

    match user_is_admin(&token_data).await? {
        true => {
            req.extensions_mut().insert(token_data);
            Ok(next.run(req).await)
        }
        false => Err(AppError::Unauthorized),
    }
}

#[derive(Deserialize)]
struct TokenQueryString {
    token: String,
}

/// The authorization token from a browser connection must be sent as a query string parameter because the browsers implementation of websockets doesn't support custom headers.
pub async fn with_admin_qs<B: Send>(
    req: Request<B>,
    next: Next<B>,
) -> Result<impl IntoResponse, AppError> {
    let mut request_parts = RequestParts::new(req);
    let token_query = request_parts
        .extract::<Query<TokenQueryString>>()
        .await
        .map_err(|_| AppError::Unauthorized)?;
    let token_data = decode_token(&token_query.token).map_err(|_| AppError::Unauthorized)?;

    match user_is_admin(&token_data).await? {
        true => {
            request_parts.extensions_mut().insert(token_data);
            let request = request_parts.try_into_request().expect("body extracted");
            Ok(next.run(request).await)
        }
        false => Err(AppError::Unauthorized),
    }
}

///	Checks the authorization header for a valid token
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

async fn user_is_admin(token_data: &TokenData) -> Result<bool, AppError> {
    let user = crate::model::user::select_by_steamid(
        &mut global::RB.clone(),
        token_data.steamid64.clone(),
    )
    .await
    .map_err(|_| AppError::Unauthorized)?
    .ok_or(AppError::Unauthorized)?;
    Ok(user.is_admin)
}
