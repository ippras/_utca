use self::{plot::PlotView, table::TableView};
use super::{Behavior, MARGIN};
use crate::{
    app::{
        computers::composition::{
            Computed as CompositionComputed, Key as CompositionKey,
            filtered::{Computed as FilteredCompositionComputed, Key as FilteredCompositionKey},
            species::{Computed as CompositionSpeciesComputed, Key as CompositionSpeciesKey},
            unique::{Computed as UniqueCompositionComputed, Key as UniqueCompositionKey},
        },
        states::composition::{State, View},
    },
    export::{ron, xlsx},
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
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, FLOPPY_DISK, INTERSECT_THREE, LIST, SIGMA,
    SLIDERS_HORIZONTAL, X,
};
use egui_tiles::{TileId, UiResponse};
use metadata::{
    AUTHORS, DATE, DEFAULT_VERSION, DESCRIPTION, Metadata, NAME, VERSION, polars::MetaDataFrame,
};
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, from_fn};
use tracing::instrument;

const ID_SOURCE: &str = "Composition";

/// Composition pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    frames: Vec<HashedMetaDataFrame>,
    // Слишком большой размер вызывает задержку при десериализации, при открытие
    // программы.
    #[serde(skip)]
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
                self.central(ui, &mut state);
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
        let mut response = ui.heading(INTERSECT_THREE).on_hover_ui(|ui| {
            ui.label(ui.localize("Composition"));
        });
        response |= ui.heading(self.title(state.settings.index));
        response = response
            .on_hover_text(format!("{}/{:x}", self.id(), self.target.hash))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        self.list_button(ui, state);
        ui.separator();
        // Reset
        ui.reset_button(&mut state.reset_table_state);
        // Resize
        ui.resize_button(&mut state.settings.resizable);
        ui.separator();
        // Settings
        ui.settings_button(&mut state.windows.open_settings);
        ui.separator();
        // Save
        self.save_button(ui, state);
        ui.separator();
        // View
        self.view_button(ui, state);
        ui.separator();
        response
    }

    /// View button
    fn view_button(&self, ui: &mut Ui, state: &mut State) {
        ui.menu_button(RichText::new(state.settings.view.icon()).heading(), |ui| {
            ui.selectable_value(&mut state.settings.view, View::Plot, View::Plot.text())
                .on_hover_text(View::Plot.hover_text());
            ui.selectable_value(&mut state.settings.view, View::Table, View::Table.text())
                .on_hover_text(View::Table.hover_text());
        })
        .response
        .on_hover_text(state.settings.view.hover_text());
    }

    /// List button
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
    }

    /// Save button
    fn save_button(&self, ui: &mut Ui, state: &State) {
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            let meta = self.meta(state);
            let name = meta.format(".");
            if ui
                .button((FLOPPY_DISK, "RON"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(format!("{name}.tag.utca.ron"));
                })
                .clicked()
            {
                let _ = self.save_ron(ui, &name, state);
            }
            if ui
                .button((FLOPPY_DISK, "XLSX"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(format!("{name}.tag.utca.xlsx"));
                })
                .clicked()
            {
                let _ = self.save_xlsx(ui, &name, state);
            }
        });
    }

    #[instrument(skip_all, err)]
    fn save_ron(&self, ui: &mut Ui, name: impl Debug + Display, state: &State) -> Result<()> {
        let meta = self.meta(state);
        let data = self.data(ui, state)?;
        let frame = MetaDataFrame::new(meta, data);
        ron::save(&frame, &format!("{name}.tag.utca.ron"))?;
        Ok(())
    }

    #[instrument(skip_all, err)]
    fn save_xlsx(&self, ui: &mut Ui, name: impl Debug + Display, state: &State) -> Result<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<FilteredCompositionComputed>()
                .get(FilteredCompositionKey {
                    data_frame: &self.target,
                    settings: &state.settings,
                })
        });
        let data_frame = data_frame.data_frame.unnest(["Keys"], None)?;
        let _ = xlsx::save(&data_frame, &format!("{name}.utca.xlsx"));
        Ok(())
    }

    fn meta(&self, state: &State) -> Metadata {
        match state.settings.index {
            Some(index) => self.frames[index].meta.clone(),
            None => {
                let mut meta = Metadata::default();
                meta.insert(AUTHORS.to_owned(), authors(&self.frames));
                meta.insert(DATE.to_owned(), date(&self.frames));
                meta.insert(DESCRIPTION.to_owned(), description(&self.frames));
                meta.insert(NAME.to_owned(), name(&self.frames));
                meta.insert(VERSION.to_owned(), DEFAULT_VERSION.to_owned());
                meta
            }
        }
    }

    fn data(&self, ui: &Ui, state: &State) -> PolarsResult<DataFrame> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<FilteredCompositionComputed>()
                .get(FilteredCompositionKey {
                    data_frame: &self.target,
                    settings: &state.settings,
                })
        });
        data_frame
            .data_frame
            .lazy()
            .select([col("Species").explode()])
            .unnest(by_name(["Species"], true), None)
            .sort(
                ["Value"],
                SortMultipleOptions::default().with_order_descending(true),
            )
            .collect()
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        // Species
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CompositionSpeciesComputed>()
                .get(CompositionSpeciesKey::new(&self.frames, &state.settings))
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
        self.settings_window(ui, state);
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        if state.settings.discriminants.is_empty() {
            let unique = ui.memory_mut(|memory| {
                memory
                    .caches
                    .cache::<UniqueCompositionComputed>()
                    .get(UniqueCompositionKey {
                        frames: &self.frames,
                    })
            });
            state.settings.discriminants = unique.into_iter().collect();
        }
        if let Some(inner_response) =
            Window::new(format!("{SLIDERS_HORIZONTAL} Composition settings"))
                .id(ui.auto_id_with(ID_SOURCE))
                .default_pos(ui.next_widget_position())
                .open(&mut state.windows.open_settings)
                .show(ui.ctx(), |ui| {
                    state.settings.show(ui, &self.target);
                })
        {
            inner_response
                .response
                .on_hover_text(self.title(state.settings.index).to_string())
                .on_hover_text(self.id().to_string());
        }
    }
}

mod plot;
mod table;
