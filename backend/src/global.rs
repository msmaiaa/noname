use lazy_static::lazy_static;
use rbatis::Rbatis;

use crate::ws::server::ServerList;

lazy_static! {
    pub static ref ONLINE_SERVERS: ServerList = ServerList::default();
    pub static ref DATABASE_URL: String = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL is not set in the environment variables");
    pub static ref STEAM_KEY: String =
        std::env::var("STEAM_KEY").expect("STEAM_KEY is not set in the environment variables");
    pub static ref PORT: String = std::env::var("PORT").unwrap_or("1337".to_string());
    pub static ref HOST: String = std::env::var("HOST").unwrap_or("0.0.0.0".to_string());
    pub static ref JWT_KEY: String =
        std::env::var("JWT_KEY").expect("JWT_KEY is not set in the environment variables");
    pub static ref API_URL: String =
        std::env::var("API_URL").unwrap_or(format!("http://localhost:{}", *PORT));
    pub static ref RB: Rbatis = Rbatis::new();
}

pub const AUTHORIZED_SERVERS: [&str; 1] = ["192.168.0.13"];
