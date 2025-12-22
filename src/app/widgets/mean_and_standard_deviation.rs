use crate::r#const::{CALCULATION, EM_DASH, MEAN, NO_BREAK_SPACE, SAMPLE, STANDARD_DEVIATION};
use egui::{Color32, Response, TextWrapMode, Ui, WidgetText};
use egui_l20n::prelude::*;
use polars::prelude::*;
use polars_utils::format_list;

pub struct NewMeanAndStandardDeviation<'a> {
    series: &'a Series,
    row: usize,
    calculation: bool,
    color: Option<Color32>,
    sample: bool,
    standard_deviation: bool,
}

impl<'a> NewMeanAndStandardDeviation<'a> {
    pub fn new(series: &'a Series, row: usize) -> Self {
        Self {
            series,
            row,
            calculation: false,
            color: None,
            sample: false,
            standard_deviation: false,
        }
    }
}

impl NewMeanAndStandardDeviation<'_> {
    pub fn with_calculation(self, calculation: bool) -> Self {
        Self {
            calculation,
            ..self
        }
    }

    pub fn with_color(self, color: Option<Color32>) -> Self {
        Self { color, ..self }
    }

    pub fn with_sample(self, sample: bool) -> Self {
        Self { sample, ..self }
    }

    pub fn with_standard_deviation(self, standard_deviation: bool) -> Self {
        Self {
            standard_deviation,
            ..self
        }
    }
}

impl NewMeanAndStandardDeviation<'_> {
    pub fn show(&self, ui: &mut Ui) -> PolarsResult<Response> {
        let mean_series = self.series.struct_()?.field_by_name(MEAN)?;
        let mean = mean_series.f64()?.get(self.row);
        let standard_deviation_series = self.series.struct_()?.field_by_name(STANDARD_DEVIATION)?;
        let standard_deviation = standard_deviation_series.f64()?.get(self.row);
        let mut text = match mean {
            Some(mean)
                if self.standard_deviation
                    && let Some(standard_deviation) = standard_deviation =>
            {
                WidgetText::from(format!("{mean}{NO_BREAK_SPACE}±{standard_deviation}"))
            }
            Some(mean) => WidgetText::from(mean.to_string()),
            None => WidgetText::from(EM_DASH),
        };
        if let Some(color) = self.color {
            text = text.color(color);
        }
        let mut response = ui.label(text);
        if response.hovered() {
            // Standard deviation
            if let Some(standard_deviation) = standard_deviation {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(STANDARD_DEVIATION));
                    ui.label(format!("±{standard_deviation}"));
                });
            }
            // Sample
            if self.sample
                && let Some(sample) = self
                    .series
                    .struct_()?
                    .field_by_name(SAMPLE)?
                    .array()?
                    .get_as_series(self.row)
                && sample.len() > 1
            {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(SAMPLE));
                    ui.label(format_list!(sample.iter()));
                });
            }
        }
        Ok(response)
    }
}

/// Mean and standard deviation widget
///
/// Mean, standard deviation, sample
pub struct MeanAndStandardDeviation<'a, const N: usize> {
    data_frame: &'a DataFrame,
    column: [&'a str; N],
    row: usize,
    calculation: bool,
    color: Option<Color32>,
    sample: bool,
    standard_deviation: bool,
}

impl<'a, const N: usize> MeanAndStandardDeviation<'a, N> {
    pub fn new(data_frame: &'a DataFrame, column: [&'a str; N], row: usize) -> Self {
        Self {
            data_frame,
            column,
            row,
            calculation: false,
            color: None,
            sample: false,
            standard_deviation: false,
        }
    }

    pub fn with_calculation(self, calculation: bool) -> Self {
        Self {
            calculation,
            ..self
        }
    }

    pub fn with_color(self, color: Option<Color32>) -> Self {
        Self { color, ..self }
    }

    pub fn with_sample(self, sample: bool) -> Self {
        Self { sample, ..self }
    }

    pub fn with_standard_deviation(self, standard_deviation: bool) -> Self {
        Self {
            standard_deviation,
            ..self
        }
    }
}

impl<const N: usize> MeanAndStandardDeviation<'_, N> {
    pub fn show(&self, ui: &mut Ui) -> PolarsResult<Response> {
        let mut series = self.data_frame[self.column[0]]
            .as_materialized_series()
            .clone();
        for name in &self.column[1..] {
            series = series.struct_()?.field_by_name(name)?;
        }
        let mean_series = series.struct_()?.field_by_name(MEAN)?;
        let mean = mean_series.f64()?.get(self.row);
        let standard_deviation_series = series.struct_()?.field_by_name(STANDARD_DEVIATION)?;
        let standard_deviation = standard_deviation_series.f64()?.get(self.row);
        let mut text = match mean {
            Some(mean)
                if self.standard_deviation
                    && let Some(standard_deviation) = standard_deviation =>
            {
                WidgetText::from(format!("{mean}{NO_BREAK_SPACE}±{standard_deviation}"))
            }
            Some(mean) => WidgetText::from(mean.to_string()),
            None => WidgetText::from(EM_DASH),
        };
        if let Some(color) = self.color {
            text = text.color(color);
        }
        let mut response = ui.label(text);
        if response.hovered() {
            // Standard deviation
            if let Some(standard_deviation) = standard_deviation {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(STANDARD_DEVIATION));
                    ui.label(format!("±{standard_deviation}"));
                });
            }
            // Sample
            if self.sample
                && let Some(sample) = series
                    .struct_()?
                    .field_by_name(SAMPLE)?
                    .array()?
                    .get_as_series(self.row)
                && sample.len() > 1
            {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(SAMPLE));
                    ui.label(format_list!(sample.iter()));
                });
            }
            // Calculation
            let name = self.column.join(".") + "." + CALCULATION;
            // name.push_str(CALCULATION);
            // self.data_frame[&*name];
            if self.calculation
                && let Some(calculation) = self.data_frame[&*name].str()?.get(self.row)
            {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize(CALCULATION));
                    ui.label(calculation);
                });
            }
        }
        Ok(response)
    }
}
