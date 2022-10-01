use salvo::Depot;
use salvo::{handler, prelude::StatusError, writer::Json, Request, Response};
use serde::Deserialize;

use crate::service::auth::extract_data_from_depot;
use crate::service::server;
use crate::ws::server::get_online_servers;
use salvo::http::StatusCode;
#[derive(Deserialize)]
pub struct CreateServerPayload {
    pub ip: String,
    pub port: String,
}

#[handler]
pub async fn get_servers(req: &mut Request, res: &mut Response) {
    res.render(Json(get_online_servers().await))
}

#[handler]
pub async fn create_server(
    req: &mut Request,
    depot: &mut Depot,
    res: &mut Response,
) -> Result<(), StatusError> {
    let data = extract_data_from_depot(depot).unwrap();
    tracing::info!("Creating server for user {}", data.steamid64);
    let payload = req.parse_body::<CreateServerPayload>().await.map_err(|e| {
        res.render(e.to_string());
        StatusError::bad_request()
    })?;
    //TODO: better error handling
    let created_server = server::create_server(payload).await?;
    res.set_status_code(StatusCode::CREATED);
    res.render(Json(created_server));
    Ok(())
}
