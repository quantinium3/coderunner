use std::net::SocketAddrV4;
use comphub::config::config;
use comphub::error::ServerError;
use comphub::routes::app_router;
use comphub::utils::init_tracing;

#[tokio::main]
async fn main() -> Result<(), ServerError> {
    init_tracing();
    let app_config = config().await;

    let addr = format!("{}:{}", app_config.server_host(), app_config.server_port());
    let socket_addr: SocketAddrV4 = addr.parse()?;

    let app = app_router();

    let listener = tokio::net::TcpListener::bind(socket_addr).await?;
    tracing::info!("server listening on: {}", socket_addr);
    axum::serve(listener, app).await?;
    Ok(())
}
