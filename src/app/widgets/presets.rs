use crate::{
    app::{ICON_SIZE, identifiers::DATA},
    presets::*,
};
use egui::{
    Id, PopupCloseBehavior, Response, RichText, ScrollArea, Ui, Widget,
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
        ui.hyperlink_to(RichText::new("IPPRAS").heading(), "https://ippras.ru");
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
        ui.separator();
        // Third party
        ui.heading("Third party");
        SubMenuButton::new("Reske 1997")
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ui.hyperlink_to(
                    RichText::new("10.1007/s11746-997-0016-1").heading(),
                    "https://doi.org/10.1007/s11746-997-0016-1",
                );
                ui.labeled_separator(RichText::new("Soybean").heading());
                preset(ui, &SOYBEAN_SEED_COMMODITY);
                ui.labeled_separator(RichText::new("Sunflower").heading());
                preset(ui, &SUNFLOWER_SEED_COMMODITY);
            });
        SubMenuButton::new("Martinez-Force 2004")
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ui.hyperlink_to(
                    RichText::new("10.1016/j.ab.2004.07.019").heading(),
                    "https://doi.org/10.1016/j.ab.2004.07.019",
                );
                ui.labeled_separator(RichText::new("Hazelnut").heading());
                preset(ui, &HAZELNUT);
                ui.labeled_separator(RichText::new("Olive").heading());
                preset(ui, &OLIVE);
                ui.labeled_separator(RichText::new("Rice").heading());
                preset(ui, &RICE);
                ui.labeled_separator(RichText::new("Soybean").heading());
                preset(ui, &SOYBEAN);
                ui.labeled_separator(RichText::new("Sunflower").heading());
                preset(ui, &SUNFLOWER_CAS3);
                preset(ui, &SUNFLOWER_RHA274);
                ui.labeled_separator(RichText::new("Walnut").heading());
                preset(ui, &WALNUT);
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
        ui.data_mut(|data| data.insert_temp(Id::new(DATA), frame.clone()));
    }
}
