use crate::app::MAX_PRECISION;
use egui::{Grid, Slider, Ui};
use egui_l20n::UiExt as _;
use serde::{Deserialize, Serialize};

/// Configuration parameters
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Parameters {
    pub(crate) index: usize,
}

impl Parameters {
    pub(crate) fn new() -> Self {
        Self { index: 0 }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        Grid::new("Configuration").show(ui, |ui| {
            // // Precision
            // let mut response = ui.label(ui.localize("Precision"));
            // response |= ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            // response.on_hover_ui(|ui| {
            //     ui.label(ui.localize("Precision.hover"));
            // });
            // ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();
        });
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
    }
}
