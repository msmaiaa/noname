use std::str::FromStr;

use axum::{
    body::Body,
    headers::HeaderName,
    http::{HeaderValue, Request},
    response::{Html, IntoResponse},
};
use reqwest::StatusCode;
use serde::Serialize;

use crate::{error::AppError, response::AppResponse, service::auth::*};
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

pub async fn login(_: Request<Body>) -> Result<impl IntoResponse, AppError> {
    let redirector = generate_steam_redirector()?;
    let res = axum::response::Response::builder()
        .header(
            HeaderName::from_str("Location").unwrap(),
            HeaderValue::from_str(redirector.url().as_str()).map_err(|e| {
                tracing::error!("Failed to create redirector: {}", e);
                AppError::Unauthorized
            })?,
        )
        .status(StatusCode::FOUND)
        .body(Body::empty());
    Ok(res.unwrap())
}

#[derive(Serialize)]
pub struct CurrentUserResponse {
    personaname: String,
    avatar: String,
    is_admin: bool,
}

pub async fn steam_callback(req: Request<Body>) -> Result<impl IntoResponse, AppError> {
    let qs = req.uri().query().ok_or(AppError::Unauthorized)?;
    let (token, steam_user) = on_steam_callback(qs).await?;
    let user_in_db =
        crate::model::user::select_by_steamid(&mut crate::global::RB.clone(), steam_user.steamid)
            .await
            .map_err(|e| {
                tracing::error!("Failed to select user: {}", e);
                AppError::DatabaseError(e)
            })?
            .unwrap();

    let html = format!(
        r#"
		<!DOCTYPE html>
		<html>
			<head>
				<title>noname</title>
			</head>
			<body>
			</body>
			<script>
				let data = {{
					token: "{}",
					personaname: "{}",
					avatar: "{}",
					is_admin: {}
				}}
				window.opener.parent.postMessage(data, "*");
				window.close();
			</script>
		</html>
		"#,
        token, steam_user.personaname, steam_user.avatar, user_in_db.is_admin
    );
    Ok(Html::from(html))
}
