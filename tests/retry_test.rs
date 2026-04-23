use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::Duration;

use wiremock::MockServer;
use wiremock::matchers::method;
use wiremock::{Mock, Request, Respond, ResponseTemplate};

use repology::RepologyClient;

struct SequentialResponder {
    call_count: AtomicUsize,
    responses: Vec<ResponseTemplate>,
}

impl SequentialResponder {
    fn new(responses: Vec<ResponseTemplate>) -> Self {
        Self {
            call_count: AtomicUsize::new(0),
            responses,
        }
    }
}

impl Respond for SequentialResponder {
    fn respond(&self, _request: &Request) -> ResponseTemplate {
        let idx = self.call_count.fetch_add(1, Ordering::SeqCst);
        if idx < self.responses.len() {
            self.responses[idx].clone()
        } else {
            self.responses.last().unwrap().clone()
        }
    }
}

fn test_client(base_url: &str) -> RepologyClient {
    RepologyClient::builder()
        .base_url(base_url)
        .rate_limit(Duration::ZERO)
        .min_backoff(Duration::from_millis(10))
        .max_backoff(Duration::from_millis(50))
        .build()
        .unwrap()
}

#[tokio::test]
async fn retries_on_500_then_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(SequentialResponder::new(vec![
            ResponseTemplate::new(500),
            ResponseTemplate::new(500),
            ResponseTemplate::new(200).set_body_string("[]"),
        ]))
        .expect(3)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let packages = client.project("test").await.unwrap();
    assert!(packages.is_empty());
}

#[tokio::test]
async fn retries_on_429_then_succeeds() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(SequentialResponder::new(vec![
            ResponseTemplate::new(429),
            ResponseTemplate::new(200).set_body_string("[]"),
        ]))
        .expect(2)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let packages = client.project("test").await.unwrap();
    assert!(packages.is_empty());
}

#[tokio::test]
async fn exhausts_retries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .expect(4) // 1 initial + 3 retries
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client.project("test").await;

    assert!(
        matches!(result, Err(repology::Error::Api { ref status, .. }) if status.as_u16() == 500),
        "expected Api error with 500, got: {result:?}"
    );
}

#[tokio::test]
async fn no_retry_on_404() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(404))
        .expect(1) // only 1 attempt, no retries
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client.project("test").await;

    assert!(
        matches!(result, Err(repology::Error::Api { ref status, .. }) if status.as_u16() == 404),
    );
}

#[tokio::test]
async fn no_retry_on_deserialization_error() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(200).set_body_string("not json"))
        .expect(1)
        .mount(&server)
        .await;

    let client = test_client(&server.uri());
    let result = client.project("test").await;

    assert!(matches!(result, Err(repology::Error::Deserialize { .. })));
}

#[tokio::test]
async fn retries_disabled_with_zero_max_retries() {
    let server = MockServer::start().await;

    Mock::given(method("GET"))
        .respond_with(ResponseTemplate::new(500))
        .expect(1)
        .mount(&server)
        .await;

    let client = RepologyClient::builder()
        .base_url(server.uri())
        .rate_limit(Duration::ZERO)
        .max_retries(0_usize)
        .build()
        .unwrap();

    let result = client.project("test").await;
    assert!(matches!(result, Err(repology::Error::Api { .. })));
}
