use crate::model::custom::config::ServiceConfig;
use crate::model::custom::node::{HeartbeatError, ProxyNode, PublicNode};
use crate::model::custom::page::DetailsPage;
use crate::model::request::{DetailsRequest, RegistryRequest};
use crate::utils::logger;
use crate::utils::net::{bracket_host, is_local_ip};
use crate::utils::time::now_iso;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::Duration;

type HmacSha256 = Hmac<Sha256>;

pub struct NodeService {
    nodes: Mutex<HashMap<String, ProxyNode>>,
    config: ServiceConfig,
    http: reqwest::Client,
}

pub enum RegisterOutcome {
    InvalidPayload,
    MissingNodeId,
    InvalidStatus,
    Unregistered,
    Pending(ProxyNode),
}

impl NodeService {
    pub fn new(config: ServiceConfig) -> Self {
        let http = reqwest::Client::builder()
            .timeout(Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            nodes: Mutex::new(HashMap::new()),
            config,
            http,
        }
    }

    pub fn apply_registration(&self, body: &RegistryRequest, ip: &str) -> RegisterOutcome {
        let port = body.port.unwrap_or(0);
        if ip == "unknown" || port == 0 {
            return RegisterOutcome::InvalidPayload;
        }
        if body.status == 0 {
            let Some(node_id) = body.node_id.as_deref().filter(|id| !id.is_empty()) else {
                return RegisterOutcome::MissingNodeId;
            };
            self.unregister(node_id);
            return RegisterOutcome::Unregistered;
        }
        if body.status != 1 {
            return RegisterOutcome::InvalidStatus;
        }
        let node_id = body
            .node_id
            .clone()
            .filter(|id| !id.is_empty())
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());
        let verification_port = body.verification_port.filter(|port| *port != 0).unwrap_or(port);
        RegisterOutcome::Pending(self.register(node_id, ip.to_string(), port, verification_port))
    }

    pub fn register(
        &self,
        node_id: String,
        ip: String,
        port: u16,
        verification_port: u16,
    ) -> ProxyNode {
        let mut nodes = self.nodes.lock().expect("registry lock");
        let stale: Vec<String> = nodes
            .values()
            .filter(|node| node.ip == ip && node.port == port)
            .map(|node| node.node_id.clone())
            .collect();
        for old_id in stale {
            logger::info(format!("Replacing previous node {old_id} at {ip}:{port}"));
            nodes.remove(&old_id);
        }

        let now = now_iso();
        let node = ProxyNode {
            secret_key: secret_for(&self.config.master_secret, &node_id),
            node_id,
            ip,
            port,
            verification_port,
            status: "PENDING".to_string(),
            last_seen: now.clone(),
            registered_at: now,
            failure_count: 0,
        };
        nodes.insert(node.node_id.clone(), node.clone());
        node
    }

    pub fn unregister(&self, node_id: &str) {
        self.nodes.lock().expect("registry lock").remove(node_id);
        logger::info(format!("Unregistered node {node_id}"));
    }

    pub fn heartbeat(&self, secret_key: &str, client_ip: &str) -> Result<(), HeartbeatError> {
        let mut nodes = self.nodes.lock().expect("registry lock");
        let node_id = nodes
            .values()
            .find(|node| node.secret_key == secret_key)
            .map(|node| node.node_id.clone());
        let Some(node_id) = node_id else {
            return Err(HeartbeatError::UnknownToken);
        };
        let node = nodes.get_mut(&node_id).expect("node present");
        let same_local = is_local_ip(&node.ip) && is_local_ip(client_ip);
        if !same_local && client_ip != node.ip {
            return Err(HeartbeatError::IpMismatch);
        }
        node.status = "ACTIVE".to_string();
        node.last_seen = now_iso();
        node.failure_count = 0;
        Ok(())
    }

    pub fn active_nodes(&self) -> Vec<PublicNode> {
        let debug = self.config.debug;
        self.nodes
            .lock()
            .expect("registry lock")
            .values()
            .filter(|node| node.status == "ACTIVE" || (debug && node.status == "PENDING"))
            .map(|node| PublicNode {
                node_id: node.node_id.clone(),
                ip: node.ip.clone(),
                port: node.port,
                status: debug.then(|| node.status.clone()),
            })
            .collect()
    }

    pub fn list_details(&self, query: &DetailsRequest) -> DetailsPage {
        let page = query.page.max(1);
        let size = query.size.clamp(1, 100);
        let mut nodes = self
            .nodes
            .lock()
            .expect("registry lock")
            .values()
            .cloned()
            .collect::<Vec<_>>();
        let descending = query.sort_order.eq_ignore_ascii_case("desc");
        nodes.sort_by(|left, right| compare_nodes(left, right, &query.sort_by, descending));
        let total = nodes.len() as i64;
        let total_pages = if total == 0 { 0 } else { (total + size - 1) / size };
        let start = ((page - 1) * size).max(0) as usize;
        let nodes = nodes.into_iter().skip(start).take(size as usize).collect();
        DetailsPage {
            nodes,
            total,
            page,
            size,
            total_pages,
        }
    }

    pub fn spawn_verification(&self, node: &ProxyNode) {
        let client = self.http.clone();
        let ip = node.ip.clone();
        let port = node.verification_port;
        let secret_key = node.secret_key.clone();
        let node_id = node.node_id.clone();
        actix_web::rt::spawn(async move {
            let host = bracket_host(&ip);
            let url = format!("http://{host}:{port}/api/v1/verify");
            match client
                .post(url)
                .json(&serde_json::json!({ "secretKey": secret_key }))
                .send()
                .await
            {
                Ok(response) if response.status().is_success() => {
                    logger::info(format!("Verification callback accepted by node {node_id}"));
                }
                Ok(response) => {
                    logger::warn(format!(
                        "Verification callback to node {node_id} returned {}",
                        response.status()
                    ));
                }
                Err(error) => {
                    logger::warn(format!("Verification callback to node {node_id} failed: {error}"));
                }
            }
        });
    }

    pub async fn probe_health(&self) {
        let snapshot: Vec<ProxyNode> = self
            .nodes
            .lock()
            .expect("registry lock")
            .values()
            .cloned()
            .collect();

        for node in snapshot {
            let host = bracket_host(&node.ip);
            let url = format!("http://{host}:{}/api/v1/health", node.verification_port);
            let ok = matches!(
                self.http.get(url).send().await,
                Ok(response) if response.status().is_success()
            );
            let mut nodes = self.nodes.lock().expect("registry lock");
            let remove_id = {
                let Some(current) = nodes.get_mut(&node.node_id) else {
                    continue;
                };
                if ok {
                    current.last_seen = now_iso();
                    current.failure_count = 0;
                    None
                } else {
                    current.failure_count += 1;
                    logger::warn(format!(
                        "Health check failed for {} ({}/{})",
                        current.node_id, current.failure_count, self.config.failure_limit
                    ));
                    (current.failure_count >= self.config.failure_limit).then(|| current.node_id.clone())
                }
            };
            if let Some(id) = remove_id {
                nodes.remove(&id);
                logger::info(format!("Removed node {id} after repeated health-check failures"));
            }
        }
    }
}

