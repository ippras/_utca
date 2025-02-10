use egui::{
    Button, DragValue, Grid, Id, InnerResponse, Response, Sense, Ui, Vec2, Widget,
    collapsing_header, style::Widgets,
};
use egui_phosphor::regular::{ASTERISK, EQUALS, LIST};
use lipid::fatty_acid::{
    FattyAcid, FattyAcidExt as _, Isomerism, Unsaturated, Unsaturation,
    display::{COMMON, DisplayWithOptions},
};
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, hash::Hash};

/// Fatty acid widget
pub(crate) struct FattyAcidWidget<'a> {
    fatty_acid: Option<&'a mut FattyAcid>,
    id_salt: Id,
    editable: bool,
    hover: bool,
}

impl<'a> FattyAcidWidget<'a> {
    pub(crate) fn new(fatty_acid: Option<&'a mut FattyAcid>) -> Self {
        Self {
            fatty_acid,
            id_salt: Id::new("FattyAcid"),
            editable: false,
            hover: false,
        }
    }

    pub fn id_salt(mut self, id_salt: impl Hash) -> Self {
        self.id_salt = Id::new(id_salt);
        self
    }

    pub(crate) fn editable(self, editable: bool) -> Self {
        Self { editable, ..self }
    }

    pub(crate) fn hover(self) -> Self {
        Self {
            hover: true,
            ..self
        }
    }
}

impl FattyAcidWidget<'_> {
    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<Option<FattyAcid>> {
        let mut inner = None;
        // // Error
        // let value = match (self.fatty_acid)() {
        //     Ok(value) => value,
        //     Err(error) => {
        //         let response = ui.label("Error").on_hover_text(error.to_string());
        //         return InnerResponse::new(inner, response);
        //     }
        // };
        // None
        let Some(fatty_acid) = self.fatty_acid else {
            let mut response = ui.label("None");
            if self.editable {
                let mut changed = false;
                response.context_menu(|ui| {
                    if ui.button("Some").clicked() {
                        inner = Some(Default::default());
                        changed = true;
                        ui.close_menu();
                    }
                });
                if changed {
                    response.mark_changed();
                };
            }
            return InnerResponse::new(inner, response);
        };
        // Some
        let text = &format!("{:#}", fatty_acid.display(COMMON));
        let mut response = if self.editable {
            let mut changed = false;
            let mut response = ui.add_sized(
                [ui.available_width(), ui.spacing().interact_size.y],
                |ui: &mut Ui| {
                    ui.menu_button(text, |ui| {
                        let response = FattyAcidContent::new(self.id_salt, fatty_acid).show(ui);
                        inner = Some(fatty_acid.clone());
                        changed |= response.changed();
                    })
                    .response
                },
            );
            response.context_menu(|ui| {
                let response = ui.button("None");
                if response.clicked() {
                    inner = None;
                    changed = true;
                    ui.close_menu();
                }
            });
            if changed {
                response.mark_changed();
            };
            response
        } else {
            ui.label(text)
        };
        if self.hover {
            response = response.on_hover_text(text);
        }
        InnerResponse::new(inner, response)
    }
}

impl Widget for FattyAcidWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

/// Fatty acid content
struct FattyAcidContent<'a> {
    id_salt: Id,
    fatty_acid: &'a mut FattyAcid,
}

impl<'a> FattyAcidContent<'a> {
    fn new(id_salt: Id, fatty_acid: &'a mut FattyAcid) -> Self {
        Self {
            id_salt,
            fatty_acid,
        }
    }

