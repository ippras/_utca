use crate::{app::ICON_SIZE, presets::*, utils::title};
use egui::{Id, Response, RichText, ScrollArea, Separator, Ui, Widget};
use egui_ext::LabeledSeparator;
use egui_phosphor::regular::DATABASE;
use metadata::MetaDataFrame;

/// Presets
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Presets;

impl Presets {
    fn presets(&mut self, ui: &mut Ui) {
        // IPPRAS
        ui.horizontal(|ui| {
            ui.hyperlink_to(RichText::new("IPPRAS").heading(), "https://ippras.ru");
            ui.add(Separator::default().horizontal());
        });
        ui.labeled_separator(RichText::new("519").heading());
        preset(ui, &C519_2025_04_23_1);
        preset(ui, &C519_2025_04_23_2);
        ui.labeled_separator(RichText::new("C108").heading());
        preset(ui, &C108_2025_04_23_1);
        preset(ui, &C108_2025_04_23_2);
        preset(ui, &C108_2025_04_23_3);
        ui.labeled_separator(RichText::new("C1210").heading());
        preset(ui, &C1210_2025_04_23_1);
        preset(ui, &C1210_2025_04_23_2);
        preset(ui, &C1210_2025_04_23_3);
        ui.labeled_separator(RichText::new("H626").heading());
        preset(ui, &H626_2025_04_24);
        ui.labeled_separator(RichText::new("Lobosphera").heading());
        preset(ui, &LOBOSPHERA_2025_04_24_1);
        preset(ui, &LOBOSPHERA_2025_04_24_2);
        preset(ui, &LOBOSPHERA_2025_04_24_3);
        ui.separator();
    }
}

impl Widget for Presets {
    fn ui(mut self, ui: &mut Ui) -> Response {
        ui.menu_button(RichText::new(DATABASE).size(ICON_SIZE), |ui| {
            ScrollArea::new([false, true]).show(ui, |ui| self.presets(ui));
        })
        .response
    }
}

fn preset(ui: &mut Ui, frame: &MetaDataFrame) {
    let title = title(&frame.meta, " ");
    if ui.button(format!("{DATABASE} {title}")).clicked() {
        ui.data_mut(|data| data.insert_temp(Id::new("Input"), frame.clone()));
    }
}
