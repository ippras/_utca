use serde::{Deserialize, Serialize};

/// Configuration windows
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_settings: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            open_settings: false,
        }
    }
}
