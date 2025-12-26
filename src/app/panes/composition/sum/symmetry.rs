use crate::{
    app::{
        states::composition::settings::Settings,
        widgets::mean_and_standard_deviation::{
            MeanAndStandardDeviation, NewMeanAndStandardDeviation,
        },
    },
    r#const::{EM_DASH, GROUP, TRIACYLGLYCEROLS, VALUE},
};
use egui::{
    Grid, InnerResponse, PopupCloseBehavior, Response, ScrollArea, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_ext::InnerResponseExt;
use egui_l20n::prelude::*;
use egui_phosphor::regular::LIST;
use lipid::prelude::LABEL;
use polars::prelude::*;

/// Symmetry widget
pub(crate) struct Symmetry<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
}

impl<'a> Symmetry<'a> {
    pub(crate) fn new(data_frame: &'a DataFrame, settings: &'a Settings) -> Self {
        Self {
            data_frame,
            settings,
        }
    }

    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<()>> {
        Grid::new(ui.auto_id_with("Symmetry")).show(ui, |ui| -> PolarsResult<()> {
            ui.heading(ui.localize("Symmetry"));
            ui.heading(ui.localize("Value"))
                .on_hover_localized("Value.hover");
            ui.heading(ui.localize("Species"));
            ui.end_row();
            for row in 0..self.data_frame.height() {
                let group = self.data_frame[GROUP].str()?.get(row).unwrap_or(EM_DASH);
                ui.label(group);
                MeanAndStandardDeviation::new(self.data_frame, [VALUE], row)
                    .with_standard_deviation(self.settings.standard_deviation)
                    .with_sample(true)
                    .show(ui)?;
                self.list_button(ui, row)?;
                ui.end_row();
            }
            Ok(())
        })
    }

    fn list_button(&self, ui: &mut Ui, row: usize) -> PolarsResult<()> {
        let (_, inner_response) = MenuButton::new(LIST)
            .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
            .ui(ui, |ui| {
                ScrollArea::vertical()
                    .max_height(ui.spacing().combo_height)
                    .show(ui, |ui| self.list_button_content(ui, row))
                    .inner
            });
        inner_response.transpose()?;
        Ok(())
    }

    fn list_button_content(&self, ui: &mut Ui, row: usize) -> PolarsResult<()> {
        let triacylglycerols = self.data_frame[TRIACYLGLYCEROLS]
            .list()?
            .get_as_series(row)
            .unwrap_or_default();
        Grid::new(ui.next_auto_id())
            .show(ui, |ui| -> PolarsResult<()> {
                for (index, label) in triacylglycerols
                    .struct_()?
                    .field_by_name(LABEL)?
                    .iter()
                    .enumerate()
                {
                    ui.label(index.to_string());
                    ui.label(label.to_string());
                    NewMeanAndStandardDeviation::new(
                        &triacylglycerols.struct_()?.field_by_name(VALUE)?,
                        index,
                    )
                    .with_standard_deviation(self.settings.standard_deviation)
                    .with_sample(true)
                    .show(ui)?;
                    ui.end_row();
                }
                Ok(())
            })
            .inner
    }
}

impl Widget for Symmetry<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}
