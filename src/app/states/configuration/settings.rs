use super::ID_SOURCE;
use crate::app::{MAX_PRECISION, states::ColumnFilter};
use egui::{
    Grid, PopupCloseBehavior, Slider, Ui,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::FUNNEL;
use serde::{Deserialize, Serialize};

/// Configuration settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: usize,

    pub(crate) column_filter: ColumnFilter,
    pub(crate) edit_table: bool,
    pub(crate) precision: usize,
    pub(crate) resize_table: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate_headers: bool,
    // Hover
    pub(crate) hover_names: bool,
    pub(crate) hover_properties: bool,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            index: 0,

            column_filter: ColumnFilter::new(),
            edit_table: false,
            precision: 1,
            resize_table: false,
            sticky_columns: 0,
            truncate_headers: false,
            // Hover
            hover_names: true,
            hover_properties: true,
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
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
            ui.add(Slider::new(&mut self.sticky_columns, 0..=5));
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
                    self.column_filter.show(ui);
                });
            ui.end_row();

            ui.heading("Hover");
            ui.separator();
            ui.end_row();

            // Names
            ui.label(ui.localize("Names")).on_hover_ui(|ui| {
                ui.label(ui.localize("Names.hover"));
            });
            ui.checkbox(&mut self.hover_names, "");
            ui.end_row();

            // Properties
            ui.label(ui.localize("Properties")).on_hover_ui(|ui| {
                ui.label(ui.localize("Properties.hover"));
            });
            ui.checkbox(&mut self.hover_properties, "");
            ui.end_row();
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
