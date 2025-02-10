use crate::{app::ResultExt, localize, try_localize};
use egui::{
    Align, Button, DragValue, Grid, Id, InnerResponse, Layout, Response, RichText, ScrollArea,
    Sense, Ui, Vec2, Widget, style::Widgets, vec2,
};
use egui_phosphor::regular::{ASTERISK, EQUALS, LIST};
use lipid::fatty_acid::{
    FattyAcid, FattyAcidExt as _, Isomerism, Unsaturated, Unsaturation,
    display::{COMMON, DisplayWithOptions, ID},
};
use polars::prelude::*;
use std::{cmp::Ordering, hash::Hash, mem::replace};

/// Fatty acid widget
pub(crate) struct FattyAcidWidget<'a> {
    pub(crate) value: Box<dyn Fn() -> PolarsResult<Option<FattyAcid>> + 'a>,
    pub(crate) id_salt: Id,
    pub(crate) editable: bool,
    pub(crate) hover: bool,
    pub(crate) names: bool,
}

impl<'a> FattyAcidWidget<'a> {
    pub(crate) fn new(value: impl Fn() -> PolarsResult<Option<FattyAcid>> + 'a) -> Self {
        Self {
            value: Box::new(value),
            id_salt: Id::new("FattyAcid"),
            editable: false,
            hover: false,
            names: false,
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

    pub(crate) fn names(self, names: bool) -> Self {
        Self { names, ..self }
    }
}

impl FattyAcidWidget<'_> {
    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<Option<FattyAcid>> {
        let mut inner = None;
        let mut changed = false;
        // Error
        let value = match (self.value)() {
            Ok(value) => value,
            Err(error) => {
                let response = ui.label("Error").on_hover_text(error.to_string());
                return InnerResponse::new(inner, response);
            }
        };
        // None
        let Some(fatty_acid) = value else {
            let mut response = ui.label("None");
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
            return InnerResponse::new(inner, response);
        };
        // Some
        let text = &format!("{:#}", (&fatty_acid).display(COMMON));
        let mut response = ui.add_sized(
            [ui.available_width(), ui.spacing().interact_size.y],
            |ui: &mut Ui| {
                ui.menu_button(text, |ui| {
                    let inner_response = Self::fatty_acid_contents(&self, &fatty_acid)(ui);
                    if inner_response.response.changed() {
                        inner = inner_response.inner;
                        changed = true;
                        // println!("??? inner_response: {} {:?}", r.response.changed(), r.inner);
                    }
                })
                .response
                // let inner_response = ui.menu_button(text, |ui| {
                //     let r = Self::contents(&self, &fatty_acid)(ui);
                //     if r.response.changed() {
                //         println!("??? inner_response: {} {:?}", r.response.changed(), r.inner);
                //     }
                //     r
                // });
                // if let Some(inner_response) = inner_response.inner {
                //     println!("inner_response: {inner_response:?}");
                //     inner = inner_response.inner;
                //     changed = true;
                //     return inner_response.response;
                // }
                // inner_response.response

                // ui.menu_button(text, |ui| {
                //     Self::contents(&self, &fatty_acid)(ui)
                //     // let widgets = if ui.visuals().dark_mode {
                //     //     Widgets::dark()
                //     // } else {
                //     //     Widgets::light()
                //     // };
                //     // ui.visuals_mut().widgets.inactive.weak_bg_fill = widgets.active.weak_bg_fill;
                //     // Grid::new(ui.auto_id_with(self.id_salt)).show(ui, |ui| {
                //     //     // Carbons
                //     //     ui.label("Carbons");
                //     //     if ui.add(DragValue::new(&mut fatty_acid.carbons)).changed() {
                //     //         inner = Some(fatty_acid.clone());
                //     //         changed = true;
                //     //     }
                //     //     ui.end_row();
                //     //     // Unsaturated
                //     //     ui.collapsing("Unsaturated", |ui| {
                //     //         let bounds = fatty_acid.bounds();
                //     //         let mut unsaturated_changed = false;
                //     //         for unsaturated in &mut fatty_acid.unsaturated {
                //     //             Grid::new(ui.next_auto_id()).show(ui, |ui| {
                //     //                 // Index
                //     //                 unsaturated_changed |=
                //     //                     DragValue::new(unsaturated.index.get_or_insert_default())
                //     //                         .range(0..=bounds)
                //     //                         .custom_formatter(|value, _| {
                //     //                             if value != 0.0 {
                //     //                                 value.to_string()
                //     //                             } else {
                //     //                                 "*".to_owned()
                //     //                             }
                //     //                         })
                //     //                         .clamp_existing_to_range(true)
                //     //                         .update_while_editing(false)
                //     //                         .ui(ui)
                //     //                         .changed();
                //     //                 // Isomerism
                //     //                 let text = match &unsaturated.isomerism {
                //     //                     Some(Isomerism::Cis) => "C",
                //     //                     Some(Isomerism::Trans) => "T",
                //     //                     None => "*",
                //     //                 };
                //     //                 if ui.button(text).clicked() {
                //     //                     unsaturated.isomerism = match unsaturated.isomerism {
                //     //                         None => Some(Isomerism::Cis),
                //     //                         Some(Isomerism::Cis) => Some(Isomerism::Trans),
                //     //                         Some(Isomerism::Trans) => None,
                //     //                     };
                //     //                     unsaturated_changed = true;
                //     //                 }
                //     //                 // Unsaturation
                //     //                 let text = match &unsaturated.unsaturation {
                //     //                     Some(Unsaturation::One) => "1",
                //     //                     Some(Unsaturation::Two) => "2",
                //     //                     None => "*",
                //     //                 };
                //     //                 if ui.button(text).clicked() {
                //     //                     unsaturated.unsaturation = match unsaturated.unsaturation {
                //     //                         None => Some(Unsaturation::One),
                //     //                         Some(Unsaturation::One) => Some(Unsaturation::Two),
                //     //                         Some(Unsaturation::Two) => None,
                //     //                     };
                //     //                     unsaturated_changed = true;
                //     //                 }
                //     //             });
                //     //             ui.end_row();
                //     //         }
                //     //         if unsaturated_changed {
                //     //             fatty_acid.unsaturated.sort_by_cached_key(|unsaturated| {
                //     //                 (
                //     //                     unsaturated.index,
                //     //                     unsaturated.isomerism,
                //     //                     unsaturated.unsaturation,
                //     //                 )
                //     //             });
                //     //             inner = Some(fatty_acid.clone());
                //     //         }
                //     //         changed |= unsaturated_changed;
                //     //     });
                //     //     let mut unsaturated = fatty_acid.unsaturated.len();
                //     //     ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                //     //         if ui
                //     //             .add(
                //     //                 DragValue::new(&mut unsaturated)
                //     //                     .range(0..=fatty_acid.carbons)
                //     //                     .clamp_existing_to_range(true),
                //     //             )
                //     //             .changed()
                //     //         {
                //     //             loop {
                //     //                 match unsaturated.cmp(&fatty_acid.unsaturated.len()) {
                //     //                     Ordering::Less => {
                //     //                         fatty_acid.unsaturated.pop();
                //     //                     }
                //     //                     Ordering::Equal => break,
                //     //                     Ordering::Greater => {
                //     //                         fatty_acid.unsaturated.push(Unsaturated {
                //     //                             index: Some(0),
                //     //                             isomerism: Some(Isomerism::Cis),
                //     //                             unsaturation: Some(Unsaturation::One),
                //     //                         });
                //     //                     }
                //     //                 }
                //     //             }
                //     //             changed = true;
                //     //             inner = Some(fatty_acid.clone());
                //     //         }
                //     //     });
                //     // });
                // })
                // .response
            },
        );
        if self.hover {
            response = response.on_hover_text(text);
            if self.names {
                response = response.on_hover_ui(|ui| {
                    ui.heading(localize!("names"));
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        let id = fatty_acid.display(ID);
                        if let Some(abbreviation) = try_localize!(&format!("{id:#}.abbreviation")) {
                            ui.label(localize!("abbreviation"));
                            ui.label(abbreviation);
                            ui.end_row();
                        }

                        if let Some(common_name) = try_localize!(&format!("{id:#}.common_name")) {
                            ui.label(localize!("common_name"));
                            ui.label(common_name);
                            ui.end_row();
                        }

                        if let Some(systematic_name) =
                            try_localize!(&format!("{id:#}.systematic_name"))
                        {
                            ui.label(localize!("systematic_name"));
                            ui.label(systematic_name);
                            ui.end_row();
                        }
                    });
                });
            }
        }
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
        InnerResponse::new(inner, response)
    }

    fn fatty_acid_contents(
        &self,
        fatty_acid: &FattyAcid,
    ) -> impl Fn(&mut Ui) -> InnerResponse<Option<FattyAcid>> {
        move |ui| {
            let widgets = if ui.visuals().dark_mode {
                Widgets::dark()
            } else {
                Widgets::light()
            };
            ui.visuals_mut().widgets.inactive.weak_bg_fill = widgets.active.weak_bg_fill;

            let mut fatty_acid = fatty_acid.clone();
            let mut inner_response = ui
                .horizontal(|ui| {
                    // Carbons
                    let response = ui
                        .add(DragValue::new(&mut fatty_acid.carbons))
                        .on_hover_text("Carbons");
                    let inner = response.changed().then(|| fatty_acid.clone());
                    let mut inner_response = InnerResponse::new(inner, response);
                    // Unsaturated
                    let mut unsaturated = fatty_acid.unsaturated.len();
                    let response = ui
                        .add(
                            DragValue::new(&mut unsaturated)
                                .range(0..=fatty_acid.carbons)
                                .clamp_existing_to_range(true),
                        )
                        .on_hover_text("Unsaturated");
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
                                        isomerism: Some(Isomerism::Cis),
                                        unsaturation: Some(Unsaturation::One),
                                    });
                                }
                            }
                        }
                        inner_response.inner = Some(fatty_acid.clone());
                        inner_response.response = response;
                    }
                    ui.menu_button(RichText::new("⏵"), |ui| {
                        //
                    });
                    inner_response
                })
                .inner;
            ui.separator();
            Grid::new(ui.auto_id_with(self.id_salt))
                .show(ui, |ui| {
                    let bounds = fatty_acid.bounds();
                    for unsaturated in &mut fatty_acid.unsaturated {
                        // Index
                        let response = ui.add(
                            DragValue::new(unsaturated.index.get_or_insert_default())
                                .range(0..=bounds)
                                .clamp_existing_to_range(true)
                                .update_while_editing(false),
                        );
                        if response.changed() {
                            inner_response.response.mark_changed();
                        }
                        ui.horizontal(|ui| {
                            // ui.spacing_mut().item_spacing.x = ui.spacing().item_spacing.y;
                            let x = ui.spacing().interact_size.x / 2.0;
                            // Unsaturation
                            let (text, hover_text) = match &unsaturated.unsaturation {
                                Some(Unsaturation::One) => (EQUALS, "Double bounds"),
                                Some(Unsaturation::Two) => (LIST, "Triple bounds"),
                                None => (ASTERISK, "Any number of bounds"),
                            };
                            let response = ui.button(text).on_hover_text(hover_text);
                            if response.clicked() {
                                unsaturated.unsaturation = match unsaturated.unsaturation {
                                    None => Some(Unsaturation::One),
                                    Some(Unsaturation::One) => Some(Unsaturation::Two),
                                    Some(Unsaturation::Two) => None,
                                };
                                inner_response.response.mark_changed();
                            }
                            // Isomerism
                            let (text, hover_text) = match &unsaturated.isomerism {
                                Some(Isomerism::Cis) => ("C", "Cis"),
                                Some(Isomerism::Trans) => ("T", "Trans"),
                                None => (ASTERISK, "Any isomerism"),
                            };
                            let response = ui
                                .add(Button::new(text).min_size(response.rect.size()))
                                .on_hover_text(hover_text);
                            if response.clicked() | response.middle_clicked() {
                                unsaturated.isomerism = match unsaturated.isomerism {
                                    None => Some(Isomerism::Cis),
                                    Some(Isomerism::Cis) => Some(Isomerism::Trans),
                                    Some(Isomerism::Trans) => None,
                                };
                                inner_response.response.mark_changed();
                            }
                        });
                        ui.end_row();
                    }

                    // Unsaturated
                    // ui.collapsing("Unsaturated", |ui| {
                    //     let bounds = fatty_acid.bounds();
                    //     let mut unsaturated_changed = false;
                    //     for unsaturated in &mut fatty_acid.unsaturated {
                    //         Grid::new(ui.next_auto_id()).show(ui, |ui| {
                    //             // Index
                    //             unsaturated_changed |=
                    //                 DragValue::new(unsaturated.index.get_or_insert_default())
                    //                     .range(0..=bounds)
                    //                     .custom_formatter(|value, _| {
                    //                         if value != 0.0 {
                    //                             value.to_string()
                    //                         } else {
                    //                             "*".to_owned()
                    //                         }
                    //                     })
                    //                     .clamp_existing_to_range(true)
                    //                     .update_while_editing(false)
                    //                     .ui(ui)
                    //                     .changed();
                    //             // Isomerism
                    //             let text = match &unsaturated.isomerism {
                    //                 Some(Isomerism::Cis) => "C",
                    //                 Some(Isomerism::Trans) => "T",
                    //                 None => "*",
                    //             };
                    //             if ui.button(text).clicked() {
                    //                 unsaturated.isomerism = match unsaturated.isomerism {
                    //                     None => Some(Isomerism::Cis),
                    //                     Some(Isomerism::Cis) => Some(Isomerism::Trans),
                    //                     Some(Isomerism::Trans) => None,
                    //                 };
                    //                 unsaturated_changed = true;
                    //             }
                    //             // Unsaturation
                    //             let text = match &unsaturated.unsaturation {
                    //                 Some(Unsaturation::One) => "1",
                    //                 Some(Unsaturation::Two) => "2",
                    //                 None => "*",
                    //             };
                    //             if ui.button(text).clicked() {
                    //                 unsaturated.unsaturation = match unsaturated.unsaturation {
                    //                     None => Some(Unsaturation::One),
                    //                     Some(Unsaturation::One) => Some(Unsaturation::Two),
                    //                     Some(Unsaturation::Two) => None,
                    //                 };
                    //                 unsaturated_changed = true;
                    //             }
                    //         });
                    //         ui.end_row();
                    //     }
                    //     if unsaturated_changed {
                    //         fatty_acid.unsaturated.sort_by_cached_key(|unsaturated| {
                    //             (
                    //                 unsaturated.index,
                    //                 unsaturated.isomerism,
                    //                 unsaturated.unsaturation,
                    //             )
                    //         });
                    //         *inner = Some(fatty_acid.clone());
                    //     }
                    //     changed |= unsaturated_changed;
                    // });

                    // let mut unsaturated = fatty_acid.unsaturated.len();
                    // ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                    //     if ui
                    //         .add(
                    //             DragValue::new(&mut unsaturated)
                    //                 .range(0..=fatty_acid.carbons)
                    //                 .clamp_existing_to_range(true),
                    //         )
                    //         .changed()
                    //     {
                    //         loop {
                    //             match unsaturated.cmp(&fatty_acid.unsaturated.len()) {
                    //                 Ordering::Less => {
                    //                     fatty_acid.unsaturated.pop();
                    //                 }
                    //                 Ordering::Equal => break,
                    //                 Ordering::Greater => {
                    //                     fatty_acid.unsaturated.push(Unsaturated {
                    //                         index: Some(0),
                    //                         isomerism: Some(Isomerism::Cis),
                    //                         unsaturation: Some(Unsaturation::One),
                    //                     });
                    //                 }
                    //             }
                    //         }
                    //         *inner = Some(fatty_acid.clone());
                    //     }
                    // });
                    if inner_response.response.changed() {
                        inner_response.inner = Some(fatty_acid.clone());
                    }
                    inner_response
                })
                .inner
        }
    }
}

