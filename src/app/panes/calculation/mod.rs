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
        widgets::{FattyAcidWidget, FloatWidget, IndicesWidget},
    },
    export::ron,
    utils::{
        HashedDataFrame, HashedMetaDataFrame,
        egui::UiExt as _,
        metadata::{authors, date, name},
    },
};
use anyhow::{Result, bail};
use chrono::NaiveDate;
use egui::{CursorIcon, Grid, Id, Response, RichText, ScrollArea, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    CALCULATOR, FLOPPY_DISK, GEAR, INTERSECT_THREE, LIST, MATH_OPERATIONS, SIGMA,
    SLIDERS_HORIZONTAL,
};
use itertools::Itertools;
use lipid::prelude::*;
use metadata::{
    AUTHORS, DATE, DEFAULT_DATE, DEFAULT_VERSION, Metadata, NAME, VERSION, polars::MetaDataFrame,
};
use polars::prelude::*;
use polars_utils::{format_list, format_list_truncated};
use serde::{Deserialize, Serialize};
use tracing::instrument;

const ID_SOURCE: &str = "Calculation";

/// Calculation pane
#[derive(Deserialize, Serialize)]
pub(crate) struct Pane {
    source: Vec<HashedMetaDataFrame>,
    target: HashedDataFrame,
    parameters: Parameters,
}

