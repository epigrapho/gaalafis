use lfs_info_server::server::{
    config::ServerConfig, injected_services::from_server_config, run_server::run_server,
};
use std::env;
use std::net::SocketAddr;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    let config = ServerConfig::default().parse_args(args).parse_env();
    let services = from_server_config(&config);
    let app = run_server(&config, Arc::new(services));
    let addr = SocketAddr::from(([0, 0, 0, 0], 3000));
    tracing::debug!("listening on {}", addr);
    axum::Server::bind(&addr)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
