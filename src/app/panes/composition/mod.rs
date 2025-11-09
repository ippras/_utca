use self::{plot::PlotView, table::TableView};
use super::{Behavior, MARGIN};
use crate::{
    app::{
        computers::{
            CompositionComputed, CompositionKey, CompositionSpeciesComputed, CompositionSpeciesKey,
            FilteredCompositionComputed, FilteredCompositionKey, UniqueCompositionComputed,
            UniqueCompositionKey,
        },
        states::composition::{Settings, State, View},
    },
    export::{self, ron, xlsx},
    text::Text,
    utils::{
        HashedDataFrame, HashedMetaDataFrame,
        egui::UiExt as _,
        metadata::{authors, date, description, name},
    },
};
use anyhow::Result;
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea, TextStyle,
    TopBottomPanel, Ui, Window, util::hash,
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, FLOPPY_DISK, GEAR, INTERSECT_THREE, LIST, SIGMA, X,
};
use egui_tiles::{TileId, UiResponse};
use metadata::{
    AUTHORS, DATE, DEFAULT_VERSION, DESCRIPTION, Metadata, NAME, VERSION, polars::MetaDataFrame,
};
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use tracing::instrument;

const ID_SOURCE: &str = "Composition";

/// Composition pane
#[derive(Default, Deserialize, Serialize)]
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
        INTERSECT_THREE
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

    pub(super) fn hash(&self) -> u64 {
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
                self.central(ui, &mut state);
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
        self.top_content(ui, state)
    }

    fn top_content(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        let mut response = ui.heading(Self::icon()).on_hover_ui(|ui| {
            ui.label(ui.localize("Composition"));
        });
        response |= ui.heading(self.title(state.settings.index));
        response = response
            .on_hover_text(format!("{:x}/{:x}", self.hash(), self.target.hash))
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
            clicked |= ui
                .selectable_value(
                    &mut state.settings.index,
                    None,
                    "Mean ± standard deviations",
                )
                .clicked();
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
        ui.reset(&mut state.reset_table_state);
        // Resize
        ui.resize(&mut state.settings.resizable);
        ui.separator();
        // Settings Parameters
        ui.parameters(&mut state.windows.open_settings);
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
                    ui.label(&format!("{title}.tag.utca.ron"));
                })
                .clicked()
            {
                let _ = self.save_ron(ui, &title, state);
            }
            // if ui
            //     .button("PARQUET")
            //     .on_hover_ui(|ui| {
            //         ui.label(ui.localize("Save"));
            //     })
            //     .on_hover_ui(|ui| {
            //         ui.label(&format!("{title}.tag.utca.parquet"));
            //     })
            //     .clicked()
            // {
            //     let mut data_frame = ui.memory_mut(|memory| {
            //         memory.caches.cache::<FilteredCompositionComputed>().get(
            //             FilteredCompositionKey {
            //                 data_frame: &self.target,
            //                 settings: &state.settings,
            //             },
            //         )
            //     });
            //     let mut data = data_frame
            //         .data_frame
            //         .lazy()
            //         .select([col("Species").explode()])
            //         .unnest(by_name(["Species"], true), None)
            //         .sort(
            //             ["Value"],
            //             SortMultipleOptions::default().with_order_descending(true),
            //         )
            //         .collect()
            //         .unwrap();
            //     println!("data_frame unnest: {}", data.clone());
            //     // let _ = parquet::save_data(&mut data_frame, &format!("{title}.utca.parquet"));
            //     let mut meta = self.source[0].meta.clone();
            //     meta.retain(|key, _| key != "ARROW:schema");
            //     println!("meta: {meta:?}");
            //     let mut frame = MetaDataFrame::new(meta, data);
            //     let _ = export::parquet::save(&mut frame, &format!("{title}.tag.utca.parquet"));
            // }
            if ui
                .button("XLSX")
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(&format!("{title}.tag.utca.xlsx"));
                })
                .clicked()
            {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<FilteredCompositionComputed>().get(
                        FilteredCompositionKey {
                            data_frame: &self.target,
                            settings: &state.settings,
                        },
                    )
                });
                let data_frame = data_frame.data_frame.unnest(["Keys"], None).unwrap();
                let _ = xlsx::save(&data_frame, &format!("{title}.utca.xlsx"));
            }
        });
        ui.separator();
        // View
        ui.menu_button(RichText::new(state.settings.view.icon()).heading(), |ui| {
            ui.selectable_value(&mut state.settings.view, View::Plot, View::Plot.text())
                .on_hover_text(View::Plot.hover_text());
            ui.selectable_value(&mut state.settings.view, View::Table, View::Table.text())
                .on_hover_text(View::Table.hover_text());
        })
        .response
        .on_hover_text(state.settings.view.hover_text());
        ui.end_row();
        ui.separator();
        response
    }

    #[instrument(skip_all, err)]
    fn save_ron(&mut self, ui: &mut Ui, title: &str, state: &mut State) -> Result<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<FilteredCompositionComputed>()
                .get(FilteredCompositionKey {
                    data_frame: &self.target,
                    settings: &state.settings,
                })
        });
        let data = data_frame
            .data_frame
            .lazy()
            .select([col("Species").explode()])
            .unnest(by_name(["Species"], true), None)
            .sort(
                ["Value"],
                SortMultipleOptions::default().with_order_descending(true),
            )
            .collect()?;
        let meta = match state.settings.index {
            Some(index) => self.source[index].meta.clone(),
            None => {
                let mut meta = Metadata::default();
                meta.insert(NAME.to_owned(), name(&self.source));
                meta.insert(AUTHORS.to_owned(), authors(&self.source));
                meta.insert(DATE.to_owned(), date(&self.source));
                meta.insert(DESCRIPTION.to_owned(), description(&self.source));
                meta.insert(VERSION.to_owned(), DEFAULT_VERSION.to_owned());
                meta
            }
        };
        let frame = MetaDataFrame::new(meta, data);
        ron::save(&frame, &format!("{title}.tag.utca.ron"))?;
        Ok(())
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        self.central_content(ui, state);
        self.windows(ui, state);
    }

    fn central_content(&mut self, ui: &mut Ui, state: &mut State) {
        // Species
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CompositionSpeciesComputed>()
                .get(CompositionSpeciesKey::new(&self.source, &state.settings))
        });
        // Composition
        self.target = ui.memory_mut(|memory| {
            let key = CompositionKey::new(&data_frame, &state.settings);
            HashedDataFrame {
                data_frame: memory.caches.cache::<CompositionComputed>().get(key),
                hash: hash(key),
            }
        });
        // Filtered
        let filtered_data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<FilteredCompositionComputed>()
                .get(FilteredCompositionKey {
                    data_frame: &self.target,
                    settings: &state.settings,
                })
        });
        match state.settings.view {
            View::Plot => PlotView::new(&filtered_data_frame, state).show(ui),
            View::Table => TableView::new(&filtered_data_frame, state).show(ui),
        }
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.settings(ui, state);
    }

    fn settings(&mut self, ui: &mut Ui, state: &mut State) {
        if state.settings.parameters.discriminants.is_empty() {
            let unique = ui.memory_mut(|memory| {
                memory
                    .caches
                    .cache::<UniqueCompositionComputed>()
                    .get(UniqueCompositionKey {
                        frames: &self.source,
                    })
            });
            state.settings.parameters.discriminants = unique.into_iter().collect();
        }
        Window::new(format!("{GEAR} Composition settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                state.settings.show(ui, &self.target);
            });
    }
}

mod plot;
mod table;
