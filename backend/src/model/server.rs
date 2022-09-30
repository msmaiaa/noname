use rbatis::{crud, rbdc::datetime::FastDateTime};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct Server {
    pub id: Option<u32>,
    pub ip: String,
    pub port: String,
    pub created_at: FastDateTime,
}
crud!(Server {});
