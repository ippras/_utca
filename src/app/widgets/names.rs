use egui::{Grid, Response, Ui, Widget};
use egui_l20n::UiExt as _;
use lipid::fatty_acid::{
    FattyAcid,
    display::{DisplayWithOptions as _, ID},
};

/// Names widget
pub(crate) struct NamesWidget<'a> {
    fatty_acid: &'a FattyAcid,
}

impl<'a> NamesWidget<'a> {
    pub(crate) fn new(fatty_acid: &'a FattyAcid) -> Self {
        Self { fatty_acid }
    }
}

impl Widget for NamesWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading(ui.localize("names"));
        Grid::new(ui.next_auto_id())
            .show(ui, |ui| {
                let id = self.fatty_acid.display(ID);
                if let Some(abbreviation) = ui.try_localize(&format!("{id:#}.abbreviation")) {
                    ui.label(ui.localize("abbreviation"));
                    ui.label(abbreviation);
                    ui.end_row();
                }

                if let Some(common_name) = ui.try_localize(&format!("{id:#}.common_name")) {
                    ui.label(ui.localize("common_name"));
                    ui.label(common_name);
                    ui.end_row();
                }

                if let Some(systematic_name) = ui.try_localize(&format!("{id:#}.systematic_name")) {
                    ui.label(ui.localize("systematic_name"));
                    ui.label(systematic_name);
                    ui.end_row();
                }
            })
            .response
    }
}
