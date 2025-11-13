#[cfg(feature = "markdown")]
use crate::asset;
use egui::{
    Grid, InnerResponse, Label, Response, RichText, TextStyle, TextWrapMode, Ui, Widget,
    text::TextWrapping,
};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_extras::{Column, TableBuilder};
use egui_l20n::UiExt;
use itertools::Itertools as _;
use lipid::prelude::LABEL;
use polars::prelude::*;

/// Correlations widget
pub(crate) struct CorrelationsWidget<'a> {
    data_frame: &'a DataFrame,
}

impl<'a> CorrelationsWidget<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame) -> Self {
        Self { data_frame }
    }

    pub(crate) fn show(self, ui: &mut Ui) -> Response {
        let mut response = ui.response();
        let height = ui.text_style_height(&TextStyle::Body);
        let width = ui.spacing().combo_width;
        ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        TableBuilder::new(ui)
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
                    for series in self.data_frame.iter() {
                        row.col(|ui| {
                            let text = series.get(index).unwrap().str_value();
                            ui.label(text);
                        });
                    }
                });
            });
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

impl Widget for CorrelationsWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
