use salvo::prelude::StatusError;

pub enum AppError {
    Unauthorized,
    SteamError(steam_auth::Error),
    JwtError(jsonwebtoken::errors::Error),
    DatabaseError(rbatis::Error),
    SteamVerifierError(steam_auth::Error),
    SteamApiError(reqwest::Error),
    ReqwestError(reqwest::Error),
    JsonParseError(serde_json::Error),
}

impl From<AppError> for StatusError {
    fn from(e: AppError) -> Self {
        match e {
            AppError::Unauthorized => StatusError::unauthorized(),
            AppError::SteamError(_) => StatusError::internal_server_error(),
            AppError::JwtError(_) => StatusError::internal_server_error(),
            AppError::DatabaseError(_) => StatusError::internal_server_error(),
            AppError::SteamVerifierError(_) => StatusError::internal_server_error(),
            AppError::ReqwestError(_) => StatusError::internal_server_error(),
            AppError::SteamApiError(_) => StatusError::internal_server_error(),
            AppError::JsonParseError(_) => StatusError::internal_server_error(),
        }
    }
}
