use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) add_table_row: bool,
    pub(crate) delete_table_row: Option<usize>,
    pub(crate) open_settings_window: bool,
    pub(crate) reset_table_state: bool,
}

impl State {
    pub(crate) const fn new() -> Self {
        Self {
            add_table_row: false,
            delete_table_row: None,
            open_settings_window: false,
            reset_table_state: false,
        }
    }
}
