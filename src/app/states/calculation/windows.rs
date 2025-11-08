use serde::{Deserialize, Serialize};

/// Windows
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_indices: bool,
    pub open_metrics: bool,
    pub open_settings: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            open_indices: false,
            open_metrics: false,
            open_settings: false,
        }
    }
}
