use crate::{
    app::{identifiers::CONFIGURE, panes::configuration::Pane as ConfigurationPane},
    utils::HashedMetaDataFrame,
};
use egui::{CentralPanel, Color32, Id, Label, MenuBar, RichText, ScrollArea, TopBottomPanel, Ui};
use egui_dnd::dnd;
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::{CHECK, DOTS_SIX_VERTICAL, TRASH};
use metadata::egui::MetadataWidget;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Data {
    pub frames: Vec<HashedMetaDataFrame>,
    pub selected: HashSet<HashedMetaDataFrame>,
}

impl Data {
    pub fn selected(&self) -> Vec<HashedMetaDataFrame> {
        self.frames
            .iter()
            .filter_map(|frame| self.selected.contains(frame).then_some(frame.clone()))
            .collect()
    }

    pub fn add(&mut self, frame: HashedMetaDataFrame) {
        if !self.frames.contains(&frame) {
            self.frames.push(frame);
        }
    }
}

impl Data {
    pub fn show(&mut self, ui: &mut Ui) {
        TopBottomPanel::top(ui.auto_id_with("LeftPane").with("TopPane")).show_inside(ui, |ui| {
            MenuBar::new().ui(ui, |ui| {
                self.top(ui);
            });
        });
        CentralPanel::default().show_inside(ui, |ui| {
            ScrollArea::vertical().auto_shrink(false).show(ui, |ui| {
                self.central(ui);
            });
        });
    }

    fn top(&mut self, ui: &mut Ui) {
        ui.heading(ui.localize("loaded-files"))
            .on_hover_localized("loaded-files.hover");
        ui.separator();
        // Toggle
        if ui
            .button(RichText::new(CHECK).heading())
            .on_hover_localized("toggle-all")
            .on_hover_localized("toggle-all.hover")
            .clicked()
        {
            if self.selected.is_empty() {
                self.selected = self.frames.iter().cloned().collect();
            } else {
                self.selected.clear();
            }
        }
        ui.separator();
        let enabled = !self.selected.is_empty();
        // Delete
        ui.add_enabled_ui(enabled, |ui| {
            if ui
                .button(RichText::new(TRASH).heading())
                .on_hover_localized("DeleteSelected.hover")
                .clicked()
            {
                self.frames.retain(|frame| !self.selected.remove(frame));
            }
        });
        ui.separator();
        // Configuration
        ui.add_enabled_ui(enabled, |ui| {
            if ui
                .button(RichText::new(ConfigurationPane::icon()).heading())
                .on_hover_localized("configuration")
                .clicked()
            {
                ui.data_mut(|data| data.insert_temp(Id::new(CONFIGURE), self.selected()));
            }
        });
        ui.separator();
    }

    fn central(&mut self, ui: &mut Ui) {
        ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
        dnd(ui, ui.next_auto_id()).show_vec(&mut self.frames, |ui, frame, handle, _state| {
            ui.horizontal(|ui| {
                handle.ui(ui, |ui| {
                    ui.label(DOTS_SIX_VERTICAL);
                });
                let mut changed = false;
                // Checkbox
                let mut checked = self.selected.contains(frame);
                changed |= ui.checkbox(&mut checked, "").changed();
                // Label
                let text = frame.meta.format(" ").to_string();
                changed |= ui
                    .add(Label::new(text).truncate())
                    .on_hover_ui(|ui| {
                        MetadataWidget::new(&frame.meta).show(ui);
                    })
                    .clicked();
                if changed {
                    if self.selected.contains(frame) {
                        self.selected.remove(frame);
                    } else {
                        self.selected.insert(frame.clone());
                    }
                }
            });
        });
    }
}
