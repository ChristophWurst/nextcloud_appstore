#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;

mod models;

use std::vec::Vec;

use failure::Error;
use futures::Stream;
use futures::future::{err, Future};
use hyper::{Body, Method, Request, StatusCode};
use hyper::client::{Client, HttpConnector};
use hyper_tls::HttpsConnector;

use models::{App, Category, NewRelease};

fn get_https_client() -> Client<HttpsConnector<HttpConnector>, Body> {
    let https = HttpsConnector::new(4).unwrap();
    Client::builder().build::<_, Body>(https)
}

pub fn get_categories() -> Box<Future<Item = Vec<Category>, Error = Error>> {
    let uri = match "https://apps.nextcloud.com/api/v1/categories.json".parse() {
        Ok(u) => u,
        Err(e) => return Box::new(err(Error::from(e))),
    };
    let client = get_https_client();

    let work = client
        .get(uri)
        .and_then(|res| {
            res.into_body().concat2().and_then(move |body| {
                let apps: Vec<Category> = serde_json::from_slice(&body).unwrap();
                Ok(apps)
            })
        })
        .map_err(|err| err.into());

    Box::new(work)
}

pub fn get_apps_and_releases(version: &String) -> Box<Future<Item = Vec<App>, Error = Error>> {
    let raw_uri = format!(
        "https://apps.nextcloud.com/api/v1/platform/{}/apps.json",
        version
    );
    let uri = match raw_uri.parse() {
        Ok(u) => u,
        Err(e) => return Box::new(err(Error::from(e))),
    };
    let https = hyper_tls::HttpsConnector::new(4).unwrap();
    let client = hyper::Client::builder().build::<_, hyper::Body>(https);
    let work = client
        .get(uri)
        .and_then(|res| {
            res.into_body().concat2().and_then(move |body| {
                let apps: Vec<App> = serde_json::from_slice(&body).unwrap();
                Ok(apps)
            })
        })
        .map_err(|err| Error::from(err));

    Box::new(work)
}

pub fn publish_app(
    url: &String,
    is_nightly: bool,
    signature: &String,
    api_token: &String,
) -> Box<Future<Item = (), Error = Error>> {
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
    let work = client
        .request(req.body(release_json.into()).unwrap())
        .map_err(|err| Error::from(err))
        .and_then(|res| match res.status() {
            StatusCode::OK => Ok(()),
            StatusCode::CREATED => Ok(()),
            _ => Err(format_err!(
                "error uploading release, got HTTP status {}",
                res.status()
            )),
        });

    Box::new(work)
}
