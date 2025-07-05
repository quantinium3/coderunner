#[cfg(test)]
mod health_test {
    use serde_json::Value;

    use super::*;
    use crate::utils::TestClient;

    #[tokio::test]
    async fn test_health_route() {
        let client = TestClient::new().await;

        let response = client
            .client
            .get(&client.url("/healthz"))
            .send()
            .await
            .unwrap();

        assert_eq!(response.status(), 200);
        let body = response.text().await.unwrap();
        let json: Value = serde_json::from_str(&body).unwrap();
        assert_eq!(json["status"], "Ok");
    }
}
