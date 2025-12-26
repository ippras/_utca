use egui::{
    Button, ComboBox, DragValue, Grid, InnerResponse, Label, Popup, PopupCloseBehavior, Response,
    ScrollArea, TextStyle, TextWrapMode, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig, SubMenuButton},
    vec2,
};
use egui_extras::{Column, TableBuilder};
use egui_l20n::ResponseExt as _;
use egui_phosphor::regular::{MINUS, PLUS, SORT_ASCENDING};
use lipid::prelude::*;
use std::{borrow::Cow, cmp::Ordering, convert::identity};

/// Fatty acid widget
pub(crate) struct FattyAcidWidget<'a> {
    fatty_acid: Option<&'a FattyAcid>,
    editable: bool,
    hover: bool,
}

impl<'a> FattyAcidWidget<'a> {
    pub(crate) fn new(fatty_acid: Option<&'a FattyAcid>) -> Self {
        Self {
            fatty_acid,
            editable: false,
            hover: false,
        }
    }

    pub(crate) fn editable(self, editable: bool) -> Self {
        Self { editable, ..self }
    }

    pub(crate) fn hover(self, hover: bool) -> Self {
        Self { hover, ..self }
    }
}

impl FattyAcidWidget<'_> {
    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<Option<FattyAcid>> {
        let mut inner = None;
        let text = self
            .fatty_acid
            .map_or_default(|fatty_acid| fatty_acid.delta().to_string());
        let mut response = if self.editable {
            let mut changed = false;
            let mut response = match self.fatty_acid {
                Some(fatty_acid) => {
                    let mut fatty_acid = fatty_acid.clone();
                    let button = Button::new(&text)
                        .min_size(vec2(ui.available_width(), ui.spacing().interact_size.y));
                    MenuButton::from_button(button)
                        .config(
                            MenuConfig::new()
                                .close_behavior(PopupCloseBehavior::CloseOnClickOutside),
                        )
                        .ui(ui, |ui| {
                            if content(&mut fatty_acid)(ui).changed() {
                                inner = Some(fatty_acid);
                                changed = true;
                            }
                        })
                        .0
                }
                None => {
                    let response = ui.add_sized(
                        vec2(ui.available_width(), ui.spacing().interact_size.y),
                        Label::new("None"),
                    );
                    response.context_menu(|ui| {
                        if ui.button("Some").clicked() {
                            inner = Some(Default::default());
                            changed = true;
                            ui.close();
                        }
                    });
                    response
                }
            };
            if changed {
                response.mark_changed();
            };
            response
        } else {
            ui.label(&text)
        };
        if self.hover {
            response = response.on_hover_text(&text);
        }
        InnerResponse::new(inner, response)
    }
}

impl Widget for FattyAcidWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

fn content(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        let response = ui
            .horizontal(|ui| {
                let mut response = ui.button(SORT_ASCENDING);
                if response.clicked() {
                    fatty_acid
                        .unsaturated
                        .sort_by_cached_key(|unsaturated| unsaturated.index);
                    response.mark_changed();
                }
                response |= carbon(fatty_acid)(ui);
                response |= unsaturated(fatty_acid)(ui);
                response
            })
            .inner;
        ui.separator();
        response | new_indices(fatty_acid)(ui)
    }
}

fn carbon(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        ui.label("Carbon");
        ui.add(DragValue::new(&mut fatty_acid.carbon).update_while_editing(false))
            .on_hover_localized("carbon.hover")
    }
}

fn unsaturated(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        ui.label("Unsaturated");
        let mut unsaturated = fatty_acid.unsaturated.len();
        let response = ui
            .add(
                DragValue::new(&mut unsaturated)
                    .clamp_existing_to_range(true)
                    .range(0..=fatty_acid.carbon.saturating_sub(1))
                    .update_while_editing(false),
            )
            .on_hover_localized("unsaturated.hover");
        if response.changed() {
            loop {
                match unsaturated.cmp(&fatty_acid.unsaturated.len()) {
                    Ordering::Less => {
                        fatty_acid.unsaturated.pop();
                    }
                    Ordering::Equal => break,
                    Ordering::Greater => {
                        fatty_acid.unsaturated.push(Unsaturated {
                            index: Some(0),
                            parity: Some(false),
                            triple: Some(false),
                        });
                    }
                }
            }
        }
        response
    }
}