impl FattyAcidWidget<'_> {
    pub(crate) fn try_show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<Option<FattyAcid>>> {
        let mut inner = (self.value)();
        let Ok(Some(mut fatty_acid)) = replace(&mut inner, Ok(None)) else {
            // Null
            let response = ui.label(AnyValue::Null.to_string());
            return InnerResponse::new(inner, response);
        };
        let text = &format!("{:#}", (&fatty_acid).display(COMMON));
        let mut response = if self.editable {
            // Writable
            ui.add_sized(
                vec2(ui.available_width(), ui.spacing().interact_size.y),
                |ui: &mut Ui| {
                    ui.menu_button(text, |ui| {
                        let widgets = if ui.visuals().dark_mode {
                            Widgets::dark()
                        } else {
                            Widgets::light()
                        };
                        ui.visuals_mut().widgets.inactive.weak_bg_fill =
                            widgets.active.weak_bg_fill;
                        // let mut fatty_acid = fatty_acid.clone();
                        Grid::new(ui.next_auto_id()).show(ui, |ui| {
                            // Carbons
                            ui.label("Carbons");
                            if ui.add(DragValue::new(&mut fatty_acid.carbons)).changed() {
                                inner = Ok(Some(fatty_acid.clone()));
                            }
                            ui.end_row();

                            // Unsaturated
                            ui.collapsing("Unsaturated", |ui| {
                                let bounds = fatty_acid.bounds();
                                let mut changed = false;
                                for unsaturated in &mut fatty_acid.unsaturated {
                                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                                        // Index
                                        changed |= DragValue::new(
                                            unsaturated.index.get_or_insert_default(),
                                        )
                                        .range(0..=bounds)
                                        .custom_formatter(|value, _| {
                                            if value != 0.0 {
                                                value.to_string()
                                            } else {
                                                "*".to_owned()
                                            }
                                        })
                                        .clamp_existing_to_range(true)
                                        .update_while_editing(false)
                                        .ui(ui)
                                        .changed();
                                        // Isomerism
                                        let text = match &unsaturated.isomerism {
                                            Some(Isomerism::Cis) => "C",
                                            Some(Isomerism::Trans) => "T",
                                            None => "*",
                                        };
                                        if ui.button(text).clicked() {
                                            unsaturated.isomerism = match unsaturated.isomerism {
                                                None => Some(Isomerism::Cis),
                                                Some(Isomerism::Cis) => Some(Isomerism::Trans),
                                                Some(Isomerism::Trans) => None,
                                            };
                                            changed = true;
                                        }
                                        // Unsaturation
                                        let text = match &unsaturated.unsaturation {
                                            Some(Unsaturation::One) => "1",
                                            Some(Unsaturation::Two) => "2",
                                            None => "*",
                                        };
                                        if ui.button(text).clicked() {
                                            unsaturated.unsaturation = match unsaturated
                                                .unsaturation
                                            {
                                                None => Some(Unsaturation::One),
                                                Some(Unsaturation::One) => Some(Unsaturation::Two),
                                                Some(Unsaturation::Two) => None,
                                            };
                                            changed = true;
                                        }
                                    });
                                    ui.end_row();
                                }
                                if changed {
                                    fatty_acid.unsaturated.sort_by_cached_key(|unsaturated| {
                                        (
                                            unsaturated.index,
                                            unsaturated.isomerism,
                                            unsaturated.unsaturation,
                                        )
                                    });
                                    inner = Ok(Some(fatty_acid.clone()));
                                }
                            });
                            let mut unsaturated = fatty_acid.unsaturated.len();
                            ui.with_layout(Layout::left_to_right(Align::TOP), |ui| {
                                if ui
                                    .add(
                                        DragValue::new(&mut unsaturated)
                                            .range(0..=fatty_acid.carbons)
                                            .clamp_existing_to_range(true),
                                    )
                                    .changed()
                                {
                                    loop {
                                        match unsaturated.cmp(&fatty_acid.unsaturated.len()) {
                                            Ordering::Less => {
                                                fatty_acid.unsaturated.pop();
                                            }
                                            Ordering::Equal => break,
                                            Ordering::Greater => {
                                                fatty_acid.unsaturated.push(Unsaturated {
                                                    index: Some(0),
                                                    isomerism: Some(Isomerism::Cis),
                                                    unsaturation: Some(Unsaturation::One),
                                                });
                                            }
                                        }
                                    }
                                    inner = Ok(Some(fatty_acid.clone()));
                                }
                            });
                        });
                    })
                    .response
                },
            )
        } else {
            // Readable
            ui.label(text)
        };
        // Hover
        if self.hover {
            response = response.on_hover_text(text);
            if self.names {
                response = response.on_hover_ui(|ui| {
                    ui.heading(localize!("names"));
                    Grid::new(ui.next_auto_id()).show(ui, |ui| {
                        let id = fatty_acid.display(ID);
                        if let Some(abbreviation) = try_localize!(&format!("{id:#}.abbreviation")) {
                            ui.label(localize!("abbreviation"));
                            ui.label(abbreviation);
                            ui.end_row();
                        }

                        if let Some(common_name) = try_localize!(&format!("{id:#}.common_name")) {
                            ui.label(localize!("common_name"));
                            ui.label(common_name);
                            ui.end_row();
                        }

                        if let Some(systematic_name) =
                            try_localize!(&format!("{id:#}.systematic_name"))
                        {
                            ui.label(localize!("systematic_name"));
                            ui.label(systematic_name);
                            ui.end_row();
                        }
                    });
                });
            }
        }
        InnerResponse::new(inner, response)
    }

    pub(crate) fn gshow(self, ui: &mut Ui) -> InnerResponse<Option<FattyAcid>> {
        let InnerResponse { inner, response } = self.try_show(ui);
        let inner = inner.context(ui.ctx()).flatten();
        InnerResponse::new(inner, response)
    }
}

