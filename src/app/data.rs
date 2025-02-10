use egui::{Frame, Grid, Label, Response, RichText, Sides, Ui, Widget, menu::bar};
use egui_dnd::dnd;
use egui_extras::{Column, TableBuilder};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROWS_OUT_CARDINAL, CHECK, TRASH};
use metadata::MetaDataFrame;
use serde::{Deserialize, Serialize};
use std::iter::zip;

/// Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) frames: Vec<MetaDataFrame>,
    pub(crate) checked: Vec<bool>,
}

impl Data {
    pub(crate) fn checked(&self) -> Vec<MetaDataFrame> {
        zip(&self.frames, &self.checked)
            .filter_map(|(frame, checked)| checked.then_some(frame.clone()))
            .collect()
    }

    pub(crate) fn is_empty(&self) -> bool {
        assert_eq!(self.frames.len(), self.checked.len());
        self.frames.is_empty()
    }

    pub(crate) fn add(&mut self, frame: MetaDataFrame) {
        self.frames.push(frame);
        self.checked.push(false);
    }

    pub(crate) fn delete(&mut self, index: usize) {
        self.frames.remove(index);
        self.checked.remove(index);
    }
}

impl Data {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        // Header
        bar(ui, |ui| {
            ui.heading(ui.localize("files"));
            ui.separator();
            // Check all
            if ui
                .button(RichText::new(CHECK).heading())
                .on_hover_text(ui.localize("check-all"))
                .clicked()
            {
                if let Some(&checked) = self.checked.get(0) {
                    self.checked = vec![!checked; self.checked.len()];
                }
            }
            ui.separator();
            // Delete all
            if ui
                .button(RichText::new(TRASH).heading())
                .on_hover_text(ui.localize("delete-all"))
                .clicked()
            {
                *self = Default::default();
            }
            ui.separator();
        });
        // Body
        ui.separator();
        let mut delete = None;
        ui.dnd_drop_zone::<usize, ()>(Frame::default(), |ui| {
            TableBuilder::new(ui)
                .column(Column::auto().resizable(true))
                .column(Column::remainder())
                .body(|mut body| {
                    let ui = body.ui_mut();
                    dnd(ui, ui.next_auto_id()).show_vec(
                        &mut self.frames,
                        |ui, frame, handle, state| {
                            ui.horizontal(|ui| {
                                Sides::new().show(
                                    ui,
                                    |ui| {
                                        handle.ui(ui, |ui| {
                                            let _ = ui.label(ARROWS_OUT_CARDINAL);
                                        });
                                        ui.checkbox(&mut self.checked[state.index], "");
                                        let text = if let Some(version) = &frame.meta.version {
                                            &format!("{} {version}", frame.meta.name)
                                        } else {
                                            &frame.meta.name
                                        };
                                        ui.add(Label::new(text).truncate()).on_hover_ui(|ui| {
                                            Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                                ui.label("Rows");
                                                ui.label(frame.data.height().to_string());
                                            });
                                        });
                                    },
                                    |ui| {
                                        if ui.button(TRASH).clicked() {
                                            delete = Some(state.index);
                                        }
                                    },
                                );
                            });
                        },
                    );
                });
        });
        // dnd(ui, ui.next_auto_id()).show_vec(&mut self.frames, |ui, frame, handle, state| {
        //     ui.horizontal(|ui| {
        //         Sides::new().show(
        //             ui,
        //             |ui| {
        //                 handle.ui(ui, |ui| {
        //                     let _ = ui.label(ARROWS_OUT_CARDINAL);
        //                 });
        //                 ui.checkbox(&mut self.checked[state.index], "");
        //                 let text = if let Some(version) = &frame.meta.version {
        //                     &format!("{} {version}", frame.meta.name)
        //                 } else {
        //                     &frame.meta.name
        //                 };
        //                 ui.add(Label::new(text).truncate()).on_hover_ui(|ui| {
        //                     Grid::new(ui.next_auto_id()).show(ui, |ui| {
        //                         ui.label("Rows");
        //                         ui.label(frame.data.height().to_string());
        //                     });
        //                 });
        //             },
        //             |ui| {
        //                 if ui.button(TRASH).clicked() {
        //                     delete = Some(state.index);
        //                 }
        //             },
        //         );
        //     });
        // });
        if let Some(index) = delete {
            self.delete(index);
            ui.ctx().request_repaint();
        }
    }
}
