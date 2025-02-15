use egui::{
    Color32, CursorIcon, DragAndDrop, Frame, Grid, Label, RichText, Sides, Stroke, TextStyle, Ui,
    Widget, menu::bar, util::hash,
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROWS_OUT_CARDINAL, CHECK, TRASH};
use metadata::MetaDataFrame;
// use re_ui::list_item::{LabelContent, ListItem};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Data {
    pub(crate) frames: Vec<MetaDataFrame>,
    #[serde(skip)]
    pub(crate) selected: HashSet<u64>,
    state: State,
}

impl Data {
    pub(crate) fn selected(&self) -> Vec<MetaDataFrame> {
        self.frames
            .iter()
            .filter_map(|frame| {
                self.selected
                    .contains(&hash(frame))
                    .then_some(frame.clone())
            })
            .collect()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.frames.is_empty()
    }

    pub(crate) fn add(&mut self, frame: MetaDataFrame) {
        self.frames.push(frame);
    }

    pub(crate) fn delete(&mut self, index: usize) {
        self.selected.remove(&hash(self.frames.remove(index)));
    }
}

impl Data {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        // Header
        // bar(ui, |ui| {
        //     ui.heading(ui.localize("files"));
        //     ui.separator();
        //     // Check all
        //     if ui
        //         .button(RichText::new(CHECK).heading())
        //         .on_hover_text(ui.localize("check-all"))
        //         .clicked()
        //     {
        //         if let Some(&checked) = self.checked.get(0) {
        //             self.checked = vec![!checked; self.checked.len()];
        //         }
        //     }
        //     ui.separator();
        //     // Delete all
        //     if ui
        //         .button(RichText::new(TRASH).heading())
        //         .on_hover_text(ui.localize("delete-all"))
        //         .clicked()
        //     {
        //         *self = Default::default();
        //     }
        //     ui.separator();
        // });
        // Body
        ui.separator();
        let mut swap: Option<(usize, usize)> = None;
        ui.dnd_drop_zone::<usize, ()>(Frame::new(), |ui| {
            for (index, frame) in self.frames.iter().enumerate() {
                let hash = hash(frame);
                // Draw the item
                let text = if let Some(version) = &frame.meta.version {
                    &format!("{} {version}", frame.meta.name)
                } else {
                    &frame.meta.name
                };
                let response = ui
                    .dnd_drag_source(ui.auto_id_with(index), index, |ui| {
                        ui.label(index.to_string())
                    })
                    .response
                    .on_hover_ui(|ui| {
                        Grid::new(ui.next_auto_id()).show(ui, |ui| {
                            ui.label("Rows");
                            ui.label(frame.data.height().to_string());
                        });
                    });
                // Handle item selection
                if response.clicked() {
                    if ui.input(|input| input.modifiers.command) {
                        if self.selected.contains(&hash) {
                            self.selected.remove(&hash);
                        } else {
                            self.selected.insert(hash);
                        }
                    } else {
                        self.selected.clear();
                        self.selected.insert(hash);
                    }
                }
                // Drag-and-drop of multiple items not supported, so dragging resets
                // selection to single item.
                if response.drag_started() {
                    self.selected.clear();
                    self.selected.insert(hash);
                    response.dnd_set_drag_payload(index);
                }
                // Detect drag situation and run the swap if it ends.
                let source_item_position_index = DragAndDrop::payload(ui.ctx()).map(|index| *index);
                if let Some(source_item_position_index) = source_item_position_index {
                    ui.ctx().set_cursor_icon(CursorIcon::Grabbing);
                    let (top, bottom) = response.rect.split_top_bottom_at_fraction(0.5);
                    let (insert_y, target) = if ui.rect_contains_pointer(top) {
                        (Some(top.top()), Some(index))
                    } else if ui.rect_contains_pointer(bottom) {
                        (Some(bottom.bottom()), Some(index + 1))
                    } else {
                        (None, None)
                    };
                    if let (Some(insert_y), Some(target)) = (insert_y, target) {
                        ui.painter().hline(
                            ui.cursor().x_range(),
                            insert_y,
                            (2.0, egui::Color32::WHITE),
                        );
                        // note: can't use `response.drag_released()` because we not
                        // the item which started the drag
                        if ui.input(|input| input.pointer.any_released()) {
                            swap = Some((source_item_position_index, target));
                            DragAndDrop::clear_payload(ui.ctx());
                        }
                    }
                }
                // ui.horizontal(|ui| {
                //     let response = ui
                //         .dnd_drag_source(ui.auto_id_with(index), index, |ui| {
                //             ui.label(index.to_string())
                //         })
                //         .response;
                //     // Detect drops onto this item
                //     if let (Some(pointer), Some(hovered_payload)) = (
                //         ui.input(|input| input.pointer.interact_pos()),
                //         response.dnd_hover_payload::<usize>(),
                //     ) {
                //         let rect = response.rect;
                //         // Preview insertion:
                //         let stroke = Stroke::new(1.0, Color32::WHITE);
                //         let to = if *hovered_payload == index {
                //             // We are dragged onto ourselves
                //             ui.painter().hline(rect.x_range(), rect.center().y, stroke);
                //             index
                //         } else if pointer.y < rect.center().y {
                //             // Above us
                //             ui.painter().hline(rect.x_range(), rect.top(), stroke);
                //             index
                //         } else {
                //             // Below us
                //             ui.painter().hline(rect.x_range(), rect.bottom(), stroke);
                //             index + 1
                //         };
                //         if let Some(from) = response.dnd_release_payload() {
                //             // The user dropped onto this item.
                //             self.state.drag_and_drop_row = Some((*from, to));
                //         }
                //     }
                //     ui.checkbox(&mut self.checked[index], "");
                //     let text = if let Some(version) = &frame.meta.version {
                //         &format!("{} {version}", frame.meta.name)
                //     } else {
                //         &frame.meta.name
                //     };
                //     ui.add(Label::new(text).truncate()).on_hover_ui(|ui| {
                //         Grid::new(ui.next_auto_id()).show(ui, |ui| {
                //             ui.label("Rows");
                //             ui.label(frame.data.height().to_string());
                //         });
                //     });
                //     if ui.button(TRASH).clicked() {
                //         self.state.delete_row = Some(index);
                //     }
                // });
            }
        });
        // Handle the swap command (if any)
        if let Some((source, target)) = swap {
            if source != target {
                let frame = self.frames.remove(source);
                if source < target {
                    self.frames.insert(target - 1, frame);
                } else {
                    self.frames.insert(target, frame);
                }
            }
        }

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
        //         });
        // });
        if let Some((from, to)) = self.state.drag_and_drop_row {
            self.frames.swap(from, to);
            // self.move_row(from, to).unwrap();
            self.state.drag_and_drop_row = None;
        }
        if let Some(index) = self.state.delete_row {
            self.delete(index);
            ui.ctx().request_repaint();
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
    pub(crate) drag_and_drop_row: Option<(usize, usize)>,
}
