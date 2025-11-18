use self::{correlations::Correlations, indices::Indices, table::TableView};
use super::{Behavior, MARGIN};
#[cfg(feature = "markdown")]
use crate::r#const::markdown::CORRELATIONS;
use crate::{
    app::{
        computers::calculation::{
            Computed as CalculationComputed, Key as CalculationKey,
            correlations::{
                Computed as CalculationCorrelationsComputed, Key as CalculationCorrelationsKey,
            },
            indices::{Computed as CalculationIndicesComputed, Key as CalculationIndicesKey},
        },
        identifiers::COMPOSE,
        states::calculation::{Settings, State},
    },
    export::ron,
    utils::{
        HashedDataFrame, HashedMetaDataFrame,
        egui::UiExt as _,
        metadata::{authors, date, name},
    },
};
use anyhow::Result;
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea, TextStyle,
    TopBottomPanel, Ui, Window, util::hash,
};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    CALCULATOR, FLOPPY_DISK, INTERSECT_THREE, LIST, MATH_OPERATIONS, SIGMA, SLIDERS_HORIZONTAL, X,
};
use egui_tiles::{TileId, UiResponse};
use lipid::prelude::*;
use metadata::{AUTHORS, DATE, DEFAULT_VERSION, Metadata, NAME, VERSION, polars::MetaDataFrame};
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use tracing::instrument;

const ID_SOURCE: &str = "Calculation";

/// Calculation pane
#[derive(Deserialize, Serialize)]
pub(crate) struct Pane {
    source: Vec<HashedMetaDataFrame>,
    target: HashedDataFrame,
}

impl Pane {
    pub(crate) fn new(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self {
            source: frames,
            target: HashedDataFrame {
                data_frame: DataFrame::empty(),
                hash: 0,
            },
        }
    }

