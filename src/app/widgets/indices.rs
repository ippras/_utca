use crate::asset;
use egui::{Grid, InnerResponse, Response, Ui, Widget};
use egui_ext::Markdown;
use egui_l20n::UiExt;
use itertools::Itertools as _;
use polars::prelude::*;

/// Indices widget
pub(crate) struct IndicesWidget<'a> {
    data_frame: &'a DataFrame,
    settings: Settings,
}

impl<'a> IndicesWidget<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame) -> Self {
        Self {
            data_frame,
            settings: Settings::default(),
        }
    }

    pub(crate) fn precision(mut self, precision: usize) -> Self {
        self.settings.precision = precision;
        self
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<()>> {
        let value = |ui: &mut Ui, name: &str| -> PolarsResult<()> {
            for column in self.data_frame.get_columns() {
                let series = column.struct_()?.field_by_name(name)?;
                if let Some(mean) = series.struct_()?.field_by_name("Mean")?.f64()?.first() {
                    let mut response = ui
                        .label(format!("{mean:.0$}", self.settings.precision))
                        .on_hover_text(mean.to_string());
                    if response.hovered() {
                        if let Some(standard_deviation) = series
                            .struct_()?
                            .field_by_name("StandardDeviation")?
                            .f64()?
                            .first()
                        {
                            response = response.on_hover_text(format!("± {standard_deviation}"));
                            if let Some(array) = series
                                .struct_()?
                                .field_by_name("Array")?
                                .array()?
                                .get_as_series(0)
                                && array.len() > 1
                            {
                                let formated = array.f64()?.iter().format_with(", ", |value, f| {
                                    if let Some(value) = value {
                                        f(&value)?;
                                    }
                                    Ok(())
                                });
                                response.on_hover_text(format!("[{formated}]"));
                            }
                        }
                    }
                }
            }
            Ok(())
        };
        Grid::new(ui.auto_id_with("Indices")).show(ui, |ui| -> PolarsResult<()> {
            ui.heading(ui.localize("Index"));
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("StereospecificNumber?number=123"));
                });
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("StereospecificNumber?number=13"));
                });
            ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("StereospecificNumber?number=2"));
                });
            ui.end_row();
            // Simple
            ui.label(ui.localize("Saturated"));
            value(ui, "Saturated")?;
            ui.end_row();
            ui.label(ui.localize("Monounsaturated"));
            value(ui, "Monounsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Polyunsaturated"));
            value(ui, "Polyunsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Unsaturated"));
            value(ui, "Unsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-9"));
            value(ui, "Unsaturated-9")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-6"));
            value(ui, "Unsaturated-6")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-3"));
            value(ui, "Unsaturated-3")?;
            ui.end_row();
            ui.label(ui.localize("Delta?index=9"));
            value(ui, "Unsaturated9")?;
            ui.end_row();
            ui.label(ui.localize("Trans")).on_hover_ui(|ui| {
                ui.markdown(asset!("/doc/en/Indices/Trans.md"));
            });
            value(ui, "Trans")?;
            ui.end_row();
            // Complex
            ui.label(ui.localize("EicosapentaenoicAndDocosahexaenoic"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!(
                        "/doc/en/Indices/EicosapentaenoicAndDocosahexaenoic.md"
                    ));
                });
            value(ui, "EicosapentaenoicAndDocosahexaenoic")?;
            ui.end_row();
            ui.label(ui.localize("FishLipidQuality")).on_hover_ui(|ui| {
                ui.markdown(asset!("/doc/en/Indices/FishLipidQuality.md"));
            });
            value(ui, "FishLipidQuality")?;
            ui.end_row();
            ui.label(ui.localize("HealthPromotingIndex"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/en/Indices/HealthPromotingIndex.md"));
                });
            value(ui, "HealthPromotingIndex")?;
            ui.end_row();
            ui.label(ui.localize("HypocholesterolemicToHypercholesterolemic"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!(
                        "/doc/en/Indices/HypocholesterolemicToHypercholesterolemic.md"
                    ));
                });
            value(ui, "HypocholesterolemicToHypercholesterolemic")?;
            ui.end_row();
            ui.label(ui.localize("IndexOfAtherogenicity"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/en/Indices/IndexOfAtherogenicity.md"));
                });
            value(ui, "IndexOfAtherogenicity")?;
            ui.end_row();
            ui.label(ui.localize("IndexOfThrombogenicity"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/en/Indices/IndexOfThrombogenicity.md"));
                });
            value(ui, "IndexOfThrombogenicity")?;
            ui.end_row();
            ui.label(ui.localize("LinoleicToAlphaLinolenic"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/en/Indices/LinoleicToAlphaLinolenic.md"));
                });
            value(ui, "LinoleicToAlphaLinolenic")?;
            ui.end_row();
            ui.label(ui.localize("Polyunsaturated-6ToPolyunsaturated-3"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!(
                        "/doc/en/Indices/Polyunsaturated-6ToPolyunsaturated-3.md"
                    ));
                });
            value(ui, "Polyunsaturated-6ToPolyunsaturated-3")?;
            ui.end_row();
            ui.label(ui.localize("PolyunsaturatedToSaturated"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/en/Indices/PolyunsaturatedToSaturated.md"));
                });
            value(ui, "PolyunsaturatedToSaturated")?;
            ui.end_row();
            ui.label(ui.localize("UnsaturationIndex"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/en/Indices/UnsaturationIndex.md"));
                });
            value(ui, "UnsaturationIndex")?;
            Ok(())
        })
    }
}

impl Widget for IndicesWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

/// Settings
#[derive(Clone, Copy, Debug, Default)]
struct Settings {
    precision: usize,
}
