use std::collections::HashMap;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Event {
    pub id: u32,
    pub relative_uri: String,
    pub method: String,
    pub headers: HashMap<String, String>,
    pub body_base64: String,
    pub created_at: DateTime<Utc>,
}