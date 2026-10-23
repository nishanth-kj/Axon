use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RegistryRequest {
    pub status: u8,
    pub node_id: Option<String>,
    pub port: Option<u16>,
    pub verification_port: Option<u16>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DetailsRequest {
    #[serde(default = "default_page")]
    pub page: i64,
    #[serde(default = "default_size")]
    pub size: i64,
    #[serde(default = "default_sort")]
    pub sort_by: String,
    #[serde(default = "default_order")]
    pub sort_order: String,
}

impl Default for DetailsRequest {
    fn default() -> Self {
        Self {
            page: default_page(),
            size: default_size(),
            sort_by: default_sort(),
            sort_order: default_order(),
        }
    }
}

fn default_page() -> i64 {
    1
}
fn default_size() -> i64 {
    10
}
fn default_sort() -> String {
    "registered_at".to_string()
}
fn default_order() -> String {
    "desc".to_string()
}
