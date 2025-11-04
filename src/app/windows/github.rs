use crate::{
    app::{ICON_SIZE, identifiers::DATA},
    utils::{HashedDataFrame, HashedMetaDataFrame, spawn},
};
use anyhow::{Context as _, Error, Result};
use egui::{
    Context, Id, PopupCloseBehavior, Response, RichText, ScrollArea, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_ext::{Doi as _, LabeledSeparator as _};
use egui_phosphor::regular::CLOUD_ARROW_DOWN;
use ehttp::{Request, fetch_async};
use metadata::polars::MetaDataFrame;
use std::borrow::Cow;
use tracing::{instrument, trace};
use url::Url;
use urlencoding::decode;

/// Github widget
pub struct Github;

impl Github {
    fn content(&mut self, ui: &mut Ui) {
        // IPPRAS
        ui.hyperlink_to(RichText::new("IPPRAS").heading(), "https://ippras.ru");
        ui.menu_button("Acer", |ui| {
            ui.labeled_separator(RichText::new("Acer Ginnala").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Ginnala.2025-07-08.0.0.1.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Ginnala.2025-07-08.0.0.2.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Ginnala.2025-07-08.0.0.3.utca.ron");
            ui.labeled_separator(RichText::new("Acer Pensylvanicum").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Pensylvanicum.2025-07-08.0.0.1.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Pensylvanicum.2025-07-08.0.0.2.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Pensylvanicum.2025-07-08.0.0.3.utca.ron");
            ui.labeled_separator(RichText::new("Acer Rubrum").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Rubrum.2025-07-09.0.0.1.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Rubrum.2025-07-09.0.0.2.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Rubrum.2025-07-09.0.0.3.utca.ron");
            ui.labeled_separator(RichText::new("Acer Spicatum").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Spicatum.2025-07-09.0.0.1.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Spicatum.2025-07-09.0.0.2.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Spicatum.2025-07-09.0.0.3.utca.ron");
            ui.labeled_separator(RichText::new("Acer Ukurunduense").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Ukurunduense.2025-07-08.0.0.1.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Ukurunduense.2025-07-08.0.0.2.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Acer/Acer Ukurunduense.2025-07-08.0.0.3.utca.ron");
        });
        ui.menu_button("Cedrus", |ui| {
            ui.labeled_separator(RichText::new("Cedrus").heading());
            // preset(ui, &CEDRUS_2023_05_19);
            // preset(ui, &CEDRUS_2023_05_19_1);
            // preset(ui, &CEDRUS_2023_05_19_2);
        });
        ui.menu_button("Helianthus annuus", |ui| {
            ui.labeled_separator(RichText::new("Helianthus annuus").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/HelianthusAnnuus/К-2233.25.10.29.0.0.1.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/HelianthusAnnuus/К-2233.25.10.29.0.0.2.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/HelianthusAnnuus/К-2233.25.10.29.0.0.3.utca.ron");
        });
        ui.menu_button("Microalgae", |ui| {
            ui.labeled_separator(RichText::new("C-108 (Chromochloris zofingiensis)").heading());
            // preset(ui, &C108_2025_04_23_1);
            // preset(ui, &C108_2025_04_23_2);
            // preset(ui, &C108_2025_04_23_3);
            // ui.labeled_separator(RichText::new("C-1210 (Neochlorella semenenkoi)").heading());
            // preset(ui, &C1210_2025_04_23_1);
            // preset(ui, &C1210_2025_04_23_2);
            // preset(ui, &C1210_2025_04_23_3);
            // ui.labeled_separator(RichText::new("C-1540 (Lobosphaera sp.)").heading());
            // preset(ui, &C1540_2025_04_24_1);
            // preset(ui, &C1540_2025_04_24_2);
            // preset(ui, &C1540_2025_04_24_3);
            // ui.labeled_separator(RichText::new("H-242 (Vischeria punctata)").heading());
            // preset(ui, &H242_2023_10_24_1);
            // preset(ui, &H242_2023_10_24_2);
            // ui.labeled_separator(RichText::new("H-626 (Coelastrella affinis)").heading());
            // preset(ui, &H626_2025_04_24);
            // ui.labeled_separator(RichText::new("P-519 (Porphyridium purpureum)").heading());
            // preset(ui, &P519_2025_04_23_1);
            // preset(ui, &P519_2025_04_23_2);
        });
        ui.menu_button("Sidorov (2014)", |ui| {
            ui.doi("10.1007/s11746-014-2553-8");
            ui.labeled_separator(RichText::new("Subgenus Euonymus").heading());
            ui.labeled_separator(RichText::new("Section Euonymus").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Bungeanus.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Europaeus.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Hamiltonianus.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Phellomanus.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Semiexsertus.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Sieboldianus.2014-06-19.utca.ron");
            ui.labeled_separator(RichText::new("Section Melanocarya").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Alatus.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Sacrosanctus.2014-06-19.utca.ron");
            ui.labeled_separator(RichText::new("Section Pseudovyenomus").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Pauciflorus.2014-06-19.utca.ron");
            ui.labeled_separator(RichText::new("Subgenus Kalonymus").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Latifolius.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Macropterus.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Maximowiczianus.2014-06-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2014/Euonymus Sachalinensis.2014-06-19.utca.ron");
        });
        ui.menu_button("Sidorov (2025)", |ui| {
            ui.doi("10.3390/plants14040612");
            // https://raw.githubusercontent.com/ippras/_utca/main/src/presets/Sidorov2025/Lunaria Rediviva.2024-01-24.1.1.1.utca.ron
            ui.labeled_separator(RichText::new("Lunaria Rediviva").heading());
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_1_1);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_1_2);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_2_1);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_2_2);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_3_1);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_1_3_2);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_1_1);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_1_2);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_2_1);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_2_2_2);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_1_1);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_1_2);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_2_1);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_2_2);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_3_1);
            // preset(ui, &LUNARIA_REDIVIVA_2024_01_24_3_3_2);
        });
        ui.separator();
        // Third party
        ui.heading("Third party");
        ui.menu_button("Reske (1997)", |ui| {
            ui.doi("10.1007/s11746-997-0016-1");
            ui.labeled_separator(RichText::new("Soybean").heading());
            // preset(ui, &SOYBEAN_SEED_COMMODITY);
            ui.labeled_separator(RichText::new("Sunflower").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Reske1997/Sunﬂower Seed (Commodity).1997-08-01.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Reske1997/Sunﬂower Seed (High linoleic).1997-08-01.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Reske1997/Sunﬂower Seed (High oleic).1997-08-01.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Reske1997/Sunﬂower Seed (High palmitic, high linoleic).1997-08-01.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Reske1997/Sunﬂower Seed (High palmitic, high oleic).1997-08-01.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Reske1997/Sunﬂower Seed (High stearic, high oleic).1997-08-01.utca.ron");
        });
        ui.menu_button("Martinez-Force (2004)", |ui| {
            ui.doi("10.1016/j.ab.2004.07.019");
            ui.labeled_separator(RichText::new("Hazelnut").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Martinez-Force2004/Hazelnut.2025-08-19.utca.ron");
            ui.labeled_separator(RichText::new("Olive").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Martinez-Force2004/Olive.2025-08-19.utca.ron");
            ui.labeled_separator(RichText::new("Rice").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Martinez-Force2004/Rice.2025-08-19.utca.ron");
            ui.labeled_separator(RichText::new("Soybean").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Martinez-Force2004/Soybean.2025-08-19.utca.ron");
            ui.labeled_separator(RichText::new("Sunflower").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Martinez-Force2004/Sunflower CAS-3.2025-08-19.utca.ron");
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Martinez-Force2004/Sunflower RHA-274.2025-08-19.utca.ron");
            ui.labeled_separator(RichText::new("Walnut").heading());
            let _ = preset(ui, "https://raw.githubusercontent.com/ippras/_utca/main/src/presets/ThirdParty/Martinez-Force2004/Walnut.2025-08-19.utca.ron");
        });
    }
}

impl Widget for Github {
    fn ui(mut self, ui: &mut Ui) -> Response {
        MenuButton::new(RichText::new(CLOUD_ARROW_DOWN).size(ICON_SIZE))
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ScrollArea::new([false, true]).show(ui, |ui| self.content(ui));
            })
            .0
    }
}

/// Preset
#[instrument(skip(ui), err)]
fn preset(ui: &mut Ui, input: &str) -> Result<()> {
    let url = Url::parse(input)?;
    let (name, date, version) = parse(&url)?;
    if ui.button(format!("{name} {date} {version}")).clicked() {
        load(ui.ctx(), url);
    }
    Ok(())
}

/// Parse preset url
fn parse<'a>(url: &'a Url) -> Result<(Cow<'a, str>, &'a str, &'a str)> {
    let segment = url
        .path_segments()
        .context("Preset get path segments")?
        .last()
        .context("Preset get last path segment")?;
    let input = segment.trim_end_matches(".utca.ron");
    let (name, input) = input.split_once(".").context("Preset parse name")?;
    let (date, version) = input.split_once(".").unwrap_or((input, ""));
    Ok((decode(name)?, date, version))
}

fn load(ctx: &Context, url: Url) {
    let ctx = ctx.clone();
    let _ = spawn(async move {
        if let Ok(frame) = try_load(&url).await {
            trace!(?frame);
            ctx.data_mut(|data| data.insert_temp(Id::new(DATA), frame));
        }
    });
}

#[instrument(err)]
async fn try_load(url: &Url) -> Result<HashedMetaDataFrame> {
    let request = Request::get(url);
    let response = fetch_async(request).await.map_err(Error::msg)?;
    let text = response.text().context("Try load get response text")?;
    trace!(?text);
    let frame = ron::de::from_str::<MetaDataFrame>(text)?;
    Ok(MetaDataFrame {
        meta: frame.meta,
        data: HashedDataFrame::new(frame.data)?,
    })
}