fn new_indices(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        let mut response = ui.response();
        let height = ui.text_style_height(&TextStyle::Body);
        let width = ui.spacing().combo_width;
        ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
        TableBuilder::new(ui)
            .column(Column::auto().at_least(width / 2.0))
            .column(Column::auto().at_least(width))
            .column(Column::auto().at_least(width / 2.0))
            .header(0.0, |_| {})
            .body(|mut body| {
                for unsaturated in &mut fatty_acid.unsaturated {
                    body.row(height, |mut row| {
                        // Index
                        row.col(|ui| {
                            let text = match unsaturated.index {
                                Some(index) => Cow::Owned(index.to_string()),
                                None => Cow::Borrowed("?"),
                            };
                            let inner_response = ui.menu_button(text, |ui| {
                                ui.set_min_width(ui.spacing().combo_width / 2.0);
                                ui.set_max_height(ui.spacing().combo_height);
                                let mut changed = false;
                                ScrollArea::vertical().show(ui, |ui| {
                                    for selected in 1..fatty_acid.carbon {
                                        changed |= ui
                                            .selectable_value(
                                                &mut unsaturated.index,
                                                Some(selected),
                                                selected.to_string(),
                                            )
                                            .changed();
                                    }
                                    changed |= ui
                                        .selectable_value(&mut unsaturated.index, None, "?")
                                        .changed();
                                });
                                changed
                            });
                            response |= inner_response.response;
                            if inner_response.inner.is_some_and(identity) {
                                response.mark_changed();
                            }
                        });
                        // Triple
                        row.col(|ui| {
                            let text = match unsaturated.triple {
                                Some(false) => "Olefinic",
                                Some(true) => "Acetylenic",
                                None => "?",
                            };
                            let inner_response = ui.menu_button(text, |ui| {
                                let mut changed = false;
                                changed |= ui
                                    .selectable_value(
                                        &mut unsaturated.triple,
                                        Some(false),
                                        "Olefinic",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(
                                        &mut unsaturated.triple,
                                        Some(true),
                                        "Acetylenic",
                                    )
                                    .changed();
                                changed |= ui
                                    .selectable_value(&mut unsaturated.triple, None, "?")
                                    .changed();
                                changed
                            });
                            response |= inner_response.response;
                            if inner_response.inner.is_some_and(identity) {
                                response.mark_changed();
                            }
                        });
                        // Parity
                        row.col(|ui| {
                            if let Some(false) = unsaturated.triple {
                                let text = match unsaturated.parity {
                                    Some(false) => "Cis",
                                    Some(true) => "Trans",
                                    None => "?",
                                };
                                let inner_response = ui.menu_button(text, |ui| {
                                    let mut changed = false;
                                    changed |= ui
                                        .selectable_value(
                                            &mut unsaturated.parity,
                                            Some(false),
                                            "Cis",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(
                                            &mut unsaturated.parity,
                                            Some(true),
                                            "Trans",
                                        )
                                        .changed();
                                    changed |= ui
                                        .selectable_value(&mut unsaturated.parity, None, "?")
                                        .changed();
                                    changed
                                });
                                response |= inner_response.response;
                                if inner_response.inner.is_some_and(identity) {
                                    response.mark_changed();
                                }
                            }
                        });
                    });
                }
            });
        response
    }
}

fn indices(fatty_acid: &mut FattyAcid) -> impl FnMut(&mut Ui) -> Response {
    |ui| {
        let mut response = ui.response();
        Grid::new(ui.next_auto_id()).show(ui, |ui| {
            for unsaturated in &mut fatty_acid.unsaturated {
                // Index
                let text = match unsaturated.index {
                    Some(index) => Cow::Owned(index.to_string()),
                    None => Cow::Borrowed("None"),
                };
                let inner_response = ComboBox::from_id_salt(ui.auto_id_with("Index"))
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        let mut changed = false;
                        for selected in 1..fatty_acid.carbon {
                            changed |= ui
                                .selectable_value(
                                    &mut unsaturated.index,
                                    Some(selected),
                                    selected.to_string(),
                                )
                                .changed();
                        }
                        changed |= ui
                            .selectable_value(&mut unsaturated.index, None, "None")
                            .changed();
                        changed
                    });
                response |= inner_response.response;
                if inner_response.inner.is_some_and(identity) {
                    response.mark_changed();
                }
                // Triple
                let text = match unsaturated.triple {
                    Some(false) => "Olefinic",
                    Some(true) => "Acetylenic",
                    None => "None",
                };
                let inner_response = ComboBox::from_id_salt(ui.auto_id_with("Triple"))
                    .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                    .selected_text(text)
                    .show_ui(ui, |ui| {
                        let mut changed = false;
                        changed |= ui
                            .selectable_value(&mut unsaturated.triple, Some(false), "Olefinic")
                            .changed();
                        changed |= ui
                            .selectable_value(&mut unsaturated.triple, Some(true), "Acetylenic")
                            .changed();
                        changed |= ui
                            .selectable_value(&mut unsaturated.triple, None, "None")
                            .changed();
                        changed
                    });
                response |= inner_response.response;
                if inner_response.inner.is_some_and(identity) {
                    response.mark_changed();
                }
                if let Some(false) = unsaturated.triple {
                    // Parity
                    let text = match unsaturated.parity {
                        Some(false) => "Cis",
                        Some(true) => "Trans",
                        None => "None",
                    };
                    let inner_response = ComboBox::from_id_salt(ui.auto_id_with("Parity"))
                        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                        .selected_text(text)
                        .show_ui(ui, |ui| {
                            let mut changed = false;
                            changed |= ui
                                .selectable_value(&mut unsaturated.parity, Some(false), "Cis")
                                .changed();
                            changed |= ui
                                .selectable_value(&mut unsaturated.parity, Some(true), "Trans")
                                .changed();
                            changed |= ui
                                .selectable_value(&mut unsaturated.parity, None, "None")
                                .changed();
                            changed
                        });
                    response |= inner_response.response;
                    if inner_response.inner.is_some_and(identity) {
                        response.mark_changed();
                    }
                }
                ui.end_row();
            }
        });
        response
    }
}

