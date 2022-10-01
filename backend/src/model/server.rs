use rbatis::{crud, rbdc::datetime::FastDateTime, sql, Rbatis};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: Option<u32>,
    pub ip: String,
    pub port: String,
    pub created_at: FastDateTime,
}
crud!(Server {});

#[sql("select * from server where ip = ? and port = ? limit 1")]
pub async fn select_by_full_ip(
    rb: &Rbatis,
    ip: String,
    port: String,
) -> rbatis::Result<Option<Server>> {
    impled!()
}
