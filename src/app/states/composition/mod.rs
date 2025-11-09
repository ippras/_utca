pub(crate) use self::{
    settings::{
        Discriminants, Method, Order, Selection, Settings, Sort, View,
        composition::{
            COMPOSITIONS, Composition, ECN_MONO, ECN_STEREO, MASS_MONO, MASS_STEREO, SPECIES_MONO,
            SPECIES_POSITIONAL, SPECIES_STEREO, TYPE_MONO, TYPE_POSITIONAL, TYPE_STEREO,
            UNSATURATION_MONO, UNSATURATION_STEREO,
        },
        filter::{Filter, FilterWidget},
    },
    windows::Windows,
};

use egui::{Context, Grid, Id, Ui};
use serde::{Deserialize, Serialize};

pub(crate) const ID_SOURCE: &str = "Composition";

/// State
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) reset_table_state: bool,
    pub(crate) settings: Settings,
    pub(crate) windows: Windows,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            reset_table_state: false,
            settings: Settings::new(),
            windows: Windows::new(),
        }
    }
}

impl State {
    pub(crate) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| {
            data.get_persisted_mut_or_insert_with(id, || Self::new())
                .clone()
        })
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

mod settings;
mod windows;
