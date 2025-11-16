use super::ID_SOURCE;
use crate::app::{MAX_PRECISION, states::ColumnFilter};
use egui::{
    Grid, PopupCloseBehavior, Slider, Ui, Widget as _,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{BOOKMARK, FUNNEL};
use serde::{Deserialize, Serialize};

/// Configuration settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: usize,

    pub(crate) column_filter: ColumnFilter,
    pub(crate) edit_table: bool,
    pub(crate) float_precision: usize,
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

            edit_table: false,
            resize_table: false,

            float_precision: 0,
            sticky_columns: 0,
            truncate_headers: false,
            // Filter
            column_filter: ColumnFilter::new(),
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

            self.float_precision(ui);
            ui.end_row();
            self.sticky_columns(ui);
            ui.end_row();
            self.truncate_headers(ui);
            ui.end_row();

            // Filter
            self.column_filter(ui);
            ui.end_row();

            ui.heading("Hover");
            ui.separator();
            ui.end_row();

            self.hover_names(ui);
            ui.end_row();
            self.hover_properties(ui);
            ui.end_row();
        });
    }

    // Precision
    fn float_precision(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Precision")).on_hover_ui(|ui| {
            ui.label(ui.localize("Precision.hover"));
        });
        ui.horizontal(|ui| {
            ui.add(Slider::new(&mut self.float_precision, 0..=MAX_PRECISION));
            if ui.button((BOOKMARK, "3")).clicked() {
                self.float_precision = 3;
            }
        });
    }

    /// Sticky
    fn sticky_columns(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("StickyColumns")).on_hover_ui(|ui| {
            ui.label(ui.localize("StickyColumns.hover"));
        });
        Slider::new(&mut self.sticky_columns, 0..=8).ui(ui);
    }

    /// Truncate
    fn truncate_headers(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("TruncateHeaders")).on_hover_ui(|ui| {
            ui.label(ui.localize("TruncateHeaders.hover"));
        });
        ui.checkbox(&mut self.truncate_headers, ());
    }

    /// Filter
    fn column_filter(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("FilterTableColumns"))
            .on_hover_ui(|ui| {
                ui.label(ui.localize("FilterTableColumns.hover"));
            });
        MenuButton::new(FUNNEL)
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                self.column_filter.show(ui);
            });
    }

    /// Names
    fn hover_names(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Names")).on_hover_ui(|ui| {
            ui.label(ui.localize("Names.hover"));
        });
        ui.checkbox(&mut self.hover_names, "");
    }

    /// Properties
    fn hover_properties(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Properties")).on_hover_ui(|ui| {
            ui.label(ui.localize("Properties.hover"));
        });
        ui.checkbox(&mut self.hover_properties, "");
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
