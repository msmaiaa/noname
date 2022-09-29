use salvo::{
    handler,
    http::HeaderValue,
    hyper::{header::LOCATION, HeaderMap},
    prelude::{StatusCode, StatusError},
    Request, Response,
};
use steam_auth;

enum Error {
    Any,
}

#[handler]
pub fn login(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let host = std::env::var("HOST").unwrap_or("localhost".to_string());
    let port = std::env::var("PORT").unwrap_or("1337".to_string());
    let redirector =
        steam_auth::Redirector::new(format!("http://{}:{}", host, port), "/auth/steam_callback")
            .map_err(|e| {
                tracing::error!("Failed to create redirector: {}", e);
                StatusError::internal_server_error()
            })?;
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

    let steamid = verify_steam_request(qs).await?;

    res.render(steamid.to_string());
    Ok(())
}

async fn verify_steam_request(query_string: &str) -> Result<u64, StatusError> {
    let client = reqwest::Client::new();

    let (req, verifier) = steam_auth::Verifier::from_querystring(query_string).map_err(|e| {
        tracing::error!("Failed to create verifier: {}", e);
        StatusError::internal_server_error()
    })?;

    let (parts, body) = req.into_parts();

    let response_string = client
        .post(&parts.uri.to_string())
        .header("Content-Type", "application/x-www-form-urlencoded")
        .body(body)
        .send()
        .await
        .map_err(|e| {
            tracing::error!("Failed to send request: {}", e);
            StatusError::internal_server_error()
        })
        .map(|res| async { res.text().await })?
        .await
        .map_err(|_| StatusError::internal_server_error())?;
    verifier
        .verify_response(response_string)
        .map_err(|_| StatusError::unauthorized())
}
