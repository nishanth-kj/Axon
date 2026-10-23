use super::node::ProxyNode;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct DetailsPage {
    pub nodes: Vec<ProxyNode>,
    pub total: i64,
    pub page: i64,
    pub size: i64,
    pub total_pages: i64,
}
