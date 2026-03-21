use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default, rename_all = "camelCase")]
pub struct User {
    pub username: String,
    pub password: String,
    pub salt: String,
    pub token: String,
    pub last_login_at: i64,
    pub created_at: i64,
    pub enable_webdav: bool,
    pub token_map: Option<HashMap<String, i64>>,
    pub enable_local_store: bool,
    pub is_admin: bool,
}
