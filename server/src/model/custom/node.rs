use serde::Serialize;

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct ProxyNode {
    pub node_id: String,
    pub ip: String,
    pub port: u16,
    pub verification_port: u16,
    pub status: String,
    #[serde(skip)]
    pub secret_key: String,
    pub last_seen: String,
    pub registered_at: String,
    pub failure_count: u32,
}

#[derive(Clone, Debug, Serialize, PartialEq, Eq)]
pub struct PublicNode {
    pub node_id: String,
    pub ip: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum HeartbeatError {
    UnknownToken,
    IpMismatch,
}
