use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct NewRelease<'a> {
    pub download: &'a str,
    pub signature: &'a str,
    pub nightly: bool,
}
