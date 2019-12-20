#[macro_use]
extern crate failure;
#[macro_use]
extern crate serde_derive;

mod models;

use std::vec::Vec;

use failure::Error;
use futures::{Future, Stream};
use hyper::{Body, Method, Request, StatusCode};
use hyper::client::{Client, HttpConnector};
use hyper_tls::HttpsConnector;

use crate::models::{App, Category, NewRelease};

fn get_https_client() -> Client<HttpsConnector<HttpConnector>, Body> {
    let https = HttpsConnector::new(4).unwrap();
    Client::builder().build::<_, Body>(https)
}

pub fn get_categories() -> impl Future<Item = Vec<Category>, Error = Error> + Send {
    let uri = "https://apps.nextcloud.com/api/v1/categories.json"
        .parse()
        .unwrap();
    let client = get_https_client();

    client
        .get(uri)
        .and_then(|res| {
            res.into_body().concat2().and_then(move |body| {
                let apps: Vec<Category> = serde_json::from_slice(&body).unwrap();
                Ok(apps)
            })
        })
        .map_err(|err| err.into())
}

pub fn get_apps_and_releases(
    version: &String,
) -> impl Future<Item = Vec<App>, Error = Error> + Send {
    let raw_uri = format!(
        "https://apps.nextcloud.com/api/v1/platform/{}/apps.json",
        version
    );
    let uri = raw_uri.parse().unwrap();
    let https = hyper_tls::HttpsConnector::new(4).unwrap();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    client
        .get(uri)
        .and_then(|res| {
            res.into_body().concat2().and_then(move |body| {
                let apps: Vec<App> = serde_json::from_slice(&body).unwrap();
                Ok(apps)
            })
        })
        .map_err(|err| Error::from(err))
}

pub fn publish_app(
    url: &String,
    is_nightly: bool,
    signature: &String,
    api_token: &String,
) -> impl Future<Item = (), Error = Error> + Send {
    let release = NewRelease {
        download: url.to_owned(),
        signature: signature.to_owned(),
        nightly: is_nightly,
    };
    let release_json = serde_json::to_string(&release).unwrap();
    let https = hyper_tls::HttpsConnector::new(4).unwrap();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);

    let mut req = Request::builder();
    req.method(Method::POST);
    req.uri("https://apps.nextcloud.com/api/v1/apps/releases");
    req.header(
        hyper::header::AUTHORIZATION,
        hyper::header::HeaderValue::from_str(&format!("Token {}", api_token)).unwrap(),
    );
    req.header(
        hyper::header::CONTENT_TYPE,
        hyper::header::HeaderValue::from_static("application/json"),
    );
    req.header(
        hyper::header::CONTENT_LENGTH,
        hyper::header::HeaderValue::from_str(&release_json.len().to_string()).unwrap(),
    );

    client
        .request(req.body(release_json.into()).unwrap())
        .map_err(|err| Error::from(err))
        .and_then(|res| match res.status() {
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
        })
}
