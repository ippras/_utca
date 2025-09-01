use crate::{app::widgets::FloatWidget, asset};
use egui::{Grid, InnerResponse, Ui, Widget};
use egui_ext::Markdown;
use egui_l20n::UiExt;
use polars::prelude::{array::ArrayNameSpace, *};

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

    pub(crate) fn hover(mut self, hover: bool) -> Self {
        self.settings.hover = hover;
        self
    }

    pub(crate) fn precision(mut self, precision: Option<usize>) -> Self {
        self.settings.precision = precision;
        self
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<()>> {
        let mean_and_standard_deviation =
            |ui: &mut Ui, r#struct: &StructChunked| -> PolarsResult<()> {
                FloatWidget::new(r#struct.field_by_name("Mean")?.f64()?.first())
                    .precision(self.settings.precision)
                    .hover(self.settings.hover)
                    .show(ui)
                    .response
                    .on_hover_text(format!(
                        "± {}",
                        r#struct.field_by_name("StandardDeviation")?.str_value(0)?,
                    ))
                    .on_hover_text(r#struct.field_by_name("Repetitions")?.str_value(0)?);
                Ok(())
            };
        let values = |ui: &mut Ui, name: &str| -> PolarsResult<()> {
            for column in self.data_frame.get_columns() {
                let column = &column.struct_()?.field_by_name(name)?;
                match column.dtype() {
                    DataType::Float64 => {
                        FloatWidget::new(column.f64()?.first())
                            .precision(self.settings.precision)
                            .hover(self.settings.hover)
                            .show(ui);
                    }
                    DataType::Struct(_) => {
                        mean_and_standard_deviation(ui, column.struct_()?)?;
                    }
                    DataType::Array(box DataType::Float64, 3) => {
                        let array = column.array()?;
                        let stereospecific_number = |index| {
                            array.array_get(&Int64Chunked::full(PlSmallStr::EMPTY, index, 1), false)
                        };
                        FloatWidget::new(stereospecific_number(0)?.f64()?.first())
                            .precision(self.settings.precision)
                            .hover(self.settings.hover)
                            .show(ui);
                        FloatWidget::new(stereospecific_number(1)?.f64()?.first())
                            .precision(self.settings.precision)
                            .hover(self.settings.hover)
                            .show(ui);
                        FloatWidget::new(stereospecific_number(2)?.f64()?.first())
                            .precision(self.settings.precision)
                            .hover(self.settings.hover)
                            .show(ui);
                    }
                    DataType::Array(box DataType::Struct(_), 3) => {
                        let array = column.array()?;
                        let stereospecific_number = |index| {
                            array.array_get(&Int64Chunked::full(PlSmallStr::EMPTY, index, 1), false)
                        };
                        mean_and_standard_deviation(ui, stereospecific_number(0)?.struct_()?)?;
                        mean_and_standard_deviation(ui, stereospecific_number(1)?.struct_()?)?;
                        mean_and_standard_deviation(ui, stereospecific_number(2)?.struct_()?)?;
                    }
                    _ => {
                        polars_bail!(SchemaMismatch: "cannot show indices, data types don't match");
                    }
                };
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
            values(ui, "Saturated")?;
            ui.end_row();
            ui.label(ui.localize("Monounsaturated"));
            values(ui, "Monounsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Polyunsaturated"));
            values(ui, "Polyunsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Unsaturated"));
            values(ui, "Unsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-9"));
            values(ui, "Unsaturated-9")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-6"));
            values(ui, "Unsaturated-6")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-3"));
            values(ui, "Unsaturated-3")?;
            ui.end_row();
            ui.label(ui.localize("Delta?index=9"));
            values(ui, "Unsaturated9")?;
            ui.end_row();
            ui.label(ui.localize("Trans")).on_hover_ui(|ui| {
                ui.markdown(asset!("/doc/Indices/Trans.md"));
            });
            values(ui, "Trans")?;
            ui.end_row();
            // Complex
            ui.label(ui.localize("EicosapentaenoicAndDocosahexaenoic"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/Indices/EicosapentaenoicAndDocosahexaenoic.md"));
                });
            values(ui, "EicosapentaenoicAndDocosahexaenoic")?;
            ui.end_row();
            ui.label(ui.localize("FishLipidQuality")).on_hover_ui(|ui| {
                ui.markdown(asset!("/doc/Indices/FishLipidQuality.md"));
            });
            values(ui, "FishLipidQuality")?;
            ui.end_row();
            ui.label(ui.localize("HealthPromotingIndex"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/Indices/HealthPromotingIndex.md"));
                });
            values(ui, "HealthPromotingIndex")?;
            ui.end_row();
            ui.label(ui.localize("HypocholesterolemicToHypercholesterolemic"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!(
                        "/doc/Indices/HypocholesterolemicToHypercholesterolemic.md"
                    ));
                });
            values(ui, "HypocholesterolemicToHypercholesterolemic")?;
            ui.end_row();
            ui.label(ui.localize("IndexOfAtherogenicity"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/Indices/IndexOfAtherogenicity.md"));
                });
            values(ui, "IndexOfAtherogenicity")?;
            ui.end_row();
            ui.label(ui.localize("IndexOfThrombogenicity"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/Indices/IndexOfThrombogenicity.md"));
                });
            values(ui, "IndexOfThrombogenicity")?;
            ui.end_row();
            ui.label(ui.localize("LinoleicToAlphaLinolenic"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/Indices/LinoleicToAlphaLinolenic.md"));
                });
            values(ui, "LinoleicToAlphaLinolenic")?;
            ui.end_row();
            ui.label(ui.localize("Polyunsaturated-6ToPolyunsaturated-3"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!(
                        "/doc/Indices/Polyunsaturated-6ToPolyunsaturated-3.md"
                    ));
                });
            values(ui, "Polyunsaturated-6ToPolyunsaturated-3")?;
            ui.end_row();
            ui.label(ui.localize("PolyunsaturatedToSaturated"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/Indices/PolyunsaturatedToSaturated.md"));
                });
            values(ui, "PolyunsaturatedToSaturated")?;
            ui.end_row();
            ui.label(ui.localize("UnsaturationIndex"))
                .on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/Indices/UnsaturationIndex.md"));
                });
            values(ui, "UnsaturationIndex")?;
            Ok(())
        })
    }
}

impl Widget for IndicesWidget<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        self.show(ui).response
    }
}

/// Settings
#[derive(Clone, Copy, Debug, Default)]
struct Settings {
    hover: bool,
    precision: Option<usize>,
}
