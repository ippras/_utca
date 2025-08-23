use super::ID_SOURCE;
use crate::{app::MAX_PRECISION, utils::egui::state::Table};
use egui::{
    Context, Grid, Id, PopupCloseBehavior, Slider, Ui,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROWS_CLOCKWISE, FUNNEL};
use serde::{Deserialize, Serialize};
use std::{hash::Hash, sync::LazyLock};

static SETTINGS: LazyLock<Id> = LazyLock::new(|| Id::new(ID_SOURCE).with("Settings"));
static WINDOWS: LazyLock<Id> = LazyLock::new(|| Id::new(ID_SOURCE).with("Windows"));

// /// Configuration settings
// #[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
// pub(crate) struct Settings {
//     pub(crate) add_table_row: bool,
//     pub(crate) delete_table_row: Option<usize>,
//     pub(crate) reset_table_state: bool,
// }

// impl Settings {
//     pub(crate) fn new() -> Self {
//         Self {
//             add_table_row: false,
//             delete_table_row: None,
//             reset_table_state: false,
//         }
//     }
// }

/// Configuration settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Settings {
    pub precision: usize,

    pub add_row: bool,
    pub delete_row: Option<usize>,
    pub edit_table: bool,
    pub filter_columns: Table,
    pub reset_state: bool,
    pub resize_table: bool,
    pub sticky_columns: usize,
    pub truncate_headers: bool,

    pub show_names: bool,
    pub show_properties: bool,
}

impl Settings {
    pub fn new() -> Self {
        Self {
            precision: 1,
            add_row: false,
            delete_row: None,
            edit_table: false,
            filter_columns: Table::new(SETTINGS.with("Filter")),
            reset_state: false,
            resize_table: false,
            sticky_columns: 0,
            truncate_headers: false,
            show_names: true,
            show_properties: true,
        }
    }
}

impl Settings {
    pub fn show(&mut self, ui: &mut Ui) {
        Grid::new(ui.auto_id_with(ID_SOURCE)).show(ui, |ui| {
            ui.visuals_mut().button_frame = true;

            // Precision
            ui.label(ui.localize("Precision")).on_hover_ui(|ui| {
                ui.label(ui.localize("Precision.hover"));
            });
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            ui.heading("Table");
            ui.separator();
            ui.end_row();

            // Reset
            ui.label(ui.localize("ResetTable")).on_hover_ui(|ui| {
                ui.label(ui.localize("ResetTable.hover"));
            });
            ui.toggle_value(&mut self.reset_state, ARROWS_CLOCKWISE);
            ui.end_row();

            // Resizable
            ui.label(ui.localize("Resizable")).on_hover_ui(|ui| {
                ui.label(ui.localize("Resizable.hover"));
            });
            ui.checkbox(&mut self.resize_table, "");
            ui.end_row();

            // Sticky
            ui.label(ui.localize("StickyColumns")).on_hover_ui(|ui| {
                ui.label(ui.localize("StickyColumns.hover"));
            });
            ui.add(Slider::new(&mut self.sticky_columns, 0..=6));
            ui.end_row();

            // Truncate
            ui.label(ui.localize("TruncateHeaders")).on_hover_ui(|ui| {
                ui.label(ui.localize("TruncateHeaders.hover"));
            });
            ui.checkbox(&mut self.truncate_headers, "");
            ui.end_row();

            // Filter
            ui.label(ui.localize("FilterTableColumns"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("FilterTableColumns.hover"));
                });
            MenuButton::new(FUNNEL)
                .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
                .ui(ui, |ui| {
                    self.filter_columns.show(ui);
                });
            ui.end_row();

            ui.heading("Hover");
            ui.separator();
            ui.end_row();

            // Names
            let mut response = ui.label(ui.localize("Names"));
            response |= ui.checkbox(&mut self.show_names, "");
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("Names.hover"));
            });
            ui.end_row();

            // Properties
            let mut response = ui.label(ui.localize("Properties"));
            response |= ui.checkbox(&mut self.show_properties, "");
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("Properties.hover"));
            });
            ui.end_row();
        });
    }
}

impl Settings {
    pub fn load(ctx: &Context, id: impl Hash) -> Self {
        ctx.data_mut(|data| {
            let settings = data.get_persisted_mut_or_insert_with(SETTINGS.with(id), || Self::new());
            settings.clone()
        })
    }

    pub fn store(self, ctx: &Context, id: impl Hash) {
        ctx.data_mut(|data| {
            data.insert_persisted(SETTINGS.with(id), self);
        });
    }
}

/// Configuration windows
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub struct Windows {
    pub open_parameters: bool,
    pub open_settings: bool,
}

impl Windows {
    pub fn new() -> Self {
        Self {
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