// impl<'a> FattyAcidContent<'a> {
//     fn new(id_salt: Id, fatty_acid: &'a mut FattyAcid) -> Self {
//         Self {
//             id_salt,
//             fatty_acid,
//         }
//     }

//     fn show(&mut self, ui: &mut Ui) -> Response {
//         let widgets = if ui.visuals().dark_mode {
//             Widgets::dark()
//         } else {
//             Widgets::light()
//         };
//         ui.visuals_mut().widgets.inactive.weak_bg_fill = widgets.hovered.weak_bg_fill;
//         ui.visuals_mut().widgets.hovered.bg_stroke = widgets.hovered.bg_stroke;
//         let mut state: State = ui.data_mut(|data| data.get_temp(self.id_salt).unwrap_or_default());
//         let mut outer_response = ui.allocate_response(Default::default(), Sense::hover());
//         let openness = ui.ctx().animate_bool(self.id_salt, state.is_opened);
//         ui.horizontal(|ui| {
//             // Carbons
//             let response = ui
//                 .add(DragValue::new(&mut self.fatty_acid.carbons))
//                 .on_hover_localized("carbons.hover");
//             outer_response |= response;
//             // Unsaturated
//             let mut unsaturated = self.fatty_acid.unsaturated.len();
//             let response = ui
//                 .add(
//                     DragValue::new(&mut unsaturated)
//                         .range(0..=self.fatty_acid.carbons)
//                         .clamp_existing_to_range(true),
//                 )
//                 .on_hover_localized("unsaturated.hover");
//             if response.changed() {
//                 loop {
//                     match unsaturated.cmp(&self.fatty_acid.unsaturated.len()) {
//                         Ordering::Less => {
//                             self.fatty_acid.unsaturated.pop();
//                         }
//                         Ordering::Equal => break,
//                         Ordering::Greater => {
//                             self.fatty_acid.unsaturated.push(Unsaturated {
//                                 index: Some(0),
//                                 isomerism: Some(Isomerism::Cis),
//                                 unsaturation: Some(Unsaturation::One),
//                             });
//                         }
//                     }
//                 }
//             }
//             outer_response |= response;
//             if unsaturated == 0 {
//                 ui.disable();
//             }
//             // let (_, response) = ui.allocate_exact_size(Vec2::splat(ui.spacing().interact_size.y), Sense::click());
//             let (_, response) = ui.allocate_exact_size(Vec2::splat(10.0), Sense::click());
//             collapsing_header::paint_default_icon(ui, openness, &response);
//             if response.clicked() {
//                 state.is_opened ^= true;
//             }
//             outer_response |= response;
//         });
//         if 0.0 < openness {
//             ui.separator();
//             if !self.fatty_acid.unsaturated.is_empty() {
//                 let response = UnsaturatedContent::new(self.id_salt, &mut self.fatty_acid).show(ui);
//                 outer_response |= response;
//             }
//         }
//         ui.data_mut(|data| data.insert_temp(self.id_salt, state));
//         outer_response
//     }
// }