impl Widget for FattyAcidWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

// fn fatty_acid_contents(
//     id_salt: &Id,
//     fatty_acid: &FattyAcid,
// ) -> impl Fn(&mut Ui) -> InnerResponse<Option<FattyAcid>> {
//     move |ui| {
//         let widgets = if ui.visuals().dark_mode {
//             Widgets::dark()
//         } else {
//             Widgets::light()
//         };
//         ui.visuals_mut().widgets.inactive.weak_bg_fill = widgets.active.weak_bg_fill;
//         let mut fatty_acid = fatty_acid.clone();
//         let mut inner_response = ui
//             .horizontal(|ui| {
//                 // Carbons
//                 let response = ui
//                     .add(DragValue::new(&mut fatty_acid.carbons))
//                     .on_hover_text("Carbons");
//                 let inner = response.changed().then(|| fatty_acid.clone());
//                 let mut inner_response = InnerResponse::new(inner, response);
//                 // Unsaturated
//                 let mut unsaturated = fatty_acid.unsaturated.len();
//                 let response = ui
//                     .add(
//                         DragValue::new(&mut unsaturated)
//                             .range(0..=fatty_acid.carbons)
//                             .clamp_existing_to_range(true),
//                     )
//                     .on_hover_text("Unsaturated");
//                 if response.changed() {
//                     loop {
//                         match unsaturated.cmp(&fatty_acid.unsaturated.len()) {
//                             Ordering::Less => {
//                                 fatty_acid.unsaturated.pop();
//                             }
//                             Ordering::Equal => break,
//                             Ordering::Greater => {
//                                 fatty_acid.unsaturated.push(Unsaturated {
//                                     index: Some(0),
//                                     isomerism: Some(Isomerism::Cis),
//                                     unsaturation: Some(Unsaturation::One),
//                                 });
//                             }
//                         }
//                     }
//                     inner_response.inner = Some(fatty_acid.clone());
//                     inner_response.response = response;
//                 }
//                 ui.menu_button(RichText::new("⏵"), |ui| {
//                     //
//                 });
//                 inner_response
//             })
//             .inner;
//         ui.separator();
//         // ui.add(UnsaturatedContent{id_salt, &fatty_acid}.show()).inner
//     }
// }

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
        ui.visuals_mut().widgets.inactive.weak_bg_fill = widgets.active.weak_bg_fill;

        let mut outer_response = ui.allocate_response(Default::default(), Sense::hover());
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
            ui.menu_button(RichText::new("⏵"), |ui| {
                let response = UnsaturatedContent::new(self.id_salt, &mut self.fatty_acid).show(ui);
                outer_response |= response;
            });
        });
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
                    // ui.spacing_mut().item_spacing.x = ui.spacing().item_spacing.y;
                    // let x = ui.spacing().interact_size.x / 2.0;
                    // Unsaturation
                    let (text, hover_text) = match &unsaturated.unsaturation {
                        Some(Unsaturation::One) => (EQUALS, "Double bounds"),
                        Some(Unsaturation::Two) => (LIST, "Triple bounds"),
                        None => (ASTERISK, "Any number of bounds"),
                    };
                    let response = ui.button(text).on_hover_text(hover_text);
                    if response.clicked() {
                        unsaturated.unsaturation = match unsaturated.unsaturation {
                            None => Some(Unsaturation::One),
                            Some(Unsaturation::One) => Some(Unsaturation::Two),
                            Some(Unsaturation::Two) => None,
                        };
                    }
                    let min_size = response.rect.size();
                    outer_response |= response;
                    // Isomerism
                    let (text, hover_text) = match &unsaturated.isomerism {
                        Some(Isomerism::Cis) => ("C", "Cis"),
                        Some(Isomerism::Trans) => ("T", "Trans"),
                        None => (ASTERISK, "Any isomerism"),
                    };
                    let response = ui
                        .add(Button::new(text).min_size(min_size))
                        .on_hover_text(hover_text);
                    if response.clicked() {
                        unsaturated.isomerism = match unsaturated.isomerism {
                            None => Some(Isomerism::Cis),
                            Some(Isomerism::Cis) => Some(Isomerism::Trans),
                            Some(Isomerism::Trans) => None,
                        };
                    }
                    outer_response |= response;
                });
                ui.end_row();
            }
        });
        outer_response
    }
}

