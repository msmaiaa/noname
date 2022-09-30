use rbatis::{crud, rbdc::datetime::FastDateTime, sql, Rbatis};
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone)]
pub struct User {
    pub steamid64: String,
    pub is_admin: bool,
    pub created_at: FastDateTime,
}
crud!(User {}, "app_user");

#[sql("select * from app_user where steamid64 = ? limit 1")]
pub async fn select_by_steamid(rb: &Rbatis, steamid64: String) -> rbatis::Result<Option<User>> {
    impled!()
}

impl User {
    pub fn from_steamid64(steamid64: u64) -> Self {
        Self {
            steamid64: steamid64.to_string(),
            is_admin: false,
            created_at: FastDateTime::now(),
        }
    }
}