impl Pane {
    pub(crate) fn new(frames: Vec<HashedMetaDataFrame>, index: usize) -> Self {
        Self {
            source: frames,
            target: HashedDataFrame {
                data_frame: DataFrame::empty(),
                hash: 0,
            },
            parameters: Parameters::new(Some(index)),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        CALCULATOR
    }

    pub(crate) fn title(&self) -> String {
        self.title_with_separator(" ")
    }

    fn title_with_separator(&self, separator: &str) -> String {
        match self.parameters.index {
            Some(index) => self.source[index].meta.format(separator).to_string(),
            None => {
                format_list_truncated!(
                    self.source
                        .iter()
                        .map(|frame| frame.meta.format(separator).to_string()),
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
        ui.reset(&mut settings.table.reset_state);
        // Resize
        ui.resize(&mut settings.table.resizable);
        ui.separator();
        // Settings
        ui.settings(&mut windows.open_settings);
        ui.separator();
        // Parameters
        ui.parameters(&mut windows.open_parameters);
        ui.separator();
        // Indices
        ui.menu_button(RichText::new(SIGMA).heading(), |ui| {
            // ui.indices(&mut windows.open_indices);
            ui.toggle_value(
                &mut windows.open_indices,
                (
                    RichText::new(SIGMA).heading(),
                    RichText::new(ui.localize("Indices")).heading(),
                ),
            )
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Indices"));
            });
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
        // Save
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            let title = self.title_with_separator(".");
            if ui
                .button("RON")
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(&format!("{title}.fa.utca.ron"));
                })
                .clicked()
            {
                let _ = self.save_ron(&title);
            }
            if ui
                .button("PARQUET")
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(&format!("{title}.fa.utca.parquet"));
                })
                .clicked()
            {
                // let _ = self.save_parquet(&title);
            }
        });
        ui.separator();
        settings.store(ui.ctx());
        windows.store(ui.ctx());
        response
    }

    #[instrument(skip_all, err)]
    fn save_ron(&mut self, title: &str) -> Result<()> {
        let data = self
            .target
            .data_frame
            .clone()
            .lazy()
            .select([
                col(LABEL),
                col(FATTY_ACID),
                col(STEREOSPECIFIC_NUMBERS123),
                col(STEREOSPECIFIC_NUMBERS13),
                col(STEREOSPECIFIC_NUMBERS2),
            ])
            .collect()?;
        let meta = match self.parameters.index {
            Some(index) => {
                let mut meta = self.source[index].meta.clone();
                meta.retain(|key, _| key != "ARROW:schema");
                meta
            }
            None => {
                let mut meta = Metadata::default();
                // let name =
                //     format_list!(self.source.iter().filter_map(|frame| frame.meta.get(NAME)));
                meta.insert(NAME.to_owned(), name(&self.source));
                meta.insert(AUTHORS.to_owned(), authors(&self.source));
                meta.insert(DATE.to_owned(), date(&self.source));
                meta.insert(VERSION.to_owned(), DEFAULT_VERSION.to_owned());
                meta
            }
        };
        let frame = MetaDataFrame::new(meta, data);
        ron::save(&frame, &format!("{title}.fa.utca.ron"))?;
        Ok(())
    }

    // #[instrument(skip_all, err)]
    // fn save_parquet(&mut self, title: &str) -> PolarsResult<()> {
    //     let data = self
    //         .target
    //         .data_frame
    //         .clone()
    //         .lazy()
    //         .select([
    //             col(LABEL),
    //             col(FATTY_ACID),
    //             col(STEREOSPECIFIC_NUMBERS123),
    //             col(STEREOSPECIFIC_NUMBERS13),
    //             col(STEREOSPECIFIC_NUMBERS2),
    //         ])
    //         .collect()?;
    //     let meta = match self.parameters.index {
    //         Some(index) => {
    //             let mut meta = self.source[index].meta.clone();
    //             meta.retain(|key, _| key != "ARROW:schema");
    //             meta
    //         }
    //         None => {
    //             let mut meta = Metadata::default();
    //             let name =
    //                 format_list!(self.source.iter().filter_map(|frame| frame.meta.get(NAME)));
    //             meta.insert(NAME.to_owned(), name);
    //             let authors = self
    //                 .source
    //                 .iter()
    //                 .flat_map(|frame| frame.meta.get(AUTHORS).map(|authors| authors.split(",")))
    //                 .flatten()
    //                 .unique()
    //                 .join(",");
    //             meta.insert(AUTHORS.to_owned(), authors);
    //             meta.insert(DATE.to_owned(), DEFAULT_DATE.to_owned());
    //             meta.insert(VERSION.to_owned(), DEFAULT_VERSION.to_owned());
    //             meta
    //         }
    //     };
    //     let mut frame = MetaDataFrame::new(meta, data);
    //     let _ = parquet::save(&mut frame, &format!("{title}.fa.utca.parquet"));
    //     Ok(())
    // }

    #[instrument(skip_all, err)]
    fn composition(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        let mut frames = Vec::with_capacity(self.source.len());
        for index in 0..self.source.len() {
            let meta = self.source[index].meta.clone();
            let HashedDataFrame { data_frame, hash } = ui.memory_mut(|memory| {
                memory
                    .caches
                    .cache::<CalculationComputed>()
                    .get(CalculationKey {
                        frames: &self.source,
                        parameters: &Parameters {
                            index: Some(index),
                            ..self.parameters
                        },
                    })
            });
            let data_frame = data_frame
                .lazy()
                .select([
                    col(LABEL),
                    col(FATTY_ACID),
                    col(STEREOSPECIFIC_NUMBERS123)
                        .struct_()
                        .field_by_name("Mean")
                        .alias(STEREOSPECIFIC_NUMBERS123),
                    col(STEREOSPECIFIC_NUMBERS13)
                        .struct_()
                        .field_by_name("Mean")
                        .alias(STEREOSPECIFIC_NUMBERS13),
                    col(STEREOSPECIFIC_NUMBERS2)
                        .struct_()
                        .field_by_name("Mean")
                        .alias(STEREOSPECIFIC_NUMBERS2),
                ])
                .collect()?;
            frames.push(MetaDataFrame::new(
                meta,
                HashedDataFrame { data_frame, hash },
            ));
        }
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
                    parameters: &self.parameters,
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
                // ScrollArea::vertical().show(ui, |ui| {
                //     Grid::new(ui.next_auto_id()).show(ui, |ui| {
                //         ui.heading("Fatty Acid");
                //         ui.heading("Value");
                //         ui.end_row();
                //         for index in 0..CHRISTIE.data.height() {
                //             let fatty_acid = CHRISTIE.data.fatty_acid().get(index).unwrap();
                //             FattyAcidWidget::new(fatty_acid.as_ref())
                //                 .hover(true)
                //                 .show(ui);
                //             FloatWidget::new(CHRISTIE.data["Christie"].f64().unwrap().get(index))
                //                 .show(ui);
                //             ui.end_row();
                //         }
                //     });
                // });
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
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationIndicesComputed>()
                .get(CalculationIndicesKey {
                    frame: &self.target,
                    ddof: self.parameters.ddof,
                })
        });
        let settings = Settings::load(ui.ctx());
        IndicesWidget::new(&data_frame)
            .precision(settings.precision)
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
pub(crate) mod state;

mod table;
