use self::{
    parameters::Parameters,
    state::{Settings, Windows},
    table::TableView,
};
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
    utils::Hashed,
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
    parameters: Parameters,
}

impl Pane {
    pub(crate) fn new(frames: Vec<MetaDataFrame>, index: usize) -> Self {
        Self {
            source: Hashed::new(frames),
            target: DataFrame::empty(),
            parameters: Parameters::new(Some(index)),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        CALCULATOR
    }

    pub(crate) fn title(&self) -> String {
        match self.parameters.index {
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
        let mut settings = Settings::load(ui.ctx());
        settings
            .table
            .filter
            .update(&self.target.get_column_names_str());
        let mut windows = Windows::load(ui.ctx());
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
                        &mut self.parameters.index,
                        Some(index),
                        self.source[index].meta.format(".").to_string(),
                    )
                    .clicked()
            }
            ui.selectable_value(
                &mut self.parameters.index,
                None,
                "Mean ± standard deviations",
            );
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
        ui.toggle_value(
            &mut settings.table.reset_state,
            RichText::new(ARROWS_CLOCKWISE).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("ResetTable"));
        });
        // Resize
        ui.toggle_value(
            &mut settings.table.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("ResizeTable"));
        });
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut windows.open_settings,
            RichText::new(SLIDERS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("Settings"));
        });
        ui.separator();
        // Parameters
        ui.toggle_value(&mut windows.open_parameters, RichText::new(GEAR).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Parameters"));
            });
        ui.separator();
        // Indices
        ui.toggle_value(&mut windows.open_indices, RichText::new(SIGMA).heading())
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
        settings.store(ui.ctx());
        windows.store(ui.ctx());
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
                        settings: &Parameters {
                            index: Some(index),
                            ..self.parameters
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
        ui.data_mut(|data| data.insert_temp(Id::new(COMPOSE), (frames, self.parameters.index)));
        Ok(())
    }

    fn body_content(&mut self, ui: &mut Ui) {
        self.target = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationComputed>()
                .get(CalculationKey {
                    frames: &self.source,
                    settings: &self.parameters,
                })
        });
        TableView::new(ui.ctx(), &self.target, &self.parameters).show(ui);
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui) {
        let windows = &mut Windows::load(ui.ctx());
        self.christie(ui, windows);
        self.indices(ui, windows);
        self.parameters(ui, windows);
        self.settings(ui, windows);
        windows.store(ui.ctx());
    }

    fn christie(&mut self, ui: &mut Ui, windows: &mut Windows) {
        Window::new(format!("{MATH_OPERATIONS} Christie"))
            .default_pos(ui.next_widget_position())
            .id(ui.auto_id_with("Christie"))
            .open(&mut windows.open_christie)
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
    }

    fn indices(&mut self, ui: &mut Ui, windows: &mut Windows) {
        Window::new(format!("{SIGMA} Calculation indices"))
            .id(ui.auto_id_with(ID_SOURCE).with("Indices"))
            .open(&mut windows.open_indices)
            .show(ui.ctx(), |ui| self.indices_content(ui));
    }

    #[instrument(skip_all, err)]
    fn indices_content(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        println!("lazydata_frame_frame0: {:?}", self.target);
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationIndicesComputed>()
                .get(CalculationIndicesKey {
                    data_frame: Hashed {
                        value: &self.target,
                        hash: hash(self.parameters.index),
                    },
                    ddof: self.parameters.ddof,
                })
        });
        let settings = Settings::load(ui.ctx());
        IndicesWidget::new(&data_frame)
            .hover(true)
            .precision(Some(settings.precision))
            .show(ui)
            .inner
    }

    fn parameters(&mut self, ui: &mut Ui, windows: &mut Windows) {
        Window::new(format!("{GEAR} Calculation parameters"))
            .id(ui.auto_id_with(ID_SOURCE).with("Parameters"))
            .open(&mut windows.open_parameters)
            .show(ui.ctx(), |ui| {
                self.parameters.show(ui);
            });
    }

    fn settings(&mut self, ui: &mut Ui, windows: &mut Windows) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Calculation settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .open(&mut windows.open_settings)
            .show(ui.ctx(), |ui| {
                let mut settings = Settings::load(ui.ctx());
                settings.show(ui);
                settings.store(ui.ctx());
            });
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

pub(crate) mod parameters;

mod state;
mod table;
