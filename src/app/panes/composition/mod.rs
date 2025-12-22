use self::{plot::PlotView, sum::symmetry::Symmetry, table::TableView};
use super::{Behavior, MARGIN};
use crate::{
    app::{
        computers::composition::{
            Computed as CompositionComputed, Key as CompositionKey,
            filtered::{Computed as FilteredComputed, Key as FilteredKey},
            species::{Computed as SpeciesComputed, Key as SpeciesKey},
            sum::symmetry::{Computed as SymmetryComputed, Key as SymmetryKey},
            table::{Computed as TableComputed, Key as TableKey},
            unique::{Computed as UniqueComputed, Key as UniqueKey},
        },
        states::composition::{ID_SOURCE, Settings, State, View},
        widgets::butons::{ResetButton, ResizeButton, SettingsButton},
    },
    export::{ron, xlsx},
    text::Text,
    utils::{
        HashedDataFrame, HashedMetaDataFrame,
        metadata::{authors, date, description, name},
    },
};
use anyhow::Result;
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea, TextStyle,
    TopBottomPanel, Ui, Widget as _, Window, util::hash,
};
use egui_l20n::prelude::*;
use egui_phosphor::regular::{FLOPPY_DISK, INTERSECT_THREE, LIST, SIGMA, SLIDERS_HORIZONTAL, X};
use egui_tiles::{TileId, UiResponse};
use lipid::prelude::*;
use metadata::{
    AUTHORS, DATE, DEFAULT_VERSION, DESCRIPTION, Metadata, NAME, VERSION, polars::MetaDataFrame,
};
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use std::fmt::{Debug, Display, from_fn};
use tracing::instrument;

/// Composition pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    frames: Vec<HashedMetaDataFrame>,
    // Слишком большой размер вызывает задержку при десериализации, при открытие
    // программы.
    #[serde(skip)]
    species: HashedDataFrame,
    #[serde(skip)]
    target: HashedDataFrame,
}

