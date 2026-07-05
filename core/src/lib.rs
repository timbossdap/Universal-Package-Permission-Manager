#[derive(Debug)]
pub enum PermissionCategory {
    Filesystem,
    Hardware,
    Network,
    Desktop,
    System,
}
#[derive(Debug)]
pub struct Permission {
    pub category: PermissionCategory,
    pub description: String,
    pub source_mechanism: String,
    pub raw: String,
}
#[derive(Debug)]
pub struct AppProfile {
    pub app_id: String,
    pub permissions: Vec<Permission>,
}
