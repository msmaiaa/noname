use salvo::{
    handler,
    http::HeaderValue,
    hyper::{header::LOCATION, HeaderMap},
    prelude::{StatusCode, StatusError},
    Request, Response,
};
use serde::{Deserialize, Serialize};
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

#[derive(Serialize, Deserialize, Clone)]
struct SteamUser {
    steamid: String,
    communityvisibilitystate: i32,
    profilestate: i32,
    personaname: String,
    commentpermission: i32,
    profileurl: String,
    avatar: String,
    avatarmedium: String,
    avatarfull: String,
    avatarhash: String,
    lastlogoff: i32,
    personastate: i32,
    realname: String,
    primaryclanid: String,
    timecreated: i32,
    personastateflags: i32,
    loccountrycode: String,
}

#[derive(Deserialize)]
struct SteamResponsePlayers {
    players: Vec<SteamUser>,
}
#[derive(Deserialize)]
struct SteamGetPlayerSummaryResponse {
    response: SteamResponsePlayers,
}

#[handler]
pub async fn steam_callback(req: &mut Request, res: &mut Response) -> Result<(), StatusError> {
    let qs = req.uri().query().ok_or(StatusError::bad_request())?;

    let steamid = verify_steam_request(qs).await?;
    let user = query_steam_user(steamid).await?;
    res.render(salvo::writer::Json(user));
    Ok(())
}

async fn query_steam_user(steamid: u64) -> Result<SteamUser, StatusError> {
    let steam_key = std::env::var("STEAM_KEY").map_err(|_| {
        tracing::error!("STEAM_KEY is not set in the environment variables");
        StatusError::internal_server_error()
    })?;

    let player_summary_api_url = format!(
        "http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
        steam_key, steamid
    );
    let resp = reqwest::get(player_summary_api_url.as_str())
        .await
        .map_err(|_| StatusError::internal_server_error())?;
    let body = resp
        .text()
        .await
        .map_err(|_| StatusError::internal_server_error())?;
    let body: SteamGetPlayerSummaryResponse = serde_json::from_str(body.as_str()).map_err(|e| {
        tracing::error!("Failed to parse steam response: {}", e);
        StatusError::internal_server_error()
    })?;
    if body.response.players.len() == 0 {
        return Err(StatusError::unauthorized());
    }
    Ok(body.response.players[0].clone())
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
