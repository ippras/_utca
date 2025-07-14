use crate::app::widgets::FloatWidget;
use egui::{Grid, InnerResponse, Ui, Widget};
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
            ui.label("Saturated");
            index(ui, "Saturated")?;
            ui.end_row();
            ui.label("Monounsaturated");
            index(ui, "Monounsaturated")?;
            ui.end_row();
            ui.label("Polyunsaturated");
            index(ui, "Polyunsaturated")?;
            ui.end_row();
            ui.label("Unsaturated");
            index(ui, "Unsaturated")?;
            ui.end_row();
            ui.label("Omega -9");
            index(ui, "Unsaturated-9")?;
            ui.end_row();
            ui.label("Omega -6");
            index(ui, "Unsaturated-6")?;
            ui.end_row();
            ui.label("Omega -3");
            index(ui, "Unsaturated-3")?;
            ui.end_row();
            ui.label("Delta 9");
            index(ui, "Unsaturated9")?;
            ui.end_row();
            ui.label("Trans");
            index(ui, "Trans")?;
            ui.end_row();
            // Complex
            ui.label("Eicosapentaenoic and Docosahexaenoic");
            index(ui, "EicosapentaenoicAndDocosahexaenoic")?;
            ui.end_row();
            ui.label("Fish Lipid Quality");
            index(ui, "FishLipidQuality")?;
            ui.end_row();
            ui.label("Health Promoting Index");
            index(ui, "HealthPromotingIndex")?;
            ui.end_row();
            ui.label("Hypocholesterolemic to Hypercholesterolemic");
            index(ui, "HypocholesterolemicToHypercholesterolemic")?;
            ui.end_row();
            ui.label("Index of Atherogenicity");
            index(ui, "IndexOfAtherogenicity")?;
            ui.end_row();
            ui.label("Index of Thrombogenicity");
            index(ui, "IndexOfThrombogenicity")?;
            ui.end_row();
            ui.label("Linoleic to αLinolenic");
            index(ui, "LinoleicToAlphaLinolenic")?;
            ui.end_row();
            ui.label("Polyunsaturated to Saturated");
            index(ui, "PolyunsaturatedToSaturated")?;
            ui.end_row();
            ui.label("Unsaturation Index");
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
