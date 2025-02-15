use egui::{
    Color32, Frame, Grid, Label, RichText, Sides, Stroke, TextStyle, TextWrapMode, Ui, Widget,
    menu::bar, util::hash,
};
use egui_extras::{Column, StripBuilder, TableBuilder};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROWS_OUT_CARDINAL, CHECK, TRASH};
use metadata::MetaDataFrame;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) frames: Vec<MetaDataFrame>,
    pub(crate) selected: HashSet<MetaDataFrame>,
}

impl Data {
    pub(crate) fn selected(&self) -> Vec<MetaDataFrame> {
        self.frames
            .iter()
            .filter_map(|frame| self.selected.contains(frame).then_some(frame.clone()))
            .collect()
    }

    pub(crate) fn is_empty(&self) -> bool {
        assert_eq!(self.frames.len(), self.selected.len());
        self.frames.is_empty()
    }

    pub(crate) fn add(&mut self, frame: MetaDataFrame) {
        self.frames.push(frame);
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
                if self.selected.is_empty() {
                    self.selected = self.frames.iter().cloned().collect();
                } else {
                    self.selected.clear();
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
        let mut swap = None;
        let mut delete = None;
        let height = TextStyle::Body.resolve(ui.style()).size;
        ui.dnd_drop_zone::<usize, ()>(Frame::new(), |ui| {
            ui.visuals_mut().widgets.inactive.bg_fill = Color32::TRANSPARENT;
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
            TableBuilder::new(ui)
                .columns(Column::auto().resizable(true), 3)
                // .column(Column::remainder())
                .body(|body| {
                    body.rows(height, self.frames.len(), |mut row| {
                        let index = row.index();
                        row.col(|ui| {
                            let response = ui
                                .dnd_drag_source(ui.auto_id_with(index), index, |ui| {
                                    ui.label(index.to_string())
                                })
                                .response;
                            // Detect drops onto this item
                            if let (Some(pointer), Some(hovered_payload)) = (
                                ui.input(|input| input.pointer.interact_pos()),
                                response.dnd_hover_payload::<usize>(),
                            ) {
                                let rect = response.rect;
                                // Preview insertion:
                                let stroke = Stroke::new(1.0, Color32::WHITE);
                                let to = if *hovered_payload == index {
                                    // We are dragged onto ourselves
                                    ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                                    index
                                } else if pointer.y < rect.center().y {
                                    // Above us
                                    ui.painter().hline(rect.x_range(), rect.top(), stroke);
                                    index
                                } else {
                                    // Below us
                                    ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                                    index + 1
                                };
                                if let Some(from) = response.dnd_release_payload() {
                                    // The user dropped onto this item.
                                    swap = Some((*from, to));
                                }
                            }
                        });
                        let frame = &self.frames[index];
                        row.col(|ui| {
                            let text = if let Some(version) = &frame.meta.version {
                                &format!("{} {version}", frame.meta.name)
                            } else {
                                &frame.meta.name
                            };
                            // ui.add(Label::new(text).truncate())
                            let response = ui
                                .selectable_label(self.selected.contains(frame), text)
                                .on_hover_ui(|ui| {
                                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                        ui.label("Rows");
                                        ui.label(frame.data.height().to_string());
                                    });
                                });
                            // Basic click and cmd/ctr-click
                            if response.clicked() {
                                if ui.input(|input| input.modifiers.command) {
                                    if self.selected.contains(frame) {
                                        self.selected.remove(frame);
                                    } else {
                                        self.selected.insert(frame.clone());
                                    }
                                } else {
                                    if self.selected.contains(frame) {
                                        self.selected.remove(&frame);
                                    } else {
                                        self.selected.insert(frame.clone());
                                    }
                                }
                            }
                        });
                        row.col(|ui| {
                            if ui.button(TRASH).clicked() {
                                delete = Some(index);
                            }
                        });
                        // ui.horizontal(|ui| {
                        //     Sides::new().show(
                        //         ui,
                        //         |ui| {
                        //             handle.ui(ui, |ui| {
                        //                 let _ = ui.label(ARROWS_OUT_CARDINAL);
                        //             });
                        //             ui.checkbox(&mut self.checked[state.index], "");
                        //             let text = if let Some(version) = &frame.meta.version {
                        //                 &format!("{} {version}", frame.meta.name)
                        //             } else {
                        //                 &frame.meta.name
                        //             };
                        //             ui.add(Label::new(text).truncate()).on_hover_ui(|ui| {
                        //                 Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        //                     ui.label("Rows");
                        //                     ui.label(frame.data.height().to_string());
                        //                 });
                        //             });
                        //         },
                        //         |ui| {
                        //             if ui.button(TRASH).clicked() {
                        //                 delete = Some(state.index);
                        //             }
                        //         },
                        //     );
                        // });
                    });
                    // let ui = body.ui_mut();
                    // dnd(ui, ui.next_auto_id()).show_vec(
                    //     &mut self.frames,
                    //     |ui, frame, handle, state| {
                    //         ui.horizontal(|ui| {
                    //             Sides::new().show(
                    //                 ui,
                    //                 |ui| {
                    //                     handle.ui(ui, |ui| {
                    //                         let _ = ui.label(ARROWS_OUT_CARDINAL);
                    //                     });
                    //                     ui.checkbox(&mut self.checked[state.index], "");
                    //                     let text = if let Some(version) = &frame.meta.version {
                    //                         &format!("{} {version}", frame.meta.name)
                    //                     } else {
                    //                         &frame.meta.name
                    //                     };
                    //                     ui.add(Label::new(text).truncate()).on_hover_ui(|ui| {
                    //                         Grid::new(ui.next_auto_id()).show(ui, |ui| {
                    //                             ui.label("Rows");
                    //                             ui.label(frame.data.height().to_string());
                    //                         });
                    //                     });
                    //                 },
                    //                 |ui| {
                    //                     if ui.button(TRASH).clicked() {
                    //                         delete = Some(state.index);
                    //                     }
                    //                 },
                    //             );
                    //         });
                    //     },
                    // );
                });
        });
        if let Some((from, to)) = swap {
            if from != to {
                let frame = self.frames.remove(from);
                if from < to {
                    self.frames.insert(to - 1, frame);
                } else {
                    self.frames.insert(to, frame);
                }
            }
        }
        if let Some(index) = delete {
            self.selected.remove(&self.frames.remove(index));
        }
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
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) delete_row: Option<usize>,
    pub(crate) swap: Option<(usize, usize)>,
}