// /// Unsaturated content
// struct UnsaturatedContent<'a> {
//     id_salt: Id,
//     fatty_acid: &'a mut FattyAcid,
// }

// impl<'a> UnsaturatedContent<'a> {
//     fn new(id_salt: Id, fatty_acid: &'a mut FattyAcid) -> Self {
//         Self {
//             id_salt,
//             fatty_acid,
//         }
//     }

//     fn show(&mut self, ui: &mut Ui) -> Response {
//         let mut outer_response = ui.allocate_response(Default::default(), Sense::hover());
//         Grid::new(ui.auto_id_with(self.id_salt)).show(ui, |ui| {
//             let bounds = self.fatty_acid.bounds();
//             for unsaturated in &mut self.fatty_acid.unsaturated {
//                 // Index
//                 let response = ui.add(
//                     DragValue::new(unsaturated.index.get_or_insert_default())
//                         .range(0..=bounds)
//                         .clamp_existing_to_range(true)
//                         .update_while_editing(false),
//                 );
//                 outer_response |= response;
//                 ui.horizontal(|ui| {
//                     // Unsaturation
//                     let (text, hover_text) = match &unsaturated.unsaturation {
//                         Some(Unsaturation::One) => (EQUALS, "Double bounds"),
//                         Some(Unsaturation::Two) => (LIST, "Triple bounds"),
//                         None => (ASTERISK, "Any number of bounds"),
//                     };
//                     let mut response = ui.button(text).on_hover_text(hover_text);
//                     if response.clicked() {
//                         unsaturated.unsaturation = match unsaturated.unsaturation {
//                             None => Some(Unsaturation::One),
//                             Some(Unsaturation::One) => Some(Unsaturation::Two),
//                             Some(Unsaturation::Two) => None,
//                         };
//                         response.mark_changed();
//                     }
//                     let min_size = response.rect.size();
//                     outer_response |= response;
//                     // Isomerism
//                     let (text, hover_text) = match &unsaturated.isomerism {
//                         Some(Isomerism::Cis) => ("C", "Cis"),
//                         Some(Isomerism::Trans) => ("T", "Trans"),
//                         None => (ASTERISK, "Any isomerism"),
//                     };
//                     let mut response = ui
//                         .add(Button::new(text).min_size(min_size))
//                         .on_hover_text(hover_text);
//                     if response.clicked() {
//                         unsaturated.isomerism = match unsaturated.isomerism {
//                             None => Some(Isomerism::Cis),
//                             Some(Isomerism::Cis) => Some(Isomerism::Trans),
//                             Some(Isomerism::Trans) => None,
//                         };
//                         response.mark_changed();
//                     }
//                     outer_response |= response;
//                 });
//                 ui.end_row();
//             }
//         });
//         outer_response
//     }
// }

// #[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
// struct State {
//     is_opened: bool,
// }
