#[derive(Debug, Serialize)]
pub struct NewRelease {
    pub download: String,
    pub signature: String,
    pub nightly: bool,
}
