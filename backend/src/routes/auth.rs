use salvo::{
    handler,
    http::HeaderValue,
    hyper::{header::LOCATION, HeaderMap},
    prelude::{StatusCode, StatusError},
    Request, Response,
};
use serde::Serialize;

use crate::service::auth::*;
#[derive(Serialize)]
pub struct LoginResponse {
    token: String,
    personaname: String,
    avatar: String,
}

impl LoginResponse {
    pub fn new(token: String, personaname: String, avatar: String) -> Self {
        Self {
            token,
            personaname,
            avatar,
        }
    }
}

#[handler]
pub fn login(_: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let redirector = generate_steam_redirector()?;
    res.set_status_code(StatusCode::FOUND);
    let mut headers = HeaderMap::new();
    headers.insert(
        LOCATION,
        HeaderValue::from_str(redirector.url().as_str()).map_err(|e| {
            tracing::error!("Failed to create redirector: {}", e);
            StatusError::internal_server_error()
        })?,
    );
    res.set_headers(headers);
    Ok(())
}

#[handler]
pub async fn steam_callback(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let qs = req.uri().query().ok_or(StatusError::bad_request())?;
    let (token, steam_user) = on_steam_callback(qs).await?;
    res.render(salvo::writer::Json(LoginResponse::new(
        token,
        steam_user.personaname,
        steam_user.avatar,
    )));

    Ok(())
}
