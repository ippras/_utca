use crate::{app::states::calculation::settings::Settings, utils::chaddock::Sign};
use egui::{Label, Response, RichText, TextStyle, TextWrapMode, Ui, Widget};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_extras::{Column, TableBuilder, TableRow};
use polars::prelude::*;
use std::mem::take;
use tracing::instrument;

/// Correlations widget
pub(crate) struct Correlations<'a> {
    data_frame: &'a DataFrame,
    settings: &'a mut Settings,
}

impl<'a> Correlations<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &'a mut Settings) -> Self {
        Self {
            data_frame,
            settings,
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
                Column::remainder()
                    .at_least(width / 2.0)
                    .auto_size_this_frame(take(&mut self.settings.auto_size_correlations_table)),
                self.data_frame.width() - 1,
            )
            .header(height, |mut row| {
                for name in self.data_frame.get_column_names_str() {
                    row.col(|ui| {
                        ui.heading(name);
                    });
                }
            })
            .body(|body| {
                body.rows(height, self.data_frame.height(), |row| {
                    _ = self.table_row(row);
                });
            })
            .inner_rect;
        response
    }

    #[instrument(skip_all, err)]
    fn table_row(&self, mut row: TableRow) -> PolarsResult<()> {
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
                if self.settings.chaddock {
                    color = sign.chaddock().color(color);
                } else {
                    color = sign.color(color);
                }
                Label::new(RichText::new(value.to_string()).color(color)).ui(ui);
            });
        }
        Ok(())
    }
}

impl Widget for Correlations<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui)
    }
}
