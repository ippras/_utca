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
            CompositionComputed, CompositionKey, FilteredCompositionComputed,
            FilteredCompositionKey, UniqueCompositionComputed, UniqueCompositionKey,
        },
        text::Text,
    },
    export::{ipc, xlsx},
    utils::{Hashed, title},
};
use egui::{CursorIcon, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CHECK, FLOPPY_DISK, GEAR, INTERSECT_THREE, LIST,
};
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};

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
            Some(index) => title(&self.source[index].meta, separator),
            None => {
                format_list_truncated!(
                    self.source
                        .iter()
                        .map(|frame| title(&frame.meta, separator)),
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
                        self.source[index].meta.title(),
                    )
                    .clicked()
            }
            clicked |= ui
                .selectable_value(&mut self.settings.index, None, "Mean ± standard deviations")
                .clicked();
            if clicked {
                ui.close_menu();
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
        // Save
        ui.menu_button(RichText::new(FLOPPY_DISK).heading(), |ui| {
            let title = self.title_with_separator(".");
            if ui
                .button("IPC")
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("save"));
                })
                .on_hover_ui(|ui| {
                    ui.label(&format!("{title}.utca.ipc"));
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
                    .unnest([col("Species")])
                    .sort(
                        ["Value"],
                        SortMultipleOptions::default().with_order_descending(true),
                    )
                    .collect()
                    .unwrap();
                println!("data_frame: {data_frame}");
                let _ = ipc::save_data(&mut data_frame, &format!("{title}.utca.ipc"));
            };
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
                // if let Err(error) = self.save() {
                //     ui.ctx().error(error);
                // }
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
        self.target = ui.memory_mut(|memory| {
            let key = CompositionKey {
                frames: &self.source,
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

    fn windows(&mut self, ui: &mut Ui) {
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
        self.windows(ui);
        self.body_content(ui);
    }
}

pub(crate) mod settings;

mod plot;
mod state;
mod table;
