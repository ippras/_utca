use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) open_christie_window: bool,
    pub(crate) open_settings_window: bool,
    pub(crate) reset_table_state: bool,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            open_christie_window: false,
            open_settings_window: false,
            reset_table_state: false,
        }
    }
}
