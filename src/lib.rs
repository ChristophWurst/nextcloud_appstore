mod models;

use reqwest::StatusCode;
use thiserror::Error;

use crate::models::NewRelease;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AppstoreError {
    #[error("Failed to send request to appstore: {0}")]
    RequestFailed(#[from] reqwest::Error),
    #[error("Client-error occurred, got HTTP status {0}")]
    BadRequest(StatusCode),
    #[error("Error uploading release, got HTTP status {0}")]
    Unknown(StatusCode),
}

const APPSTORE_RELEASES_URL: &str = "https://apps.nextcloud.com/api/v1/apps/releases";

pub async fn publish_app(
    url: &str,
    is_nightly: bool,
    signature: &str,
    api_token: &str,
) -> Result<(), AppstoreError> {
    publish_app_to(APPSTORE_RELEASES_URL, url, is_nightly, signature, api_token).await
}

async fn publish_app_to(
    appstore_url: &str,
    url: &str,
    is_nightly: bool,
    signature: &str,
    api_token: &str,
) -> Result<(), AppstoreError> {
    let release = NewRelease {
        download: url,
        signature,
        nightly: is_nightly,
    };
    let client = reqwest::Client::new();

    let res = client
        .post(appstore_url)
        .header(reqwest::header::AUTHORIZATION, format!("Token {}", api_token))
        .json(&release)
        .send()
        .await?;

    match res.status() {
        StatusCode::OK => Ok(()),
        StatusCode::CREATED => Ok(()),
        StatusCode::BAD_REQUEST => Err(AppstoreError::BadRequest(res.status())),
        _ => Err(AppstoreError::Unknown(res.status())),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{body_json, header, method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    #[tokio::test]
    async fn sends_expected_request_and_handles_created() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/"))
            .and(header("authorization", "Token secret-token"))
            .and(body_json(serde_json::json!({
                "download": "https://example.com/app.tar.gz",
                "signature": "sig",
                "nightly": false,
            })))
            .respond_with(ResponseTemplate::new(201))
            .expect(1)
            .mount(&server)
            .await;

        let result = publish_app_to(
            &server.uri(),
            "https://example.com/app.tar.gz",
            false,
            "sig",
            "secret-token",
        )
        .await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handles_ok_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(200))
            .mount(&server)
            .await;

        let result = publish_app_to(&server.uri(), "u", true, "s", "t").await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn handles_bad_request() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(400))
            .mount(&server)
            .await;

        let result = publish_app_to(&server.uri(), "u", true, "s", "t").await;

        assert!(matches!(result, Err(AppstoreError::BadRequest(_))));
    }

    #[tokio::test]
    async fn handles_unexpected_status() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let result = publish_app_to(&server.uri(), "u", true, "s", "t").await;

        assert!(matches!(result, Err(AppstoreError::Unknown(_))));
    }
}