pub fn secret_for(master_secret: &str, node_id: &str) -> String {
    let mut mac = HmacSha256::new_from_slice(master_secret.as_bytes())
        .expect("HMAC accepts any key length");
    mac.update(node_id.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

fn compare_nodes(left: &ProxyNode, right: &ProxyNode, field: &str, descending: bool) -> Ordering {
    let ordering = match field {
        "port" => left.port.cmp(&right.port),
        "verification_port" => left.verification_port.cmp(&right.verification_port),
        "failure_count" => left.failure_count.cmp(&right.failure_count),
        "node_id" => left.node_id.cmp(&right.node_id),
        "ip" => left.ip.cmp(&right.ip),
        "status" => left.status.cmp(&right.status),
        "last_seen" => left.last_seen.cmp(&right.last_seen),
        _ => left.registered_at.cmp(&right.registered_at),
    };
    if descending {
        ordering.reverse()
    } else {
        ordering
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn service() -> NodeService {
        NodeService::new(ServiceConfig {
            master_secret: "Nishanth".to_string(),
            debug: false,
            failure_limit: 3,
        })
    }

    #[test]
    fn registration_heartbeats_and_discovery() {
        let service = service();
        let node = service.register("node-1".into(), "203.0.113.8".into(), 8081, 8080);
        assert_eq!(node.status, "PENDING");
        assert_eq!(node.secret_key, secret_for("Nishanth", "node-1"));
        assert!(service.active_nodes().is_empty());

        assert_eq!(
            service.heartbeat("not-a-token", "203.0.113.8"),
            Err(HeartbeatError::UnknownToken)
        );
        assert_eq!(
            service.heartbeat(&node.secret_key, "198.51.100.4"),
            Err(HeartbeatError::IpMismatch)
        );
        assert_eq!(service.heartbeat(&node.secret_key, "203.0.113.8"), Ok(()));

        let active = service.active_nodes();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].node_id, "node-1");
        assert_eq!(active[0].port, 8081);
        assert!(active[0].status.is_none());
    }

    #[test]
    fn new_registration_replaces_the_same_endpoint() {
        let service = service();
        service.register("old".into(), "203.0.113.8".into(), 8081, 8080);
        service.register("new".into(), "203.0.113.8".into(), 8081, 8080);
        let page = service.list_details(&DetailsRequest::default());
        assert_eq!(page.total, 1);
        assert_eq!(page.nodes[0].node_id, "new");
    }

    #[test]
    fn local_heartbeat_allows_loopback_aliases() {
        let service = service();
        let node = service.register("local".into(), "127.0.0.1".into(), 8081, 8080);
        assert_eq!(service.heartbeat(&node.secret_key, "::1"), Ok(()));
    }

    #[test]
    fn details_hide_the_secret_and_paginate() {
        let service = service();
        service.register("b-node".into(), "203.0.113.8".into(), 8081, 8080);
        service.register("a-node".into(), "203.0.113.9".into(), 8082, 8080);
        let page = service.list_details(&DetailsRequest {
            page: 1,
            size: 1,
            sort_by: "node_id".into(),
            sort_order: "asc".into(),
        });
        assert_eq!(page.total, 2);
        assert_eq!(page.total_pages, 2);
        assert_eq!(page.nodes.len(), 1);
        assert_eq!(page.nodes[0].node_id, "a-node");
        let json = serde_json::to_value(&page.nodes[0]).unwrap();
        assert!(json.get("secret_key").is_none());
    }

    #[test]
    fn unregister_removes_the_node() {
        let service = service();
        let outcome = service.apply_registration(
            &RegistryRequest {
                status: 1,
                node_id: Some("node-1".into()),
                port: Some(8081),
                verification_port: Some(8080),
            },
            "203.0.113.8",
        );
        assert!(matches!(outcome, RegisterOutcome::Pending(_)));
        let removed = service.apply_registration(
            &RegistryRequest {
                status: 0,
                node_id: Some("node-1".into()),
                port: Some(8081),
                verification_port: None,
            },
            "203.0.113.8",
        );
        assert!(matches!(removed, RegisterOutcome::Unregistered));
        assert_eq!(service.list_details(&DetailsRequest::default()).total, 0);
    }

    #[test]
    fn bad_status_and_missing_id_are_rejected() {
        let service = service();
        let missing = service.apply_registration(
            &RegistryRequest {
                status: 0,
                node_id: None,
                port: Some(8081),
                verification_port: None,
            },
            "203.0.113.8",
        );
        assert!(matches!(missing, RegisterOutcome::MissingNodeId));
        let status = service.apply_registration(
            &RegistryRequest {
                status: 9,
                node_id: None,
                port: Some(8081),
                verification_port: None,
            },
            "203.0.113.8",
        );
        assert!(matches!(status, RegisterOutcome::InvalidStatus));
        let payload = service.apply_registration(
            &RegistryRequest {
                status: 1,
                node_id: None,
                port: Some(0),
                verification_port: None,
            },
            "unknown",
        );
        assert!(matches!(payload, RegisterOutcome::InvalidPayload));
    }
}
