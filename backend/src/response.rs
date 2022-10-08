use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;

pub struct AppResponse<T: Serialize> {
    status_code: StatusCode,
    body: T,
}

#[derive(Serialize)]
pub struct ResponseBody<T: Serialize> {
    data: T,
}

impl<T: Serialize> ResponseBody<T> {
    pub fn new(data: T) -> Self {
        ResponseBody { data }
    }
}

impl<T: Serialize> AppResponse<T> {
    pub fn ok(body: T) -> Self {
        Self {
            status_code: StatusCode::OK,
            body,
        }
    }

    pub fn created(body: T) -> Self {
        Self {
            status_code: StatusCode::CREATED,
            body,
        }
    }
}

impl<T: Serialize> IntoResponse for AppResponse<T> {
    fn into_response(self) -> Response {
        (self.status_code, Json(ResponseBody::new(self.body))).into_response()
    }
}
