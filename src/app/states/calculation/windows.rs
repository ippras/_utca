use serde::{Deserialize, Serialize};

/// Windows
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_correlations: bool,
    pub open_properties: bool,
    pub open_biodiesel_properties: bool,
    pub open_settings: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            open_correlations: false,
            open_properties: false,
            open_biodiesel_properties: false,
            open_settings: false,
        }
    }
}
