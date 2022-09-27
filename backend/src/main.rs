use salvo::prelude::*;
pub mod global;
pub mod routes;
pub mod server;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();
    let router = Router::new()
        .push(Router::with_path("/servers").get(routes::server::get_servers))
        .push(Router::with_path("ws/server").handle(server::on_server_connection));
    tracing::info!("Listening at http://127.0.0.1:1337");
    Server::new(TcpListener::bind("0.0.0.0:1337"))
        .serve(router)
        .await;
}