    pub(crate) const fn icon() -> &'static str {
        CALCULATOR
    }

    pub(crate) fn title(&self, index: Option<usize>) -> String {
        self.title_with_separator(index, " ")
    }

    fn title_with_separator(&self, index: Option<usize>, separator: &str) -> String {
        match index {
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
    pub(super) fn ui(
        &mut self,
        ui: &mut Ui,
        behavior: &mut Behavior,
        tile_id: TileId,
    ) -> UiResponse {
        let mut state = State::load(ui.ctx(), Id::new(tile_id));
        let response = TopBottomPanel::top(ui.auto_id_with("Pane"))
            .show_inside(ui, |ui| {
                MenuBar::new()
                    .ui(ui, |ui| {
                        ScrollArea::horizontal()
                            .show(ui, |ui| {
                                ui.set_height(
                                    ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
                                );
                                ui.visuals_mut().button_frame = false;
                                if ui.button(RichText::new(X).heading()).clicked() {
                                    behavior.close = Some(tile_id);
                                }
                                ui.separator();
                                self.top(ui, &mut state)
                            })
                            .inner
                    })
                    .inner
            })
            .inner;
        CentralPanel::default()
            .frame(Frame::central_panel(&ui.style()))
            .show_inside(ui, |ui| {
                let _ = self.central(ui, &mut state);
                self.windows(ui, &mut state);
            });
        if let Some(id) = behavior.close {
            state.remove(ui.ctx(), Id::new(id));
        } else {
            state.store(ui.ctx(), Id::new(tile_id));
        }
        if response.dragged() {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        let mut response = ui.heading(Self::icon()).on_hover_ui(|ui| {
            ui.label(ui.localize("Calculation"));
        });
        response |= ui.heading(self.title(state.settings.index));
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
                        &mut state.settings.index,
                        Some(index),
                        self.source[index].meta.format(".").to_string(),
                    )
                    .clicked()
            }
            ui.selectable_value(
                &mut state.settings.index,
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
        ui.reset(&mut state.settings.table.reset_state);
        // Resize
        ui.resize(&mut state.settings.table.resizable);
        ui.separator();
        // Settings
        ui.settings(&mut state.windows.open_settings);
        ui.separator();
        // Sum tables
        ui.menu_button(RichText::new(SIGMA).heading(), |ui| {
            ui.toggle_value(
                &mut state.windows.open_correlations,
                (
                    RichText::new(SIGMA).heading(),
                    RichText::new(ui.localize("Correlation?PluralCategory=other")).heading(),
                ),
            )
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Correlation.hover"));
            });
            ui.toggle_value(
                &mut state.windows.open_indices,
                (
                    RichText::new(SIGMA).heading(),
                    RichText::new(ui.localize("Index?PluralCategory=other")).heading(),
                ),
            )
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Index.hover"));
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
            let _ = self.composition(ui, state);
        }
        ui.separator();
        // Save
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            let title = self.title_with_separator(state.settings.index, ".");
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
                let _ = self.save_ron(&title, state);
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
        response
    }

    #[instrument(skip_all, err)]
    fn save_ron(&mut self, title: &str, state: &mut State) -> Result<()> {
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
        let meta = match state.settings.index {
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
    fn composition(&mut self, ui: &mut Ui, state: &mut State) -> PolarsResult<()> {
        let mut frames = Vec::with_capacity(self.source.len());
        for index in 0..self.source.len() {
            let meta = self.source[index].meta.clone();
            let HashedDataFrame { data_frame, hash } = ui.memory_mut(|memory| {
                memory
                    .caches
                    .cache::<CalculationComputed>()
                    .get(CalculationKey {
                        index: Some(index),
                        ..CalculationKey::new(&self.source, &state.settings)
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
        ui.data_mut(|data| data.insert_temp(Id::new(COMPOSE), frames));
        Ok(())
    }

    #[instrument(skip_all, err)]
    fn central(&mut self, ui: &mut Ui, state: &mut State) -> PolarsResult<()> {
        self.target = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationComputed>()
                .get(CalculationKey::new(&self.source, &state.settings))
        });
        state.settings.fatty_acids = self.target[LABEL]
            .str()?
            .into_no_null_iter()
            .map(ToOwned::to_owned)
            .collect();
        TableView::new(&self.target, state).show(ui);
        Ok(())
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.correlations(ui, state);
        self.indices(ui, state);
        self.settings(ui, state);
    }

    fn correlations(&mut self, ui: &mut Ui, state: &mut State) {
        let response = Window::new(format!("{SLIDERS_HORIZONTAL} Calculation correlations"))
            .id(ui.auto_id_with(ID_SOURCE).with("Correlations"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_correlations)
            .show(ui.ctx(), |ui| {
                self.correlations_content(ui, &mut state.settings)
            });
        #[allow(unused_variables)]
        if let Some(inner_response) = response {
            #[cfg(feature = "markdown")]
            inner_response.response.on_hover_ui(|ui| {
                ui.markdown(CORRELATIONS);
            });
        }
    }

    #[instrument(skip_all, err)]
    fn correlations_content(&mut self, ui: &mut Ui, settings: &mut Settings) -> PolarsResult<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationCorrelationsComputed>()
                .get(CalculationCorrelationsKey::new(&self.target, settings))
        });
        Correlations::new(&data_frame, settings).show(ui);
        Ok(())
    }

    fn indices(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SIGMA} Calculation indices"))
            .id(ui.auto_id_with(ID_SOURCE).with("Indices"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_indices)
            .show(ui.ctx(), |ui| self.indices_content(ui, &state.settings));
    }

    #[instrument(skip_all, err)]
    fn indices_content(&mut self, ui: &mut Ui, settings: &Settings) -> PolarsResult<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationIndicesComputed>()
                .get(CalculationIndicesKey::new(&self.target, settings))
        });
        Indices::new(&data_frame, settings).show(ui).inner
    }

    fn settings(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Calculation settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                state.settings.show(ui);
            });
    }
}

mod correlations;
mod indices;
mod table;
