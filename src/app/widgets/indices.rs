use crate::app::widgets::FloatWidget;
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
        let index = |ui: &mut Ui, name: &str| -> PolarsResult<()> {
            let column = &self.data_frame[name];
            match column.dtype() {
                DataType::Float64 => {
                    FloatWidget::new(self.data_frame[name].f64()?.first())
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
            Ok(())
        };
        Grid::new(ui.auto_id_with("Indices")).show(ui, |ui| -> PolarsResult<()> {
            // Simple
            ui.label(ui.localize("Saturated"));
            index(ui, "Saturated")?;
            ui.end_row();
            ui.label(ui.localize("Monounsaturated"));
            index(ui, "Monounsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Polyunsaturated"));
            index(ui, "Polyunsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Unsaturated"));
            index(ui, "Unsaturated")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-9"));
            index(ui, "Unsaturated-9")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-6"));
            index(ui, "Unsaturated-6")?;
            ui.end_row();
            ui.label(ui.localize("Omega?index=-3"));
            index(ui, "Unsaturated-3")?;
            ui.end_row();
            ui.label(ui.localize("Delta?index=9"));
            index(ui, "Unsaturated9")?;
            ui.end_row();
            ui.label(ui.localize("Trans"));
            index(ui, "Trans")?;
            ui.end_row();
            ui.separator();
            ui.separator();
            ui.end_row();
            // Complex
            ui.label(ui.localize("EicosapentaenoicAndDocosahexaenoic")).on_hover_ui(|ui| {
                ui.markdown(r#"$$C22:6(n - 3) + C20:5(n - 3)$$"#);
            });
            index(ui, "EicosapentaenoicAndDocosahexaenoic")?;
            ui.end_row();
            ui.label(ui.localize("FishLipidQuality")).on_hover_ui(|ui| {
                ui.markdown(r#"$$\frac{C22:6(n - 3) + C20:5(n - 3)}{\sum FA} \cdot 100$$"#);
            });
            index(ui, "FishLipidQuality")?;
            ui.end_row();
            ui.label(ui.localize("HealthPromotingIndex")).on_hover_ui(|ui| {
                ui.markdown(r#"$$\frac{\sum UFA}{C12:0 + 4 \cdot C14:0 + C16:0}$$"#);
            });
            index(ui, "HealthPromotingIndex")?;
            ui.end_row();
            ui.label(ui.localize("HypocholesterolemicToHypercholesterolemic")).on_hover_ui(|ui| {
                ui.markdown(r#"$$\frac{C18:1(n - 9) + \sum PUFA}{C12:0 + C14:0 + C16:0}$$"#);
            });
            index(ui, "HypocholesterolemicToHypercholesterolemic")?;
            ui.end_row();
            ui.label(ui.localize("IndexOfAtherogenicity")).on_hover_ui(|ui| {
                ui.markdown(r#"$$\frac{C12:0 + 4 \cdot C14:0 + C16:0}{\sum UFA}$$"#);
            });
            index(ui, "IndexOfAtherogenicity")?;
            ui.end_row();
            ui.label(ui.localize("IndexOfThrombogenicity")).on_hover_ui(|ui| {
                ui.markdown(r#"$$\frac{C14:0 + C16:0 + C18:0}{0.5 \cdot \sum MUFA + 0.5 \cdot \sum PUFA(n - 6) + 3 \cdot \sum PUFA(n - 3) + \frac{\sum PUFA(n - 3)}{\sum PUFA(n - 6)}}$$"#);
            });
            index(ui, "IndexOfThrombogenicity")?;
            ui.end_row();
            ui.label(ui.localize("LinoleicToAlphaLinolenic")).on_hover_ui(|ui| {
                ui.markdown(r#"$$\frac{C18:2(n - 6)}{C18:3(n - 3)}$$"#);
            });
            index(ui, "LinoleicToAlphaLinolenic")?;
            ui.end_row();
            ui.label(ui.localize("Polyunsaturated-6ToPolyunsaturated-3")).on_hover_ui(|ui| {
                ui.markdown(r#"$$\frac{\sum PUFA(n - 6)}{\sum PUFA(n - 3)}$$"#);
            });
            index(ui, "Polyunsaturated-6ToPolyunsaturated-3")?;
            ui.end_row();
            ui.label(ui.localize("PolyunsaturatedToSaturated")).on_hover_ui(|ui| {
                ui.markdown(r#"$$\frac{\sum PUFA}{\sum SFA}$$"#);
            });
            index(ui, "PolyunsaturatedToSaturated")?;
            ui.end_row();
            ui.label(ui.localize("UnsaturationIndex")).on_hover_ui(|ui| {
                ui.markdown(r#"$$1 \cdot monoenoics + 2 \cdot dienoics + 3 \cdot trienoics + 4 \cdot tetraenoics + 5 \cdot pentaenoics + 6 \cdot hexaenoics ...$$"#);
            });
            index(ui, "UnsaturationIndex")?;
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
