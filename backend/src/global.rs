use once_cell::sync::Lazy;

use crate::ws::server::ServerList;

pub static ONLINE_SERVERS: Lazy<ServerList> = Lazy::new(ServerList::default);

pub const AUTHORIZED_SERVERS: [&str; 1] = ["192.168.0.13"];
