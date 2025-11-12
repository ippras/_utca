use serde::{Deserialize, Serialize};

/// Windows
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_christie: bool,
    pub open_correlations: bool,
    pub open_indices: bool,
    pub open_settings: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            open_christie: false,
            open_correlations: false,
            open_indices: false,
            open_settings: false,
        }
    }
}
