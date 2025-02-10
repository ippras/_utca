use self::{
    plot::PlotView,
    settings::Settings,
    state::{State, View},
    table::TableView,
};
use super::PaneDelegate;
use crate::app::{
    computers::{
        FattyAcidCompositionComputed, FattyAcidCompositionKey, TriacylglycerolCompositionComputed,
        TriacylglycerolCompositionKey,
    },
    text::Text,
};
use egui::{CursorIcon, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CHECK, GEAR, INTERSECT_THREE, LIST,
};
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};

const ID_SOURCE: &str = "Composition";

/// Composition pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) source: Vec<MetaDataFrame>,
    pub(crate) target: DataFrame,
    pub(crate) settings: Settings,
    state: State,
}

impl Pane {
    pub(crate) fn new(frames: Vec<MetaDataFrame>, index: Option<usize>) -> Self {
        Self {
            source: frames,
            target: DataFrame::empty(),
            settings: Settings::new(index),
            state: State::new(),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        INTERSECT_THREE
    }

    pub(crate) fn title(&self) -> String {
        match self.settings.index {
            Some(index) => self.source[index].meta.title(),
            None => format_list_truncated!(self.source.iter().map(|frame| frame.meta.title()), 2),
        }
    }

    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui
            .heading(Self::icon())
            .on_hover_text(ui.localize("composition"));
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
            memory
                .caches
                .cache::<TriacylglycerolCompositionComputed>()
                .get(TriacylglycerolCompositionKey {
                    frames: &self.source,
                    settings: &self.settings,
                })
        });
        match self.state.view {
            View::Plot => PlotView::new(&self.target, &self.settings, &mut self.state).show(ui),
            View::Table => TableView::new(&self.target, &self.settings, &mut self.state).show(ui),
        }
    }

    fn windows(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Composition settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .default_pos(ui.next_widget_position())
            .open(&mut self.state.open_settings_window)
            .show(ui.ctx(), |ui| {
                // let data_frame = ui.memory_mut(|memory| {
                //     memory.caches.cache::<FattyAcidCompositionComputed>().get(
                //         FattyAcidCompositionKey {
                //             frames: &self.source,
                //             settings: &self.settings,
                //         },
                //     )
                // });
                self.settings.show(ui, &self.target);
                let enabled = hash(&self.settings.confirmed) != hash(&self.settings.unconfirmed);
                ui.add_enabled_ui(enabled, |ui| {
                    ui.horizontal(|ui| {
                        if ui
                            .button(RichText::new(format!("{ARROWS_CLOCKWISE} Reset")).heading())
                            .clicked()
                        {
                            self.settings.unconfirmed = self.settings.confirmed.clone();
                        }
                        if ui
                            .button(RichText::new(format!("{CHECK} Confirm")).heading())
                            .clicked()
                        {
                            self.settings.confirmed = self.settings.unconfirmed.clone();
                        }
                    });
                });
            });
    }

    fn hash(&self) -> u64 {
        hash(&self.source)
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
