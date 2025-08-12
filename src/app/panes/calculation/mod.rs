use self::{settings::Settings, state::State, table::TableView};
use super::PaneDelegate;
use crate::{
    app::{
        computers::{
            CalculationComputed, CalculationIndicesComputed, CalculationIndicesKey, CalculationKey,
        },
        presets::CHRISTIE,
        widgets::{FattyAcidWidget, FloatWidget, IndicesWidget},
    },
    utils::Hashed,
};
use egui::{CursorIcon, Grid, Id, Response, RichText, ScrollArea, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CALCULATOR, GEAR, INTERSECT_THREE, LIST, MATH_OPERATIONS,
    SIGMA,
};
use lipid::prelude::DataFrameExt as _;
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
    state: State,
}

impl Pane {
    pub(crate) fn new(frames: Vec<MetaDataFrame>, index: usize) -> Self {
        Self {
            source: Hashed::new(frames),
            target: DataFrame::empty(),
            settings: Settings::new(Some(index)),
            state: State::new(),
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

    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui.heading(Self::icon()).on_hover_ui(|ui| {
            ui.label(ui.localize("calculation"));
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
            ui.label(ui.localize("list"));
        });
        ui.separator();
        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("reset_table"));
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
            ui.label(ui.localize("resize_table"));
        });
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.open_settings_window,
            RichText::new(GEAR).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("settings"));
        });
        ui.separator();
        // Indices
        ui.toggle_value(
            &mut self.state.open_indices_window,
            RichText::new(SIGMA).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("indices"));
        });
        ui.separator();
        // Composition
        if ui
            .button(RichText::new(INTERSECT_THREE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("composition"));
            })
            .clicked()
        {
            let mut target = Vec::with_capacity(self.source.len());
            for index in 0..self.source.len() {
                let meta = self.source[index].meta.clone();
                let data = ui.memory_mut(|memory| {
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
                target.push(MetaDataFrame::new(meta, data));
            }
            ui.data_mut(|data| data.insert_temp(Id::new("Compose"), (target, self.settings.index)));
        }
        ui.separator();
        response
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

    fn hash(&self) -> u64 {
        hash(&self.source)
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui) {
        self.christie_window(ui);
        self.indices_window(ui);
        self.settings_window(ui);
    }

    fn christie_window(&mut self, ui: &mut Ui) {
        let mut open_christie_window = self.state.open_christie_window;
        Window::new(format!("{MATH_OPERATIONS} Christie"))
            .default_pos(ui.next_widget_position())
            .id(ui.auto_id_with("Christie"))
            .open(&mut open_christie_window)
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
        self.state.open_christie_window = open_christie_window;
    }

    fn indices_window(&mut self, ui: &mut Ui) {
        let mut open_indices_window = self.state.open_indices_window;
        Window::new(format!("{SIGMA} Calculation indices"))
            .id(ui.auto_id_with(ID_SOURCE).with("Indices"))
            .open(&mut open_indices_window)
            .show(ui.ctx(), |ui| self.indices_content(ui));
        self.state.open_indices_window = open_indices_window;
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

    fn settings_window(&mut self, ui: &mut Ui) {
        let mut open_settings_window = self.state.open_settings_window;
        Window::new(format!("{GEAR} Calculation settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .open(&mut open_settings_window)
            .show(ui.ctx(), |ui| self.settings.show(ui, &mut self.state));
        self.state.open_settings_window = open_settings_window;
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
