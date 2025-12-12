use crate::{
    app::{ICON_SIZE, identifiers::DATA},
    utils::HashedMetaDataFrame,
};
use egui::{
    Id, PopupCloseBehavior, Response, RichText, ScrollArea, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_ext::{Doi as _, LabeledSeparator as _};
use egui_phosphor::regular::DATABASE;
use metadata::egui::MetadataWidget;

/// Presets
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Presets;

impl Presets {
    fn content(&mut self, ui: &mut Ui) {
        // IPPRAS
        ui.hyperlink_to(RichText::new("IPPRAS").heading(), "https://ippras.ru");
        ui.menu_button("Acer", |ui| {
            use crate::presets::ippras::*;

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
        ui.menu_button("Cedrus", |ui| {
            use crate::presets::ippras::*;

            ui.labeled_separator(RichText::new("Cedrus").heading());
            preset(ui, &CEDRUS_2023_05_19);
            preset(ui, &CEDRUS_2023_05_19_1);
            preset(ui, &CEDRUS_2023_05_19_2);
        });
        ui.menu_button("Helianthus annuus", |ui| {
            use crate::presets::ippras::*;

            ui.labeled_separator(RichText::new("Helianthus annuus").heading());
            preset(ui, &HELIANTHUS_ANNUUS_2025_10_29_1);
            preset(ui, &HELIANTHUS_ANNUUS_2025_10_29_2);
            preset(ui, &HELIANTHUS_ANNUUS_2025_10_29_3);
        });
        ui.menu_button("Microalgae", |ui| {
            use crate::presets::ippras::*;

            ui.labeled_separator(RichText::new("C-108 (Chromochloris zofingiensis)").heading());
            preset(ui, &C108_2025_04_23_1);
            preset(ui, &C108_2025_04_23_2);
            preset(ui, &C108_2025_04_23_3);
            ui.labeled_separator(RichText::new("C-1210 (Neochlorella semenenkoi)").heading());
            preset(ui, &C1210_2025_04_23_1);
            preset(ui, &C1210_2025_04_23_2);
            preset(ui, &C1210_2025_04_23_3);
            ui.labeled_separator(RichText::new("C-1540 (Lobosphaera sp.)").heading());
            preset(ui, &C1540_2025_04_24_1);
            preset(ui, &C1540_2025_04_24_2);
            preset(ui, &C1540_2025_04_24_3);
            ui.labeled_separator(RichText::new("H-242 (Vischeria punctata)").heading());
            preset(ui, &H242_2023_10_24_1);
            preset(ui, &H242_2023_10_24_2);
            ui.labeled_separator(RichText::new("H-626 (Coelastrella affinis)").heading());
            preset(ui, &H626_2025_04_24);
            ui.labeled_separator(RichText::new("P-519 (Porphyridium purpureum)").heading());
            preset(ui, &P519_2025_04_23_1);
            preset(ui, &P519_2025_04_23_2);
        });
        ui.menu_button("Sidorov (2014)", |ui| {
            use crate::presets::sidorov2014::*;

            ui.doi("10.1007/s11746-014-2553-8");
            ui.labeled_separator(RichText::new("Subgenus Euonymus").heading());
            ui.labeled_separator(RichText::new("Section Euonymus").heading());
            preset(ui, &EUONYMUS_BUNGEANUS);
            preset(ui, &EUONYMUS_EUROPAEUS);
            preset(ui, &EUONYMUS_HAMILTONIANUS);
            preset(ui, &EUONYMUS_PHELLOMANUS);
            preset(ui, &EUONYMUS_SEMIEXSERTUS);
            preset(ui, &EUONYMUS_SIEBOLDIANUS);
            ui.labeled_separator(RichText::new("Section Melanocarya").heading());
            preset(ui, &EUONYMUS_ALATUS);
            preset(ui, &EUONYMUS_SACROSANCTUS);
            ui.labeled_separator(RichText::new("Section Pseudovyenomus").heading());
            preset(ui, &EUONYMUS_PAUCIFLORUS);
            ui.labeled_separator(RichText::new("Subgenus Kalonymus").heading());
            preset(ui, &EUONYMUS_LATIFOLIUS);
            preset(ui, &EUONYMUS_MACROPTERUS);
            preset(ui, &EUONYMUS_MAXIMOWICZIANUS);
            preset(ui, &EUONYMUS_SACHALINENSIS);
        });
        ui.menu_button("Sidorov (2025)", |ui| {
            use crate::presets::sidorov2025::*;

            ui.doi("10.3390/plants14040612");
            ui.labeled_separator(RichText::new("Lunaria Rediviva").heading());
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_1_1);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_1_2);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_2_1);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_2_2);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_3_1);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_3_2);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_1_1);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_1_2);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_2_1);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_2_2);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_1_1);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_1_2);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_2_1);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_2_2);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_3_1);
            preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_3_2);
        });
        ui.separator();
        // Third party
        ui.heading("Third party");
        ui.menu_button("Reske (1997)", |ui| {
            use crate::presets::reske1997::*;

            ui.doi("10.1007/s11746-997-0016-1");
            ui.labeled_separator(RichText::new("Soybean").heading());
            // preset(ui, &SOYBEAN_SEED_COMMODITY);
            ui.labeled_separator(RichText::new("Sunflower").heading());
            preset(ui, &SUNFLOWER_SEED_COMMODITY);
            preset(ui, &SUNFLOWER_SEED_HIGH_LINOLEIC);
            preset(ui, &SUNFLOWER_SEED_HIGH_OLEIC);
            preset(ui, &SUNFLOWER_SEED_HIGH_PALMITIC_HIGH_LINOLEIC);
            preset(ui, &SUNFLOWER_SEED_HIGH_PALMITIC_HIGH_OLEIC);
            preset(ui, &SUNFLOWER_SEED_HIGH_STEARIC_HIGH_OLEIC);
        });
        ui.menu_button("Martinez-Force (2004)", |ui| {
            use crate::presets::martínez_force2004::*;

            ui.doi("10.1016/j.ab.2004.07.019");
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
        MenuButton::new(RichText::new(DATABASE).size(ICON_SIZE))
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| self.content(ui));
            })
            .0
    }
}

fn preset(ui: &mut Ui, frame: &HashedMetaDataFrame) {
    let title = frame.meta.format(" ");
    let response = ui.button(format!("{DATABASE} {title}")).on_hover_ui(|ui| {
        MetadataWidget::new(&frame.meta).show(ui);
    });
    if response.clicked() {
        ui.data_mut(|data| data.insert_temp(Id::new(DATA), frame.clone()));
    }
}
