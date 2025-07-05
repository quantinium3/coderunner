use axum::{Router, routing::IntoMakeService};
use comphub::{config::config, routes::app_router};
use std::net::SocketAddr;
use tokio::sync::OnceCell;

static TEST_SERVICE: OnceCell<IntoMakeService<Router>> = OnceCell::const_new();

async fn get_test_service() -> &'static IntoMakeService<Router> {
    TEST_SERVICE
        .get_or_init(|| async { app_router().into_make_service() })
        .await
}

#[derive(Debug)]
pub struct TestClient {
    pub client: reqwest::Client,
    pub addr: String,
}

impl TestClient {
    pub async fn new() -> Self {
        let app_config = config().await;
        let addr = format!("{}:{}", app_config.server_host(), app_config.server_port());
        let socket_addr: SocketAddr = addr.parse().unwrap();
        let service = get_test_service().await;
        let listener = tokio::net::TcpListener::bind(socket_addr).await.unwrap();

        tokio::spawn(async move {
            axum::serve(listener, service.clone()).await.unwrap();
        });

        TestClient {
            client: reqwest::Client::new(),
            addr,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("http://{}{}", self.addr, path)
    }
}
