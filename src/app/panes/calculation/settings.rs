use super::ID_SOURCE;
use crate::{app::MAX_PRECISION, utils::egui::table::TableState};
use egui::{Context, Grid, Id, Slider, Ui};
use egui_l20n::{ResponseExt, UiExt as _};
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static SETTINGS: LazyLock<Id> = LazyLock::new(|| Id::new(ID_SOURCE).with("Settings"));
static TABLE_STATE: LazyLock<Id> = LazyLock::new(|| Id::new(ID_SOURCE).with("TableState"));
static WINDOWS: LazyLock<Id> = LazyLock::new(|| Id::new(ID_SOURCE).with("Windows"));

/// Calculation settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub percent: bool,
    pub precision: usize,
    pub table: TableSettings,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            percent: true,
            precision: 1,
            table: TableSettings::new(),
        }
    }
}

impl Settings {
    pub fn show(&mut self, ui: &mut Ui) {
        Grid::new(ID_SOURCE).show(ui, |ui| {
            // Precision
            ui.label(ui.localize("Precision"))
                .on_hover_localized("Precision.hover");
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // Percent
            ui.label(ui.localize("Percent"))
                .on_hover_localized("Percent.hover");
            ui.checkbox(&mut self.percent, "");
            ui.end_row();

            // // Sticky
            // ui.label(ui.localize("StickyColumns"))
            //     .on_hover_localized("StickyColumns.hover");
            // ui.add(Slider::new(&mut self.sticky_columns, 0..=14));
            // ui.end_row();

            // // Truncate
            // ui.label(ui.localize("TruncateHeaders"))
            //     .on_hover_localized("TruncateHeaders.hover");
            // ui.checkbox(&mut self.truncate_headers, "");
            // ui.end_row();
        });
    }
}

impl Settings {
    pub fn load(ctx: &Context) -> Self {
        ctx.data_mut(|data| {
            let settings = data.get_persisted_mut_or_insert_with(*SETTINGS, || Self::new());
            settings.clone()
        })
    }

    pub fn store(self, ctx: &Context) {
        ctx.data_mut(|data| {
            data.insert_persisted(*SETTINGS, self);
        });
    }
}

/// Calculation table settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TableSettings {
    pub resizable: bool,
    pub sticky_columns: usize,
    pub truncate_headers: bool,
    pub state: TableState,
}

impl TableSettings {
    pub fn new() -> Self {
        Self {
            resizable: false,
            sticky_columns: 0,
            truncate_headers: false,
            state: TableState::new(*TABLE_STATE),
        }
    }
}

/// Calculation windows state
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_christie: bool,
    pub open_indices: bool,
    pub open_parameters: bool,
    pub open_settings: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
            open_christie: false,
            open_indices: false,
            open_parameters: false,
            open_settings: false,
        }
    }
}

impl Windows {
    pub fn load(ctx: &Context) -> Self {
        ctx.data_mut(|data| {
            data.get_persisted_mut_or_insert_with(*WINDOWS, || Self::new())
                .clone()
        })
    }

    pub fn store(self, ctx: &Context) {
        ctx.data_mut(|data| {
            data.insert_persisted(*WINDOWS, self);
        });
    }
}

// /// Table state
// #[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
// pub struct Table {
//     pub reset: bool,
//     pub resizable: bool,
//     pub sticky_columns: usize,
//     pub truncate_headers: bool,
// }

// impl Table {
//     pub fn new() -> Self {
//         Self {
//             reset: false,
//             resizable: false,
//             sticky_columns: 0,
//             truncate_headers: false,
//         }
//     }
// }

// impl Table {
//     pub(crate) fn show(&mut self, ui: &mut Ui) {
//         // // Precision
//         // ui.label(ui.localize("Precision"))
//         //     .on_hover_localized("Precision.hover");
//         // ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
//         // ui.end_row();

//         // // Percent
//         // ui.label(ui.localize("Percent"))
//         //     .on_hover_localized("Percent.hover");
//         // ui.checkbox(&mut self.percent, "");
//         // ui.end_row();

//         // Sticky
//         ui.label(ui.localize("StickyColumns"))
//             .on_hover_localized("StickyColumns.hover");
//         ui.add(Slider::new(&mut self.sticky_columns, 0..=14));
//         ui.end_row();

//         // Truncate
//         ui.label(ui.localize("TruncateHeaders"))
//             .on_hover_localized("TruncateHeaders.hover");
//         ui.checkbox(&mut self.truncate_headers, "");
//         ui.end_row();
//     }
// }

// impl Table {
//     pub fn load(ctx: &Context) -> Self {
//         ctx.data_mut(|data| {
//             data.get_persisted_mut_or_insert_with(*TABLE, || Self::new())
//                 .clone()
//         })
//     }

//     pub fn store(self, ctx: &Context) {
//         ctx.data_mut(|data| {
//             data.insert_persisted(*TABLE, self);
//         });
//     }

//     pub fn reset(ctx: &Context) {
//         ctx.data_mut(|data| {
//             data.insert_persisted(*TABLE, Self::new());
//         })
//     }
// }
