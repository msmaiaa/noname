use once_cell::sync::Lazy;

use crate::server::ServerList;

pub static ONLINE_SERVERS: Lazy<ServerList> = Lazy::new(ServerList::default);
