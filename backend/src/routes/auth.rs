use chrono::{Duration, Utc};
use jsonwebtoken::errors::Error as JwtError;
use jsonwebtoken::{encode, Algorithm, EncodingKey, Header};
use rbatis::rbdc::datetime::FastDateTime;
use salvo::{
    handler,
    http::HeaderValue,
    hyper::{header::LOCATION, HeaderMap},
    prelude::{StatusCode, StatusError},
    Request, Response,
};
use serde::{Deserialize, Serialize};
use steam_auth;

use crate::model::user::User;

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

#[derive(Serialize)]
pub struct TokenData {
    pub steamid64: String,
    pub is_admin: bool,
    pub iat: i64,
    pub exp: i64,
}

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

    let steamid64 = verify_steam_request(qs).await?;

    let mut token = String::new();
    match crate::model::user::select_by_steamid(
        &mut crate::global::RB.clone(),
        steamid64.to_string(),
    )
    .await
    .map_err(|e| {
        tracing::error!("Failed to select user: {}", e);
        StatusError::internal_server_error()
    })? {
        Some(user) => {
            token = create_access_token(user.steamid64, user.is_admin).map_err(|e| {
                tracing::error!("Failed to create access token: {}", e);
                StatusError::internal_server_error()
            })?;
        }
        None => {
            let new_user = User {
                steamid64: steamid64.to_string(),
                is_admin: false,
                created_at: FastDateTime::now(),
            };
            User::insert(&mut crate::global::RB.clone(), &new_user)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to insert user: {}", e);
                    StatusError::internal_server_error()
                })?;
            token = create_access_token(new_user.steamid64, new_user.is_admin).map_err(|e| {
                tracing::error!("Failed to create access token: {}", e);
                StatusError::internal_server_error()
            })?;
        }
    }
    let steam_user = query_steam_user(steamid64).await?;
    res.render(salvo::writer::Json(LoginResponse::new(
        token,
        steam_user.personaname,
        steam_user.avatar,
    )));

    Ok(())
}

pub fn create_access_token(steamid64: String, is_admin: bool) -> Result<String, JwtError> {
    let iat = Utc::now();
    let exp = iat + Duration::seconds(3600);
    let iat = iat.timestamp_millis();
    let exp = exp.timestamp_millis();

    let key = EncodingKey::from_secret(
        std::env::var("JWT_KEY")
            .expect("JWT_KEY not set")
            .as_bytes(),
    );
    let claims = TokenData {
        steamid64,
        is_admin,
        iat,
        exp,
    };
    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &key)
}

#[derive(Deserialize)]
struct SteamResponsePlayers {
    players: Vec<SteamUser>,
}
#[derive(Deserialize)]
struct SteamGetPlayerSummaryResponse {
    response: SteamResponsePlayers,
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
