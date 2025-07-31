use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct QueryParams {
    pub page: Option<u64>,
    pub per_page: Option<u64>,
    pub sort_by: Option<String>,
    pub sort_order: Option<String>,
    pub search: Option<String>,
}
