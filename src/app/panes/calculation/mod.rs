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
    CALCULATOR, FLOPPY_DISK, INTERSECT_THREE, LIST, SIGMA, SLIDERS_HORIZONTAL, X,
};
use egui_tiles::{TileId, UiResponse};
use lipid::prelude::*;
use metadata::{AUTHORS, DATE, DEFAULT_VERSION, Metadata, NAME, VERSION, polars::MetaDataFrame};
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, from_fn};
use tracing::instrument;

const ID_SOURCE: &str = "Calculation";

/// Calculation pane
#[derive(Deserialize, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    frames: Vec<HashedMetaDataFrame>,
    target: HashedDataFrame,
}

impl Pane {
    pub(crate) fn new(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self {
            id: None,
            frames,
            target: HashedDataFrame {
                data_frame: DataFrame::empty(),
                hash: 0,
            },
        }
    }

    pub(crate) fn title(&self, index: Option<usize>) -> String {
        self.title_with_separator(index, " ")
    }

    fn title_with_separator(&self, index: Option<usize>, separator: &str) -> String {
        match index {
            Some(index) => self.frames[index].meta.format(separator).to_string(),
            None => {
                format_list_truncated!(
                    self.frames.iter().map(|frame| frame.meta.format(separator)),
                    2
                )
            }
        }
    }

    fn id(&self) -> impl Display {
        from_fn(|f| {
            if let Some(id) = self.id {
                write!(f, "{id:?}-")?;
            }
            write!(f, "{}", hash(&self.frames))
        })
    }
}

impl Pane {
    pub(super) fn ui(
        &mut self,
        ui: &mut Ui,
        behavior: &mut Behavior,
        tile_id: TileId,
    ) -> UiResponse {
        let id = *self.id.get_or_insert_with(|| ui.next_auto_id());
        let mut state = State::load(ui.ctx(), id);
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
        if behavior.close == Some(tile_id) {
            state.remove(ui.ctx(), id);
        } else {
            state.store(ui.ctx(), id);
        }
        if response.dragged() {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        let mut response = ui.heading(CALCULATOR).on_hover_ui(|ui| {
            ui.label(ui.localize("Calculation"));
        });
        response |= ui.heading(self.title(state.settings.index));
        response = response
            .on_hover_text(format!("{}/{:x}", self.id(), self.target.hash))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        self.list_button(ui, state);
        ui.separator();
        // Reset
        ui.reset_button(&mut state.settings.table.reset_state);
        // Resize
        ui.resize_button(&mut state.settings.table.resizable);
        ui.separator();
        // Settings
        ui.settings_button(&mut state.windows.open_settings);
        ui.separator();
        self.sum_button(ui, state);
        ui.separator();
        self.composition_button(ui, state);
        ui.separator();
        self.save_button(ui, state);
        ui.separator();
        response
    }

    fn list_button(&self, ui: &mut Ui, state: &mut State) {
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            let mut clicked = false;
            for index in 0..self.frames.len() {
                clicked |= ui
                    .selectable_value(
                        &mut state.settings.index,
                        Some(index),
                        self.frames[index].meta.format(".").to_string(),
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
    }

    fn sum_button(&self, ui: &mut Ui, state: &mut State) {
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
    }

    fn composition_button(&self, ui: &mut Ui, state: &mut State) {
        if ui
            .button(RichText::new(INTERSECT_THREE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Composition"));
            })
            .clicked()
        {
            let _ = self.composition(ui, state);
        }
    }

    #[instrument(skip_all, err)]
    fn composition(&self, ui: &mut Ui, state: &mut State) -> PolarsResult<()> {
        let mut frames = Vec::with_capacity(self.frames.len());
        for index in 0..self.frames.len() {
            let meta = self.frames[index].meta.clone();
            let HashedDataFrame { data_frame, hash } = ui.memory_mut(|memory| {
                memory
                    .caches
                    .cache::<CalculationComputed>()
                    .get(CalculationKey {
                        index: Some(index),
                        ..CalculationKey::new(&self.frames, &state.settings)
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

    fn save_button(&self, ui: &mut Ui, state: &mut State) {
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
    }

    #[instrument(skip_all, err)]
    fn save_ron(&self, title: &str, state: &mut State) -> Result<()> {
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
                let mut meta = self.frames[index].meta.clone();
                meta.retain(|key, _| key != "ARROW:schema");
                meta
            }
            None => {
                let mut meta = Metadata::default();
                // let name =
                //     format_list!(self.source.iter().filter_map(|frame| frame.meta.get(NAME)));
                meta.insert(NAME.to_owned(), name(&self.frames));
                meta.insert(AUTHORS.to_owned(), authors(&self.frames));
                meta.insert(DATE.to_owned(), date(&self.frames));
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
    fn central(&mut self, ui: &mut Ui, state: &mut State) -> PolarsResult<()> {
        self.target = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationComputed>()
                .get(CalculationKey::new(&self.frames, &state.settings))
        });
        state.settings.fatty_acids = self.target[LABEL]
            .str()?
            .into_no_null_iter()
            .map(ToOwned::to_owned)
            .collect();
        state.settings.threshold.manual =
            self.target["Filter"].bool()?.into_no_null_iter().collect();
        TableView::new(&self.target, state).show(ui);
        Ok(())
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.correlations_window(ui, state);
        self.indices_window(ui, state);
        self.settings_window(ui, state);
    }

    fn correlations_window(&mut self, ui: &mut Ui, state: &mut State) {
        if let Some(inner_response) =
            Window::new(format!("{SLIDERS_HORIZONTAL} Calculation correlations"))
                .id(ui.auto_id_with(ID_SOURCE).with("Correlations"))
                .default_pos(ui.next_widget_position())
                .open(&mut state.windows.open_correlations)
                .scroll([true, true])
                .show(ui.ctx(), |ui| {
                    self.correlations_content(ui, &mut state.settings)
                })
        {
            #[allow(unused_variables)]
            let response = inner_response
                .response
                .on_hover_text(self.title(state.settings.index).to_string())
                .on_hover_text(self.id().to_string());
            #[cfg(feature = "markdown")]
            response.on_hover_ui(|ui| {
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

    fn indices_window(&mut self, ui: &mut Ui, state: &mut State) {
        if let Some(inner_response) = Window::new(format!("{SIGMA} Calculation indices"))
            .id(ui.auto_id_with(ID_SOURCE).with("Indices"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_indices)
            .show(ui.ctx(), |ui| self.indices_content(ui, &state.settings))
        {
            inner_response
                .response
                .on_hover_text(self.title(state.settings.index).to_string())
                .on_hover_text(self.id().to_string());
        }
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

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        if let Some(inner_response) =
            Window::new(format!("{SLIDERS_HORIZONTAL} Calculation settings"))
                .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
                .default_pos(ui.next_widget_position())
                .open(&mut state.windows.open_settings)
                .show(ui.ctx(), |ui| {
                    state.settings.show(ui);
                })
        {
            inner_response
                .response
                .on_hover_text(self.title(state.settings.index).to_string())
                .on_hover_text(self.id().to_string());
        }
    }
}

mod correlations;
mod indices;
mod table;
