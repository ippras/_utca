use crate::{app::ICON_SIZE, presets::*};
use egui::{
    Id, PopupCloseBehavior, Response, RichText, ScrollArea, Separator, Ui, Widget,
    containers::menu::{MenuConfig, SubMenuButton},
};
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
        SubMenuButton::new("Acer")
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ui.labeled_separator(RichText::new("Acer Ginnala").heading());
                preset(ui, &ACER_GINNALA_2025_07_08_1);
                preset(ui, &ACER_GINNALA_2025_07_08_2);
                preset(ui, &ACER_GINNALA_2025_07_08_3);
                ui.labeled_separator(RichText::new("Acer Pensylvanicum").heading());
                preset(ui, &ACER_PENSYLVANICUM_2025_07_08_1);
                preset(ui, &ACER_PENSYLVANICUM_2025_07_08_2);
                preset(ui, &ACER_PENSYLVANICUM_2025_07_08_3);
                ui.labeled_separator(RichText::new("Acer Rubrum").heading());
                preset(ui, &ACER_RUBRUM_2025_07_09_1);
                preset(ui, &ACER_RUBRUM_2025_07_09_2);
                preset(ui, &ACER_RUBRUM_2025_07_09_3);
                ui.labeled_separator(RichText::new("Acer Spicatum").heading());
                preset(ui, &ACER_SPICATUM_2025_07_09_1);
                preset(ui, &ACER_SPICATUM_2025_07_09_2);
                preset(ui, &ACER_SPICATUM_2025_07_09_3);
                ui.labeled_separator(RichText::new("Acer Ukurunduense").heading());
                preset(ui, &ACER_UKURUNDUENSE_2025_07_08_1);
                preset(ui, &ACER_UKURUNDUENSE_2025_07_08_2);
                preset(ui, &ACER_UKURUNDUENSE_2025_07_08_3);
            });
        SubMenuButton::new("Cedrus")
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ui.labeled_separator(RichText::new("Cedrus").heading());
                preset(ui, &CEDRUS_2023_05_19);
            });
        SubMenuButton::new("Lunaria")
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ui.labeled_separator(RichText::new("Lunaria Rediviva").heading());
                preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_1);
                preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_2);
                preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_3);
                preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_1);
                preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_2);
                preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_1);
                preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_2);
                preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_3);
            });
        SubMenuButton::new("Microalgae")
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
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
            });
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
    let title = frame.meta.format(" ");
    if ui.button(format!("{DATABASE} {title}")).clicked() {
        ui.data_mut(|data| data.insert_temp(Id::new("Input"), frame.clone()));
    }
}
