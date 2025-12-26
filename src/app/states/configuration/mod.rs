pub(crate) use self::{settings::Settings, windows::Windows};

use egui::{Context, Id};
use serde::{Deserialize, Serialize};

pub(crate) const ID_SOURCE: &str = "Configuration";

/// Configuration state
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    #[serde(skip)]
    pub(crate) event: Event,
    pub(crate) settings: Settings,
    pub(crate) windows: Windows,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            event: Event::new(),
            settings: Settings::new(),
            windows: Windows::new(),
        }
    }
}

impl State {
    pub(crate) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| data.get_persisted_mut_or_insert_with(id, Self::new).clone())
    }

    pub(crate) fn remove(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.remove::<Self>(id);
        });
    }

    pub(crate) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.insert_persisted(id, self);
        });
    }
}

/// Event
#[derive(Clone, Copy, Debug, Default, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Event {
    pub(crate) add_table_row: bool,
    pub(crate) delete_table_row: Option<usize>,
    pub(crate) reset_table_state: bool,
    pub(crate) up_table_row: Option<usize>,
}

impl Event {
    pub(crate) fn new() -> Self {
        Self {
            add_table_row: false,
            delete_table_row: None,
            reset_table_state: false,
            up_table_row: None,
        }
    }
}

mod settings;
mod windows;
