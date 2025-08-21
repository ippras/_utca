use self::{settings::Settings, table::TableView};
use super::PaneDelegate;
use crate::{
    app::{
        computers::{
            CalculationComputed, CalculationIndicesComputed, CalculationIndicesKey, CalculationKey,
        },
        identifiers::COMPOSE,
        presets::CHRISTIE,
        widgets::{FattyAcidWidget, FloatWidget, IndicesWidget},
    },
    utils::{
        Hashed,
        egui::table::{ColumnState, TableState},
    },
};
use egui::{CursorIcon, Grid, Id, Response, RichText, ScrollArea, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CALCULATOR, GEAR, INTERSECT_THREE, LIST, MATH_OPERATIONS,
    SIGMA, SLIDERS_HORIZONTAL,
};
use lipid::prelude::*;
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use tracing::instrument;

const ID_SOURCE: &str = "Calculation";

/// Calculation pane
#[derive(Deserialize, Serialize)]
pub(crate) struct Pane {
    source: Hashed<Vec<MetaDataFrame>>,
    target: DataFrame,
    settings: Settings,
}

impl Pane {
    pub(crate) fn new(frames: Vec<MetaDataFrame>, index: usize) -> Self {
        Self {
            source: Hashed::new(frames),
            target: DataFrame::empty(),
            settings: Settings::new(Some(index)),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        CALCULATOR
    }

    pub(crate) fn title(&self) -> String {
        match self.settings.index {
            Some(index) => self.source[index].meta.format(" ").to_string(),
            None => {
                format_list_truncated!(
                    self.source
                        .iter()
                        .map(|frame| frame.meta.format(" ").to_string()),
                    2
                )
            }
        }
    }

    fn hash(&self) -> u64 {
        hash(&self.source)
    }
}

impl Pane {
    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui.heading(Self::icon()).on_hover_ui(|ui| {
            ui.label(ui.localize("Calculation"));
        });
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.hash()))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            let mut clicked = false;
            for index in 0..self.source.len() {
                clicked |= ui
                    .selectable_value(
                        &mut self.settings.index,
                        Some(index),
                        self.source[index].meta.format(".").to_string(),
                    )
                    .clicked()
            }
            ui.selectable_value(&mut self.settings.index, None, "Mean ± standard deviations");
            if clicked {
                ui.close();
            }
        })
        .response
        .on_hover_ui(|ui| {
            ui.label(ui.localize("List"));
        });
        ui.separator();
        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("ResetTable"));
            })
            .clicked()
        {
            self.state.reset_table_state = true;
        }
        // Resize
        ui.toggle_value(
            &mut self.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("ResizeTable"));
        });
        ui.separator();
        // Parameters
        ui.toggle_value(
            &mut self.state.windows.open_parameters,
            RichText::new(SLIDERS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("Parameters"));
        });
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.windows.open_settings,
            RichText::new(GEAR).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("Settings"));
        });
        ui.separator();
        // Indices
        ui.toggle_value(
            &mut self.state.windows.open_indices,
            RichText::new(SIGMA).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("Indices"));
        });
        ui.separator();
        // Composition
        if ui
            .button(RichText::new(INTERSECT_THREE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Composition"));
            })
            .clicked()
        {
            let _ = self.composition(ui);
        }
        ui.separator();
        response
    }

    #[instrument(skip_all, err)]
    fn composition(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        let mut frames = Vec::with_capacity(self.source.len());
        for index in 0..self.source.len() {
            let meta = self.source[index].meta.clone();
            let mut data = ui.memory_mut(|memory| {
                memory
                    .caches
                    .cache::<CalculationComputed>()
                    .get(CalculationKey {
                        frames: &self.source,
                        settings: &Settings {
                            index: Some(index),
                            ..self.settings
                        },
                    })
            });
            data = data
                .lazy()
                .select([
                    col(LABEL),
                    col(FATTY_ACID),
                    col("Calculated")
                        .struct_()
                        .field_by_name("Triacylglycerol")
                        .alias(STEREOSPECIFIC_NUMBER123),
                    col("Calculated")
                        .struct_()
                        .field_by_name("Diacylglycerol13")
                        .alias(STEREOSPECIFIC_NUMBER13),
                    col("Calculated")
                        .struct_()
                        .field_by_name("Monoacylglycerol2")
                        .alias(STEREOSPECIFIC_NUMBER2),
                ])
                .collect()?;
            frames.push(MetaDataFrame::new(meta, data));
        }
        // println!("target: {target:?}");
        ui.data_mut(|data| data.insert_temp(Id::new(COMPOSE), (frames, self.settings.index)));
        Ok(())
    }

    fn body_content(&mut self, ui: &mut Ui) {
        self.target = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationComputed>()
                .get(CalculationKey {
                    frames: &self.source,
                    settings: &self.settings,
                })
        });
        TableView::new(&self.target, &self.settings, &mut self.state).show(ui);
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui) {
        self.christie(ui);
        self.indices(ui);
        self.parameters(ui);
        self.settings(ui);
    }

    fn christie(&mut self, ui: &mut Ui) {
        let mut open = self.state.windows.open_christie;
        Window::new(format!("{MATH_OPERATIONS} Christie"))
            .default_pos(ui.next_widget_position())
            .id(ui.auto_id_with("Christie"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                ScrollArea::vertical().show(ui, |ui| {
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        ui.heading("Fatty Acid");
                        ui.heading("Value");
                        ui.end_row();
                        for index in 0..CHRISTIE.data.height() {
                            let fatty_acid = CHRISTIE.data.fatty_acid().get(index).unwrap();
                            FattyAcidWidget::new(fatty_acid.as_ref())
                                .hover(true)
                                .show(ui);
                            FloatWidget::new(CHRISTIE.data["Christie"].f64().unwrap().get(index))
                                .show(ui);
                            ui.end_row();
                        }
                    });
                });
            });
        self.state.windows.open_christie = open;
    }

    fn indices(&mut self, ui: &mut Ui) {
        let mut open = self.state.windows.open_indices;
        Window::new(format!("{SIGMA} Calculation indices"))
            .id(ui.auto_id_with(ID_SOURCE).with("Indices"))
            .open(&mut open)
            .show(ui.ctx(), |ui| self.indices_content(ui));
        self.state.windows.open_indices = open;
    }

    #[instrument(skip_all, err)]
    fn indices_content(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationIndicesComputed>()
                .get(CalculationIndicesKey {
                    data_frame: Hashed {
                        value: &self.target,
                        hash: hash(self.settings.index),
                    },
                    ddof: self.settings.ddof,
                })
        });
        IndicesWidget::new(&data_frame)
            .hover(true)
            .precision(Some(self.settings.precision))
            .show(ui)
            .inner
    }

    fn parameters(&mut self, ui: &mut Ui) {
        let mut open = self.state.windows.open_parameters;
        Window::new(format!("{SLIDERS_HORIZONTAL} Calculation parameters"))
            .id(ui.auto_id_with(ID_SOURCE).with("Parameters"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                self.state.parameters.show(ui);
                let id = ui.make_persistent_id(ID_SOURCE).with("Parameters");
                let mut table_state = TableState::load(
                    ui.ctx(),
                    id,
                    self.target
                        .get_column_names_str()
                        .into_iter()
                        .map(|name| ColumnState::new(Id::new(name), name.to_owned())),
                );
                table_state.show(ui);
                table_state.store(ui.ctx());
            });
        self.state.windows.open_parameters = open;
    }

    fn settings(&mut self, ui: &mut Ui) {
        let mut open = self.state.windows.open_settings;
        Window::new(format!("{GEAR} Calculation settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .open(&mut open)
            .show(ui.ctx(), |ui| {
                self.settings.show(ui, &mut self.state);
            });
        self.state.windows.open_settings = open;
    }
}

impl PaneDelegate for Pane {
    fn header(&mut self, ui: &mut Ui) -> Response {
        self.header_content(ui)
    }

    fn body(&mut self, ui: &mut Ui) {
        self.body_content(ui);
        self.windows(ui);
    }
}

pub(crate) mod settings;

mod state;
mod table;
