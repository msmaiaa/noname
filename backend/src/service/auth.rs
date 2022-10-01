use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header};

use salvo::Depot;
use serde::{Deserialize, Serialize};
use steam_auth;

use crate::error::AppError;
use crate::global;
use crate::model::user::User;

#[derive(Serialize, Deserialize, Clone)]
pub struct TokenData {
    pub steamid64: String,
    pub iat: i64,
    pub exp: i64,
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SteamUser {
    pub steamid: String,
    pub communityvisibilitystate: i32,
    pub profilestate: i32,
    pub personaname: String,
    pub commentpermission: i32,
    pub profileurl: String,
    pub avatar: String,
    pub avatarmedium: String,
    pub avatarfull: String,
    pub avatarhash: String,
    pub lastlogoff: i32,
    pub personastate: i32,
    pub realname: String,
    pub primaryclanid: String,
    pub timecreated: i32,
    pub personastateflags: i32,
    pub loccountrycode: String,
}

#[derive(Deserialize)]
struct SteamResponsePlayers {
    players: Vec<SteamUser>,
}
#[derive(Deserialize)]
struct SteamGetPlayerSummaryResponse {
    response: SteamResponsePlayers,
}

pub fn token_data_from_depot(depot: &mut Depot) -> Option<TokenData> {
    depot
        .get::<TokenData>("token_data")
        .map(|data| data.clone())
}

///	/login -> redirect to steam -> /steam_callback -> verify stuff -> create token -> send token and some data to client
pub fn generate_steam_redirector() -> Result<steam_auth::Redirector, AppError> {
    steam_auth::Redirector::new(global::API_URL.to_string(), "/api/auth/steam_callback").map_err(
        |e| {
            tracing::error!("Failed to create redirector: {}", e);
            AppError::SteamError(e)
        },
    )
}

pub fn create_access_token(steamid64: String) -> Result<String, AppError> {
    let iat = Utc::now();
    let exp = iat + Duration::seconds(3600);
    let iat = iat.timestamp_millis();
    let exp = exp.timestamp_millis();

    let key = EncodingKey::from_secret(crate::global::JWT_KEY.as_bytes());
    let claims = TokenData {
        steamid64,
        iat,
        exp,
    };
    let header = Header::new(Algorithm::HS256);
    encode(&header, &claims, &key).map_err(|e| {
        tracing::error!("Failed to create access token: {}", e);
        AppError::JwtError(e)
    })
}

pub async fn on_steam_callback(qs: &str) -> Result<(String, SteamUser), AppError> {
    let steamid64 = verify_steam_request(qs).await?;

    let mut _token = String::new();
    match crate::model::user::select_by_steamid(&mut global::RB.clone(), steamid64.to_string())
        .await
        .map_err(|e| {
            tracing::error!("Failed to select user: {}", e);
            AppError::DatabaseError(e)
        })? {
        Some(user) => {
            _token = create_access_token(user.steamid64)?;
        }
        None => {
            let new_user = User::from_steamid64(steamid64);
            User::insert(&mut global::RB.clone(), &new_user)
                .await
                .map_err(|e| {
                    tracing::error!("Failed to insert user: {}", e);
                    AppError::DatabaseError(e)
                })?;
            _token = create_access_token(new_user.steamid64)?;
        }
    }
    let steam_user = query_steam_user(steamid64).await?;
    Ok((_token, steam_user))
}

pub async fn query_steam_user(steamid: u64) -> Result<SteamUser, AppError> {
    let player_summary_api_url = format!(
        "http://api.steampowered.com/ISteamUser/GetPlayerSummaries/v0002/?key={}&steamids={}",
        *global::STEAM_KEY,
        steamid
    );
    let resp = reqwest::get(&player_summary_api_url)
        .await
        .map_err(|e| AppError::SteamApiError(e))?;

    let body = resp.text().await.map_err(|e| AppError::ReqwestError(e))?;
    let body: SteamGetPlayerSummaryResponse = serde_json::from_str(&body).map_err(|e| {
        tracing::error!("Failed to parse steam response: {}", e);
        AppError::JsonParseError(e)
    })?;

    if body.response.players.len() == 0 {
        return Err(AppError::Unauthorized);
    }
    Ok(body.response.players[0].clone())
}

pub async fn verify_steam_request(query_string: &str) -> Result<u64, AppError> {
    let client = reqwest::Client::new();

    let (req, verifier) = steam_auth::Verifier::from_querystring(query_string).map_err(|e| {
        tracing::error!("Failed to create verifier: {}", e);
        AppError::SteamVerifierError(e)
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
            AppError::ReqwestError(e)
        })
        .map(|res| async { res.text().await })?
        .await
        .map_err(|e| AppError::ReqwestError(e))?;

    verifier
        .verify_response(response_string)
        .map_err(|_| AppError::Unauthorized)
}

pub fn decode_token(token: &str) -> Result<TokenData, jsonwebtoken::errors::Error> {
    let key = DecodingKey::from_secret(global::JWT_KEY.as_bytes());
    decode::<TokenData>(&token, &key, &jsonwebtoken::Validation::default()).map(|data| data.claims)
}
