use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Serialize;

pub enum AppError {
    Unauthorized,
    BadRequest(String),
    SteamError(steam_auth::Error),
    JwtError(jsonwebtoken::errors::Error),
    DatabaseError(rbatis::Error),
    SteamVerifierError(steam_auth::Error),
    SteamApiError(reqwest::Error),
    ReqwestError(reqwest::Error),
    JsonParseError(serde_json::Error),
}

#[derive(Serialize)]
struct JsonError {
    pub error: String,
}

impl From<String> for JsonError {
    fn from(error: String) -> Self {
        JsonError { error }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        match self {
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized").into_response(),
            AppError::BadRequest(e) => (StatusCode::BAD_REQUEST, e).into_response(),
            AppError::SteamError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonError::from(e.to_string())),
            )
                .into_response(),
            AppError::JwtError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonError::from(e.to_string())),
            )
                .into_response(),
            AppError::DatabaseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonError::from(e.to_string())),
            )
                .into_response(),
            AppError::SteamVerifierError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonError::from(e.to_string())),
            )
                .into_response(),
            AppError::ReqwestError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonError::from(e.to_string())),
            )
                .into_response(),
            AppError::SteamApiError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonError::from(e.to_string())),
            )
                .into_response(),
            AppError::JsonParseError(e) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(JsonError::from(e.to_string())),
            )
                .into_response(),
        }
    }
}
