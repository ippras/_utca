use serde::{Deserialize, Serialize};

/// Windows
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_correlations: bool,
    pub open_sum: bool,
    pub open_biodiesel_sum: bool,
    pub open_settings: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            open_correlations: false,
            open_sum: false,
            open_biodiesel_sum: false,
            open_settings: false,
        }
    }
}
