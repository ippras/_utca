use crate::app::MAX_PRECISION;
use egui::{Grid, Slider, Ui};
use egui_l20n::UiExt as _;
use serde::{Deserialize, Serialize};

/// Configuration settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: usize,

    pub(crate) resizable: bool,
    pub(crate) editable: bool,
    pub(crate) precision: usize,
    pub(crate) sticky: usize,
    pub(crate) truncate: bool,

    pub(crate) names: bool,
    pub(crate) properties: bool,
}

impl Settings {
    pub(crate) const fn new() -> Self {
        Self {
            index: 0,
            resizable: false,
            editable: false,
            precision: 2,
            sticky: 0,
            truncate: false,
            names: true,
            properties: true,
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        Grid::new("Configuration").show(ui, |ui| {
            // Precision
            let mut response = ui.label(ui.localize("settings-precision"));
            response |= ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("settings-precision.description"));
            });
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Properties
            let mut response = ui.label(ui.localize("settings-properties"));
            response |= ui.checkbox(&mut self.properties, "");
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("settings-properties.description"));
            });
            ui.end_row();

            // Names
            let mut response = ui.label(ui.localize("settings-names"));
            response |= ui.checkbox(&mut self.names, "");
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("settings-names.description"));
            });
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}
