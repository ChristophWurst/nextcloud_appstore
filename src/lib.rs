#[macro_use]
extern crate failure;
extern crate futures;
extern crate hyper;
extern crate hyper_tls;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate tokio_core;

mod models;

use std::vec::Vec;

use failure::Error;
use futures::Stream;
use futures::future::{err, Future};
use hyper::{Client, Method, Request, StatusCode};
use hyper::header::{Authorization, ContentLength, ContentType};
use hyper_tls::HttpsConnector;
use tokio_core::reactor::Handle;

use models::{App, Category, NewRelease};

pub fn get_categories(handle: &Handle) -> Box<Future<Item = Vec<Category>, Error = Error>> {
    let uri = match "https://apps.nextcloud.com/api/v1/categories.json".parse() {
        Ok(u) => u,
        Err(e) => return Box::new(err(Error::from(e))),
    };
    let client = Client::configure()
        .connector(HttpsConnector::new(4, handle).unwrap())
        .build(handle);
    let work = client
        .get(uri)
        .and_then(|res| {
                      res.body().concat2().and_then(move |body| {
                let apps: Vec<Category> = serde_json::from_slice(&body).unwrap();
                Ok(apps)
            })
                  })
        .map_err(|err| err.into());

    Box::new(work)
}

pub fn get_apps_and_releases(handle: &Handle,
                             version: &String)
                             -> Box<Future<Item = Vec<App>, Error = Error>> {
    let raw_uri = format!("https://apps.nextcloud.com/api/v1/platform/{}/apps.json",
                          version);
    let uri = match raw_uri.parse() {
        Ok(u) => u,
        Err(e) => return Box::new(err(Error::from(e))),
    };
    let client = Client::configure()
        .connector(HttpsConnector::new(4, handle).unwrap())
        .build(handle);
    let work =
        client
            .get(uri)
            .and_then(|res| {
                          res.body().concat2().and_then(move |body| {
                let apps: Vec<App> = serde_json::from_slice(&body).unwrap();
                Ok(apps)
            })
                      })
            .map_err(|err| Error::from(err));

    Box::new(work)
}

pub fn publish_app(handle: &Handle,
                   url: &String,
                   is_nightly: bool,
                   signature: &String,
                   api_token: &String)
                   -> Box<Future<Item = (), Error = Error>> {
    let uri = match "https://apps.nextcloud.com/api/v1/apps/releases".parse() {
        Ok(u) => u,
        Err(e) => return Box::new(err(Error::from(e))),
    };
    let release = NewRelease {
        download: url.to_owned(),
        signature: signature.to_owned(),
        nightly: is_nightly,
    };
    let release_json = serde_json::to_string(&release).unwrap();
    let client = Client::configure()
        .connector(HttpsConnector::new(4, handle).unwrap())
        .build(handle);
    let mut req = Request::new(Method::Post, uri);
    req.headers_mut()
        .set(Authorization(format!("Token {}", api_token)));
    req.headers_mut().set(ContentType::json());
    req.headers_mut()
        .set(ContentLength(release_json.len() as u64));
    req.set_body(release_json);
    let work = client
        .request(req)
        .map_err(|err| Error::from(err))
        .and_then(|res| match res.status() {
                      StatusCode::Ok => Ok(()),
                      StatusCode::Created => Ok(()),
                      _ => {
                          Err(format_err!("error uploading release, got HTTP status {}",
                                          res.status()))
                      }
                  });

    Box::new(work)
}
