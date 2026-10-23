mod node_service;

pub use node_service::{secret_for, NodeService, RegisterOutcome};

use crate::model::custom::config::ServiceConfig;

pub struct AppState {
    pub nodes: NodeService,
    pub master_secret: String,
}

impl AppState {
    pub fn new(config: ServiceConfig) -> Self {
        let master_secret = config.master_secret.clone();
        Self {
            nodes: NodeService::new(config),
            master_secret,
        }
    }
}
