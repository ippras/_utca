use egui::{Grid, InnerResponse, Label, Response, Ui, Widget};
use egui_l20n::UiExt as _;
use lipid::prelude::*;
use polars::prelude::*;
use std::{
    borrow::Cow,
    fmt::{Write, from_fn},
};

/// Label widget
pub(crate) struct LabelWidget<'a> {
    label: &'a StringChunked,
    fatty_acid: &'a FattyAcidChunked,
    row: usize,
    editable: bool,
    hover_names: bool,
}

impl<'a> LabelWidget<'a> {
    pub(crate) fn new(
        label: &'a StringChunked,
        fatty_acid: &'a FattyAcidChunked,
        row: usize,
    ) -> Self {
        Self {
            label,
            fatty_acid,
            row,
            editable: false,
            hover_names: false,
        }
    }

    pub(crate) fn editable(self, editable: bool) -> Self {
        Self { editable, ..self }
    }

    pub(crate) fn hover_names(self, hover_names: bool) -> Self {
        Self {
            hover_names,
            ..self
        }
    }
}

impl LabelWidget<'_> {
    pub(crate) fn show(self, ui: &mut Ui) -> InnerResponse<PolarsResult<Option<Inner>>> {
        let mut inner = Ok(None);
        let Some(text) = self.label.get(self.row) else {
            let mut response = ui.response();
            response.mark_changed();
            return InnerResponse::new(Ok(Some(Inner::Cell(String::new()))), response);
        };
        let fatty_acid = match self.fatty_acid.get(self.row) {
            Ok(fatty_acid) => fatty_acid,
            Err(error) => return InnerResponse::new(Err(error), ui.response()),
        };
        let mut response = if self.editable {
            let mut text = text.to_owned();
            let mut response = ui.text_edit_singleline(&mut text);
            if response.changed() {
                inner = Ok(Some(Inner::Cell(text)));
            }
            let mut changed = false;
            response.context_menu(|ui| {
                if let Some(fatty_acid) = &fatty_acid {
                    ui.menu_button("Fill one label", |ui| {
                        let id = fatty_acid.id();
                        if ui.button("Common name").clicked()
                            && let Some(name) = ui.try_localize(&format!("{id}.common"))
                        {
                            let label = name.to_owned();
                            inner = Ok(Some(Inner::Cell(label)));
                            changed = true;
                        }
                        if ui.button("Empty string").clicked() {
                            inner = Ok(Some(Inner::Cell(String::new())));
                            changed = true;
                        }
                    });
                }
                ui.menu_button("Fill all labels", |ui| -> PolarsResult<()> {
                    if ui.button("Common name").clicked() {
                        // self.fatty_acid
                        //     .fields()?
                        //     .iter()
                        //     .map(|fatty_acid| {
                        //         let Some(fatty_acid) = fatty_acid? else {
                        //             return Ok(None);
                        //         };
                        //         from_fn(|f| {
                        //             // f.write_char('(');
                        //             for unsaturated in fatty_acid.unsaturated {
                        //                 f.write_fmt("{}{}", unsaturated.index, unsaturated.parity);
                        //             }
                        //             f.write_char('-');
                        //             match fatty_acid.carbon % 10 {
                        //                 1 => f.write_str("un")?,
                        //                 2 => f.write_str("eth")?,
                        //                 3 => f.write_str("prop")?,
                        //                 4 => f.write_str("but")?,
                        //                 5 => f.write_str("pent")?,
                        //                 6 => f.write_str("hex")?,
                        //                 7 => f.write_str("hept")?,
                        //                 8 => f.write_str("oct")?,
                        //             };
                        //             match fatty_acid.carbon / 10 {
                        //                 0 => {}
                        //                 1 => f.write_str("dec")?,
                        //             }
                        //             Ok(())
                        //         });
                        //         Ok(Some(fatty_acid.id().to_string()))
                        //     })
                        //     .collect();
                        let label = self.fatty_acid.id()?.apply(|id| {
                            let id = id?;
                            let name = ui
                                .try_localize(&format!("{id}.common"))
                                .or_else(|| ui.try_localize(&format!("{id}.systematic")))
                                .unwrap_or_default();
                            Some(Cow::Owned(name))
                        });
                        inner = Ok(Some(Inner::Column(label)));
                        changed = true;
                    }
                    if ui.button("Empty string").clicked() {
                        let label = StringChunked::full(PlSmallStr::EMPTY, "", 1);
                        inner = Ok(Some(Inner::Column(label)));
                        changed = true;
                    }
                    Ok(())
                });
            });
            if changed {
                response.mark_changed();
            }
            response
        } else {
            Label::new(text).truncate().ui(ui)
        };
        if self.hover_names {
            response = response.on_hover_ui(|ui| {
                ui.add(NamesWidget::new(fatty_acid.as_ref()));
            });
        }
        InnerResponse::new(inner, response)
    }
}

impl Widget for LabelWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).response
    }
}

/// Inner
pub enum Inner {
    Cell(String),
    Column(StringChunked),
}

/// Names widget
pub(crate) struct NamesWidget<'a> {
    fatty_acid: Option<&'a FattyAcid>,
}

impl<'a> NamesWidget<'a> {
    pub(crate) fn new(fatty_acid: Option<&'a FattyAcid>) -> Self {
        Self { fatty_acid }
    }
}

impl Widget for NamesWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let Some(fatty_acid) = self.fatty_acid else {
            return ui.response();
        };
        ui.heading(ui.localize("names"));
        Grid::new(ui.next_auto_id())
            .show(ui, |ui| {
                let id = fatty_acid.id();
                if let Some(abbreviation) = ui.try_localize(&format!("{id}.abbreviation")) {
                    ui.label(ui.localize("abbreviation"));
                    ui.label(abbreviation);
                    ui.end_row();
                }
                if let Some(common) = ui.try_localize(&format!("{id}.common")) {
                    ui.label(ui.localize("common"));
                    if let Some(synonyms) = ui.try_localize(&format!("{id}.synonyms")) {
                        ui.label(format!("{common}; {synonyms}"));
                    } else {
                        ui.label(common);
                    }
                    ui.end_row();
                }
                if let Some(systematic) = ui.try_localize(&format!("{id}.systematic")) {
                    ui.label(ui.localize("systematic"));
                    ui.label(systematic);
                    ui.end_row();
                }
            })
            .response
    }
}
