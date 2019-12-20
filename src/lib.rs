#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

mod models;

use failure::Error;
use hyper::client::{Client, HttpConnector};
use hyper::{Body, Method, Request, StatusCode};
use hyper_tls::HttpsConnector;

use crate::models::NewRelease;

fn get_https_client() -> Client<HttpsConnector<HttpConnector>, Body> {
    let https = HttpsConnector::new();
    Client::builder().build::<_, Body>(https)
}

pub async fn publish_app(
    url: &String,
    is_nightly: bool,
    signature: &String,
    api_token: &String,
) -> Result<(), Error> {
    let release = NewRelease {
        download: url.to_owned(),
        signature: signature.to_owned(),
        nightly: is_nightly,
    };
    let release_json = serde_json::to_string(&release).unwrap();
    let client = get_https_client();

    let req = Request::builder()
        .method(Method::POST)
        .uri("https://apps.nextcloud.com/api/v1/apps/releases")
        .header(
            hyper::header::AUTHORIZATION,
            hyper::header::HeaderValue::from_str(&format!("Token {}", api_token)).unwrap(),
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
        StatusCode::BAD_REQUEST => Err(format_err!(
            "client-error occurred, got HTTP status {}",
            res.status()
        )),
        _ => Err(format_err!(
            "error uploading release, got HTTP status {}",
            res.status()
        )),
    }
}