impl Pane {
    pub(crate) fn new(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self {
            id: None,
            frames,
            species: HashedDataFrame::EMPTY,
            target: HashedDataFrame::EMPTY,
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
        _ = self.init(ui, &mut state);
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

    fn init(&mut self, ui: &mut Ui, state: &mut State) {
        // Species
        self.species = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<SpeciesComputed>()
                .get(SpeciesKey::new(&self.frames, &state.settings))
        });
    }

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        let mut response = ui
            .heading(INTERSECT_THREE)
            .on_hover_localized("Composition");
        response |= ui.heading(self.title(state.settings.index));
        response = response
            .on_hover_text(format!("{}/{:x}", self.id(), self.target.hash))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        self.list_button(ui, state);
        ui.separator();
        ResetButton::new(&mut state.reset_table_state).ui(ui);
        ResizeButton::new(&mut state.settings.resizable).ui(ui);
        ui.separator();
        SettingsButton::new(&mut state.windows.open_settings).ui(ui);
        ui.separator();
        // Sum
        self.sum_button(ui, state);
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
        .on_hover_localized("List");
    }

    /// Sum button
    fn sum_button(&self, ui: &mut Ui, state: &mut State) {
        ui.menu_button(RichText::new(SIGMA).heading(), |ui| {
            ui.toggle_value(
                &mut state.windows.open_sum,
                (
                    RichText::new(SIGMA).heading(),
                    RichText::new(ui.localize("Symmetry")).heading(),
                ),
            )
            .on_hover_localized("Symmetry.hover");
        });
    }

    /// Save button
    fn save_button(&self, ui: &mut Ui, state: &State) {
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            let meta = self.meta(state);
            let name = meta.format(".");
            if ui
                .button((FLOPPY_DISK, "RON"))
                .on_hover_localized("Save")
                .on_hover_ui(|ui| {
                    ui.label(format!("{name}.tag.utca.ron"));
                })
                .clicked()
            {
                _ = self.save_ron(ui, &name, state);
            }
            if ui
                .button((FLOPPY_DISK, "XLSX"))
                .on_hover_localized("Save")
                .on_hover_ui(|ui| {
                    ui.label(format!("{name}.tag.utca.xlsx"));
                })
                .clicked()
            {
                _ = self.save_xlsx(ui, &name, state);
            }
        });
    }

    #[instrument(skip_all, err)]
    fn save_ron(&self, ui: &mut Ui, name: impl Debug + Display, state: &State) -> Result<()> {
        let meta = self.meta(state);
        let data = self
            .species
            .data_frame
            .select([LABEL, TRIACYLGLYCEROL, "Value"])?;
        // let data = self.data(ui, state)?;
        println!("data: {data}");
        let frame = MetaDataFrame::new(meta, HashedDataFrame::new(data)?);
        ron::save(&frame, &format!("{name}.tag.utca.ron"))?;
        Ok(())
    }
    // ┌─────────────────────┬─────────────────────┬──────────┬──────────┬──────────┬─────────────────────┐
    // │ Label               ┆ Triacylglycerol     ┆ Value[0] ┆ Value[1] ┆ Value[2] ┆ Value               │
    // │ ---                 ┆ ---                 ┆ ---      ┆ ---      ┆ ---      ┆ ---                 │
    // │ struct[3]           ┆ struct[3]           ┆ f64      ┆ f64      ┆ f64      ┆ struct[3]           │
    // ╞═════════════════════╪═════════════════════╪══════════╪══════════╪══════════╪═════════════════════╡

    #[instrument(skip_all, err)]
    fn save_xlsx(&self, ui: &mut Ui, name: impl Debug + Display, state: &State) -> Result<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory.caches.cache::<FilteredComputed>().get(FilteredKey {
                data_frame: &self.target,
                settings: &state.settings,
            })
        });
        let data_frame = data_frame.data_frame.unnest(["Keys"], None)?;
        _ = xlsx::save(&data_frame, &format!("{name}.utca.xlsx"));
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
            memory.caches.cache::<FilteredComputed>().get(FilteredKey {
                data_frame: &self.target,
                settings: &state.settings,
            })
        });
        data_frame
            .data_frame
            .lazy()
            .select([col("Species").explode(ExplodeOptions {
                empty_as_null: true,
                keep_nulls: true,
            })])
            .unnest(by_name(["Species"], true), None)
            .sort(
                ["Value"],
                SortMultipleOptions::default().with_order_descending(true),
            )
            .collect()
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        // Composition
        self.target = ui.memory_mut(|memory| {
            let key = CompositionKey::new(&self.species, &state.settings);
            HashedDataFrame {
                data_frame: memory.caches.cache::<CompositionComputed>().get(key),
                hash: hash(key),
            }
        });
        // Filtered
        let data_frame = ui.memory_mut(|memory| {
            memory.caches.cache::<FilteredComputed>().get(FilteredKey {
                data_frame: &self.target,
                settings: &state.settings,
            })
        });
        // Table
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<TableComputed>()
                .get(TableKey::new(&data_frame, &state.settings))
        });
        match state.settings.view {
            View::Plot => PlotView::new(&data_frame, state).show(ui),
            View::Table => TableView::new(&data_frame, state).show(ui),
        }
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.settings_window(ui, state);
        self.sum_window(ui, state);
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        if state.settings.discriminants.is_empty() {
            let unique = ui.memory_mut(|memory| {
                memory.caches.cache::<UniqueComputed>().get(UniqueKey {
                    frames: &self.frames,
                })
            });
            state.settings.discriminants = unique.into_iter().collect();
        }
        Window::new(format!("{SLIDERS_HORIZONTAL} Composition settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_settings)
            .show(ui.ctx(), |ui| {
                state.settings.show(ui, &self.target);
            });
    }

    fn sum_window(&mut self, ui: &mut Ui, state: &mut State) {
        Window::new(format!("{SIGMA} Composition symmetry"))
            .id(ui.auto_id_with(ID_SOURCE).with("Symmetry"))
            .default_pos(ui.next_widget_position())
            .open(&mut state.windows.open_sum)
            .show(ui.ctx(), |ui| self.sum_content(ui, &state.settings));
    }

    #[instrument(skip_all, err)]
    fn sum_content(&mut self, ui: &mut Ui, settings: &Settings) -> PolarsResult<()> {
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<SymmetryComputed>()
                .get(SymmetryKey::new(&self.species, settings))
        });
        Symmetry::new(&data_frame, settings).show(ui).inner
    }
}

mod plot;
mod sum;
mod table;
