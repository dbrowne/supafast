use serde::{Deserialize, Serialize};

// Generic request type
#[derive(Debug, Clone, Deserialize)]
pub struct WorkRequest {
    pub id: String,
    // Add your request fields here
}

// Lean response type - minimal allocations
#[derive(Debug, Serialize)]
pub struct WorkResponse {
    pub success: bool,
    #[serde(skip_serializing_if = "String::is_empty")]
    pub id: String,
    pub status: ResponseStatus,
}

#[derive(Debug, Serialize, Clone, Copy)]
#[serde(rename_all = "snake_case")]
pub enum ResponseStatus {
    Completed,
    Failed,
    Invalid,
    ConnectionError,
}

impl WorkResponse {
    #[inline]
    pub fn success(id: String) -> Self {
        Self {
            success: true,
            id,
            status: ResponseStatus::Completed,
        }
    }

    #[inline]
    pub fn failure(id: String, status: ResponseStatus) -> Self {
        Self {
            success: false,
            id,
            status,
        }
    }
}
