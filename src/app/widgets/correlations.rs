#[cfg(feature = "markdown")]
use crate::asset;
use crate::{app::states::calculation::Settings, utils::chaddock::Sign};
use egui::{
    Grid, InnerResponse, Label, Response, RichText, TextStyle, TextWrapMode, Ui, Widget,
    text::TextWrapping,
};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_extras::{Column, TableBuilder};
use polars::prelude::*;

/// Correlations widget
pub(crate) struct CorrelationsWidget<'a> {
    data_frame: &'a DataFrame,
    chaddock: bool,
}

impl<'a> CorrelationsWidget<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &Settings) -> Self {
        Self {
            data_frame,
            chaddock: settings.chaddock,
        }
    }

    pub(crate) fn show(self, ui: &mut Ui) -> Response {
        let mut response = ui.response();
        let height = ui.text_style_height(&TextStyle::Body);
        let width = ui.spacing().combo_width;
        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        response.rect = TableBuilder::new(ui)
            .resizable(true)
            .striped(true)
            .column(Column::auto().resizable(true))
            .columns(
                Column::remainder().at_least(width / 2.0),
                self.data_frame.width() - 1,
            )
            .header(height, |mut roe| {
                for name in self.data_frame.get_column_names_str() {
                    roe.col(|ui| {
                        ui.heading(name);
                    });
                }
            })
            .body(|body| {
                body.rows(height, self.data_frame.height(), |mut row| {
                    let index = row.index();
                    let mut iter = self.data_frame.iter();
                    if let Some(series) = iter.next() {
                        row.col(|ui| {
                            let text = series.get(index).unwrap().str_value();
                            ui.label(text);
                        });
                    }
                    for series in iter {
                        row.col(|ui| {
                            let value = series.f64().unwrap().get(index).unwrap();
                            let sign = Sign::from(value);
                            let mut color = ui.style().visuals.text_color();
                            if self.chaddock {
                                color = sign.chaddock().color(color);
                            } else {
                                color = sign.color(color);
                            }
                            Label::new(RichText::new(value.to_string()).color(color)).ui(ui);
                        });
                    }
                });
            })
            .inner_rect;
        response
        // Grid::new(ui.auto_id_with("Indices")).show(ui, |ui| -> PolarsResult<()> {
        //     ui.heading(ui.localize("Index"));
        //     ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
        //         .on_hover_ui(|ui| {
        //             ui.label(ui.localize("StereospecificNumber?number=123"));
        //         });
        //     ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
        //         .on_hover_ui(|ui| {
        //             ui.label(ui.localize("StereospecificNumber?number=13"));
        //         });
        //     ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
        //         .on_hover_ui(|ui| {
        //             ui.label(ui.localize("StereospecificNumber?number=2"));
        //         });
        //     ui.end_row();
        //     // Simple
        //     ui.label(ui.localize("Saturated"));
        //     value(ui, "Saturated")?;
        //     ui.end_row();
        //     Ok(())
        // })
    }
}

// let sign = Sign::from(value);
// let mut color = ui.style().visuals.text_color();
// if self.settings.parameters.metric.is_finite() {
//     if self.settings.chaddock {
//         color = sign.chaddock().color(color);
//     } else {
//         color = sign.color(color);
//     }
// }
// Label::new(RichText::new(text).color(color))
//     .ui(ui)
//     .on_hover_text(value.to_string())
//     .on_hover_text(format!("{sign:?}"));

impl Widget for CorrelationsWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
