use salvo::{handler, writer::Json, Request, Response};

use crate::ws::server::get_online_servers;

#[handler]
pub async fn get_servers(req: &mut Request, res: &mut Response) {
    res.render(Json(get_online_servers().await))
}
