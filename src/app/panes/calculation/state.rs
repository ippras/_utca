
use super::ID_SOURCE;
use crate::{app::MAX_PRECISION, utils::egui::state::Table};
use egui::{
    containers::menu::{MenuButton, MenuConfig},
    Context, Grid, Id, PopupCloseBehavior, Slider, Ui,
};
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::FUNNEL;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

static SETTINGS: LazyLock<Id> = LazyLock::new(|| Id::new(ID_SOURCE).with("Settings"));
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
        Grid::new(ui.auto_id_with(ID_SOURCE)).show(ui, |ui| {
            // Precision
            ui.label(ui.localize("Precision")).on_hover_ui(|ui| {
                ui.label(ui.localize("Precision.hover"));
            });
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // Percent
            ui.label(ui.localize("Percent")).on_hover_ui(|ui| {
                ui.label(ui.localize("Percent.hover"));
            });
            ui.checkbox(&mut self.percent, "");
            ui.end_row();

            ui.heading("Table");
            ui.separator();
            ui.end_row();

            // Resizable
            ui.label(ui.localize("Resizable")).on_hover_ui(|ui| {
                ui.label(ui.localize("Resizable.hover"));
            });
            ui.checkbox(&mut self.table.resizable, "");
            ui.end_row();

            // Sticky
            ui.label(ui.localize("StickyColumns")).on_hover_ui(|ui| {
                ui.label(ui.localize("StickyColumns.hover"));
            });
            ui.add(Slider::new(&mut self.table.sticky_columns, 0..=14));
            ui.end_row();

            // Truncate
            ui.label(ui.localize("TruncateHeaders")).on_hover_ui(|ui| {
                ui.label(ui.localize("TruncateHeaders.hover"));
            });
            ui.checkbox(&mut self.table.truncate_headers, "");
            ui.end_row();

            // Filter
            ui.label(ui.localize("FilterTableColumns"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("FilterTableColumns.hover"));
                });
            MenuButton::new(FUNNEL)
                .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
                .ui(ui, |ui| {
                    self.table.filter.show(ui);
                });
            ui.end_row();
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
    pub reset_state: bool,
    pub resizable: bool,
    pub sticky_columns: usize,
    pub truncate_headers: bool,
    pub filter: Table,
}

impl TableSettings {
    pub fn new() -> Self {
        Self {
            reset_state: false,
            resizable: false,
            sticky_columns: 0,
            truncate_headers: false,
            filter: Table::new(SETTINGS.with("Filter")),
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
