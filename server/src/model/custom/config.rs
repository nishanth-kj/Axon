#[derive(Clone, Debug)]
pub struct ServiceConfig {
    pub master_secret: String,
    pub debug: bool,
    pub failure_limit: u32,
}