    fn show(&mut self, ui: &mut Ui) -> Response {
        let widgets = if ui.visuals().dark_mode {
            Widgets::dark()
        } else {
            Widgets::light()
        };
        ui.visuals_mut().widgets.inactive.weak_bg_fill = widgets.hovered.weak_bg_fill;
        ui.visuals_mut().widgets.hovered.bg_stroke = widgets.hovered.bg_stroke;

        let mut state: State = ui.data_mut(|data| data.get_temp(self.id_salt).unwrap_or_default());

        let mut outer_response = ui.allocate_response(Default::default(), Sense::hover());
        let openness = ui.ctx().animate_bool(self.id_salt, state.is_opened);
        ui.horizontal(|ui| {
            // Carbons
            let response = ui
                .add(DragValue::new(&mut self.fatty_acid.carbons))
                .on_hover_text("Carbons");
            outer_response |= response;
            // Unsaturated
            let mut unsaturated = self.fatty_acid.unsaturated.len();
            let response = ui
                .add(
                    DragValue::new(&mut unsaturated)
                        .range(0..=self.fatty_acid.carbons)
                        .clamp_existing_to_range(true),
                )
                .on_hover_text("Unsaturated");
            if response.changed() {
                loop {
                    match unsaturated.cmp(&self.fatty_acid.unsaturated.len()) {
                        Ordering::Less => {
                            self.fatty_acid.unsaturated.pop();
                        }
                        Ordering::Equal => break,
                        Ordering::Greater => {
                            self.fatty_acid.unsaturated.push(Unsaturated {
                                index: Some(0),
                                isomerism: Some(Isomerism::Cis),
                                unsaturation: Some(Unsaturation::One),
                            });
                        }
                    }
                }
            }
            outer_response |= response;
            let (_, response) =
                ui.allocate_exact_size(Vec2::splat(ui.spacing().interact_size.y), Sense::click());
            collapsing_header::paint_default_icon(ui, openness, &response);
            if response.clicked() {
                state.is_opened ^= true;
            }
            outer_response |= response;
        });
        if 0.0 < openness {
            ui.separator();
            if !self.fatty_acid.unsaturated.is_empty() {
                let response = UnsaturatedContent::new(self.id_salt, &mut self.fatty_acid).show(ui);
                outer_response |= response;
            }
        }
        ui.data_mut(|data| data.insert_temp(self.id_salt, state));
        outer_response
    }
}

/// Unsaturated content
struct UnsaturatedContent<'a> {
    id_salt: Id,
    fatty_acid: &'a mut FattyAcid,
}

impl<'a> UnsaturatedContent<'a> {
    fn new(id_salt: Id, fatty_acid: &'a mut FattyAcid) -> Self {
        Self {
            id_salt,
            fatty_acid,
        }
    }

    fn show(&mut self, ui: &mut Ui) -> Response {
        let mut outer_response = ui.allocate_response(Default::default(), Sense::hover());
        Grid::new(ui.auto_id_with(self.id_salt)).show(ui, |ui| {
            let bounds = self.fatty_acid.bounds();
            for unsaturated in &mut self.fatty_acid.unsaturated {
                // Index
                let response = ui.add(
                    DragValue::new(unsaturated.index.get_or_insert_default())
                        .range(0..=bounds)
                        .clamp_existing_to_range(true)
                        .update_while_editing(false),
                );
                outer_response |= response;
                ui.horizontal(|ui| {
                    // Unsaturation
                    let (text, hover_text) = match &unsaturated.unsaturation {
                        Some(Unsaturation::One) => (EQUALS, "Double bounds"),
                        Some(Unsaturation::Two) => (LIST, "Triple bounds"),
                        None => (ASTERISK, "Any number of bounds"),
                    };
                    let mut response = ui.button(text).on_hover_text(hover_text);
                    if response.clicked() {
                        unsaturated.unsaturation = match unsaturated.unsaturation {
                            None => Some(Unsaturation::One),
                            Some(Unsaturation::One) => Some(Unsaturation::Two),
                            Some(Unsaturation::Two) => None,
                        };
                        response.mark_changed();
                    }
                    let min_size = response.rect.size();
                    outer_response |= response;
                    // Isomerism
                    let (text, hover_text) = match &unsaturated.isomerism {
                        Some(Isomerism::Cis) => ("C", "Cis"),
                        Some(Isomerism::Trans) => ("T", "Trans"),
                        None => (ASTERISK, "Any isomerism"),
                    };
                    let mut response = ui
                        .add(Button::new(text).min_size(min_size))
                        .on_hover_text(hover_text);
                    if response.clicked() {
                        unsaturated.isomerism = match unsaturated.isomerism {
                            None => Some(Isomerism::Cis),
                            Some(Isomerism::Cis) => Some(Isomerism::Trans),
                            Some(Isomerism::Trans) => None,
                        };
                        response.mark_changed();
                    }
                    outer_response |= response;
                });
                ui.end_row();
            }
        });
        outer_response
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
struct State {
    is_opened: bool,
}
