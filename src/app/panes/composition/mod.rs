use self::{
    plot::PlotView,
    settings::Settings,
    state::{State, View},
    table::TableView,
};
use super::PaneDelegate;
use crate::{
    app::{
        computers::{
            CompositionComputed, CompositionIndicesComputed, CompositionIndicesKey, CompositionKey,
            CompositionSpeciesComputed, CompositionSpeciesKey, FilteredCompositionComputed,
            FilteredCompositionKey, UniqueCompositionComputed, UniqueCompositionKey,
        },
        widgets::IndicesWidget,
    },
    export::{parquet, xlsx},
    text::Text,
    utils::Hashed,
};
use egui::{CursorIcon, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, FLOPPY_DISK, GEAR, INTERSECT_THREE, LIST, SIGMA,
};
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use tracing::instrument;

const ID_SOURCE: &str = "Composition";

/// Composition pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    source: Hashed<Vec<MetaDataFrame>>,
    target: Hashed<DataFrame>,
    settings: Settings,
    state: State,
}

impl Pane {
    pub(crate) fn new(frames: Vec<MetaDataFrame>, index: Option<usize>) -> Self {
        Self {
            source: Hashed::new(frames),
            target: Hashed {
                value: DataFrame::empty(),
                hash: 0,
            },
            settings: Settings::new(index),
            state: State::new(),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        INTERSECT_THREE
    }

    pub(crate) fn title(&self) -> String {
        self.title_with_separator(" ")
    }

    fn title_with_separator(&self, separator: &str) -> String {
        match self.settings.index {
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

    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui
            .heading(Self::icon())
            .on_hover_text(ui.localize("composition"));
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}/{:x}", self.source.hash, self.target.hash))
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
            clicked |= ui
                .selectable_value(&mut self.settings.index, None, "Mean ± standard deviations")
                .clicked();
            if clicked {
                ui.close();
            }
        })
        .response
        .on_hover_text(ui.localize("list"));
        ui.separator();
        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .clicked()
        {
            self.state.reset_table_state = true;
        }
        // Resize
        ui.toggle_value(
            &mut self.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_text(ui.localize("resize"));
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.open_settings_window,
            RichText::new(GEAR).heading(),
        );
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
        // Save
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            let title = self.title_with_separator(".");
            if ui
                .button("PARQUET")
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(&format!("{title}.utca.parquet"));
                })
                .clicked()
            {
                let mut data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<FilteredCompositionComputed>().get(
                        FilteredCompositionKey {
                            data_frame: &self.target,
                            settings: &self.settings,
                        },
                    )
                });
                data_frame = data_frame
                    .lazy()
                    .select([col("Species").explode()])
                    .unnest(by_name(["Species"], true))
                    .sort(
                        ["Value"],
                        SortMultipleOptions::default().with_order_descending(true),
                    )
                    .collect()
                    .unwrap();
                println!(
                    "data_frame unnest: {}",
                    data_frame
                        .clone()
                        .unnest(["FattyAcid"])
                        .unwrap()
                        .unnest(["StereospecificNumber1"])
                        .unwrap()
                );
                let _ = parquet::save_data(&mut data_frame, &format!("{title}.utca.parquet"));
            }
            if ui
                .button("XLSX")
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(&format!("{title}.utca.xlsx"));
                })
                .clicked()
            {
                let mut data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<FilteredCompositionComputed>().get(
                        FilteredCompositionKey {
                            data_frame: &self.target,
                            settings: &self.settings,
                        },
                    )
                });
                data_frame = data_frame.unnest(["Keys"]).unwrap();
                let _ = xlsx::save(&data_frame, &format!("{title}.utca.xlsx"));
            }
        });
        ui.separator();
        // View
        ui.menu_button(RichText::new(self.state.view.icon()).heading(), |ui| {
            ui.selectable_value(&mut self.state.view, View::Plot, View::Plot.text())
                .on_hover_text(View::Plot.hover_text());
            ui.selectable_value(&mut self.state.view, View::Table, View::Table.text())
                .on_hover_text(View::Table.hover_text());
        })
        .response
        .on_hover_text(self.state.view.hover_text());
        ui.end_row();
        ui.separator();
        response
    }

    fn body_content(&mut self, ui: &mut Ui) {
        // Species
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CompositionSpeciesComputed>()
                .get(CompositionSpeciesKey {
                    frames: &self.source,
                    index: self.settings.index,
                    ddof: self.settings.special.ddof,
                    method: self.settings.special.method,
                })
        });
        self.target = ui.memory_mut(|memory| {
            let key = CompositionKey {
                data_frame: &data_frame,
                settings: &self.settings,
            };
            Hashed {
                value: memory.caches.cache::<CompositionComputed>().get(key),
                hash: hash(key),
            }
        });
        let filtered_data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<FilteredCompositionComputed>()
                .get(FilteredCompositionKey {
                    data_frame: &self.target,
                    settings: &self.settings,
                })
        });
        match self.state.view {
            View::Plot => {
                PlotView::new(&filtered_data_frame, &self.settings, &mut self.state).show(ui)
            }
            View::Table => {
                TableView::new(&filtered_data_frame, &self.settings, &mut self.state).show(ui)
            }
        }
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui) {
        self.indices(ui);
        self.settings(ui);
    }

    fn indices(&mut self, ui: &mut Ui) {
        let mut open_indices_window = self.state.open_indices_window;
        Window::new(format!("{SIGMA} Composition indices"))
            .id(ui.auto_id_with(ID_SOURCE).with("Indices"))
            .open(&mut open_indices_window)
            .show(ui.ctx(), |ui| self.indices_content(ui));
        self.state.open_indices_window = open_indices_window;
    }

    #[instrument(skip_all, err)]
    fn indices_content(&mut self, ui: &mut Ui) -> PolarsResult<()> {
        // Species
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CompositionSpeciesComputed>()
                .get(CompositionSpeciesKey {
                    frames: &self.source,
                    index: self.settings.index,
                    ddof: self.settings.special.ddof,
                    method: self.settings.special.method,
                })
        });
        // Indices
        let data_frame = ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CompositionIndicesComputed>()
                .get(CompositionIndicesKey {
                    data_frame: Hashed {
                        value: &data_frame,
                        hash: hash(self.settings.index),
                    },
                    ddof: self.settings.special.ddof,
                })
        });
        IndicesWidget::new(&data_frame)
            .hover(true)
            .precision(Some(self.settings.precision))
            .show(ui)
            .inner
    }

    fn settings(&mut self, ui: &mut Ui) {
        if self.settings.special.discriminants.is_empty() {
            let unique = ui.memory_mut(|memory| {
                memory
                    .caches
                    .cache::<UniqueCompositionComputed>()
                    .get(UniqueCompositionKey {
                        frames: &self.source,
                    })
            });
            self.settings.special.discriminants = unique.into_iter().collect();
        }
        Window::new(format!("{GEAR} Composition settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .default_pos(ui.next_widget_position())
            .open(&mut self.state.open_settings_window)
            .show(ui.ctx(), |ui| {
                self.settings.show(ui, &self.target);
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

pub(crate) mod settings;

mod plot;
mod state;
mod table;
