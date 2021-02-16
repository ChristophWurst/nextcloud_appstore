mod models;

use hyper::client::{Client, HttpConnector};
use hyper::{Body, Method, Request, StatusCode};
use hyper_rustls::HttpsConnector;
use thiserror::Error;

use crate::models::NewRelease;
use hyper::header::InvalidHeaderValue;

#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AppstoreError {
    #[error("Failed to send request to appstore: {0}")]
    RequestFailed(#[from] hyper::Error),
    #[error("Client-error occurred, got HTTP status {0}")]
    BadRequest(StatusCode),
    #[error("Error uploading release, got HTTP status {0}")]
    Unknown(StatusCode),
    #[error("Error invalid token provided, Only visible ASCII characters (32-127) are permitted")]
    InvalidToken(#[from] InvalidHeaderValue),
}

fn get_https_client() -> Client<HttpsConnector<HttpConnector>, Body> {
    let https = HttpsConnector::with_webpki_roots();
    Client::builder().build::<_, Body>(https)
}

pub async fn publish_app(
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
    let release_json = serde_json::to_string(&release).unwrap();
    let client = get_https_client();

    let req = Request::builder()
        .method(Method::POST)
        .uri("https://apps.nextcloud.com/api/v1/apps/releases")
        .header(
            hyper::header::AUTHORIZATION,
            hyper::header::HeaderValue::from_str(&format!("Token {}", api_token))?,
        )
        .header(
            hyper::header::CONTENT_TYPE,
            hyper::header::HeaderValue::from_static("application/json"),
        )
        .header(
            hyper::header::CONTENT_LENGTH,
            hyper::header::HeaderValue::from_str(&release_json.len().to_string()).unwrap(),
        );

    let res = client
        .request(req.body(release_json.into()).unwrap())
        .await?;

    match res.status() {
        StatusCode::OK => Ok(()),
        StatusCode::CREATED => Ok(()),
        StatusCode::BAD_REQUEST => Err(AppstoreError::BadRequest(res.status())),
        _ => Err(AppstoreError::Unknown(res.status())),
    }
}