// fn unsaturated_contents(
//     id_salt: &Id,
//     fatty_acid: &FattyAcid,
// ) -> impl Fn(&mut Ui) -> InnerResponse<Option<FattyAcid>> {
//     move |ui| {
//         Grid::new(ui.auto_id_with(id_salt))
//             .show(ui, |ui| {
//                 let bounds = fatty_acid.bounds();
//                 for unsaturated in &mut fatty_acid.unsaturated {
//                     // Index
//                     let response = ui.add(
//                         DragValue::new(unsaturated.index.get_or_insert_default())
//                             .range(0..=bounds)
//                             .clamp_existing_to_range(true)
//                             .update_while_editing(false),
//                     );
//                     if response.changed() {
//                         inner_response.response.mark_changed();
//                     }
//                     ui.horizontal(|ui| {
//                         // ui.spacing_mut().item_spacing.x = ui.spacing().item_spacing.y;
//                         let x = ui.spacing().interact_size.x / 2.0;
//                         // Unsaturation
//                         let (text, hover_text) = match &unsaturated.unsaturation {
//                             Some(Unsaturation::One) => (EQUALS, "Double bounds"),
//                             Some(Unsaturation::Two) => (LIST, "Triple bounds"),
//                             None => (ASTERISK, "Any number of bounds"),
//                         };
//                         let response = ui.button(text).on_hover_text(hover_text);
//                         if response.clicked() {
//                             unsaturated.unsaturation = match unsaturated.unsaturation {
//                                 None => Some(Unsaturation::One),
//                                 Some(Unsaturation::One) => Some(Unsaturation::Two),
//                                 Some(Unsaturation::Two) => None,
//                             };
//                             inner_response.response.mark_changed();
//                         }
//                         // Isomerism
//                         let (text, hover_text) = match &unsaturated.isomerism {
//                             Some(Isomerism::Cis) => ("C", "Cis"),
//                             Some(Isomerism::Trans) => ("T", "Trans"),
//                             None => (ASTERISK, "Any isomerism"),
//                         };
//                         let response = ui
//                             .add(Button::new(text).min_size(response.rect.size()))
//                             .on_hover_text(hover_text);
//                         if response.clicked() | response.middle_clicked() {
//                             unsaturated.isomerism = match unsaturated.isomerism {
//                                 None => Some(Isomerism::Cis),
//                                 Some(Isomerism::Cis) => Some(Isomerism::Trans),
//                                 Some(Isomerism::Trans) => None,
//                             };
//                             inner_response.response.mark_changed();
//                         }
//                     });
//                     ui.end_row();
//                 }
//                 if inner_response.response.changed() {
//                     inner_response.inner = Some(fatty_acid.clone());
//                 }
//                 inner_response
//             })
//             .inner
//     }
// }

// #[derive(Clone, Debug, Default, serde::Deserialize, serde::Serialize)]
// pub struct State {
//     // Maps columns ids to their widths.
//     pub col_widths: IdMap<f32>,

//     pub parent_width: Option<f32>,
// }
