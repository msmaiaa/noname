use once_cell::sync::Lazy;
use rbatis::Rbatis;

use crate::ws::server::ServerList;

pub static ONLINE_SERVERS: Lazy<ServerList> = Lazy::new(ServerList::default);
pub static DATABASE_URL: Lazy<String> = Lazy::new(|| {
    std::env::var("DATABASE_URL").expect("DATABASE_URL is not set in the environment variables")
});

pub static RB: Lazy<Rbatis> = Lazy::new(Rbatis::new);
pub const AUTHORIZED_SERVERS: [&str; 1] = ["192.168.0.13"];
