use axum::{
    async_trait,
    extract::{FromRequest, RequestParts},
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::{
    error::AppError,
    service::auth::{decode_token, TokenData},
};

#[async_trait]
impl<B> FromRequest<B> for TokenData
where
    B: Send,
{
    type Rejection = AppError;

    async fn from_request(req: &mut RequestParts<B>) -> Result<Self, Self::Rejection> {
        let claims = claims_from_req(req).await?;
        Ok(claims)
    }
}

pub async fn claims_from_req<B: Send>(req: &mut RequestParts<B>) -> Result<TokenData, AppError> {
    let TypedHeader(Authorization(bearer)) =
        TypedHeader::<Authorization<Bearer>>::from_request(req)
            .await
            .map_err(|_| AppError::Unauthorized)?;
    let claims = decode_token(bearer.token()).map_err(|e| {
        tracing::error!("{:?}", e.to_string());
        AppError::JwtError(e)
    })?;
    Ok(claims)
}
