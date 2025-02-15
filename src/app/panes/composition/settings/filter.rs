use super::Selection;
use crate::{
    app::text::Text,
    special::composition::{
        Composition, MMC, MSC, NMC, NSC, SMC, SPC, SSC, TMC, TPC, TSC, UMC, USC,
    },
};
use ahash::RandomState;
use egui::{
    CentralPanel, DragValue, Grid, Response, ScrollArea, Sense, Slider, SliderClamping, TextStyle,
    TopBottomPanel, Ui, Widget, emath::Float as _, style::Widgets,
};
use egui_ext::LabeledSeparator as _;
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{FUNNEL, FUNNEL_X, HASH, MINUS, PLUS};
use itertools::Itertools;
use polars::prelude::*;
use re_ui::UiExt as _;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashMap,
    hash::{Hash, Hasher},
    ops::{BitXor, Range},
};
use tracing::error;

const DEFAULT: [bool; 3] = [false; 3];

// /// Acylglycerol
// #[derive(Clone, Copy, Debug)]
// pub enum Acylglycerol<T> {
//     Mono(Mag<T>),
//     Di(Dag<T>),
//     Tri(Tag<T>),
// }
// enum Value {
//     Mono(bool),
//     Di([bool; 2]),
//     Tri([bool; 3]),
// }

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Filter {
    pub key: HashMap<AnyValue<'static>, [bool; 3]>,
    pub value: f64,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            key: HashMap::new(),
            value: 0.0,
        }
    }
}

impl Eq for Filter {}

impl Hash for Filter {
    fn hash<H: Hasher>(&self, state: &mut H) {
        state.write_usize(self.key.len());
        let hash = self
            .key
            .iter()
            .map(|value| RandomState::with_seeds(1, 2, 3, 4).hash_one(value))
            .fold(0, BitXor::bitxor);
        state.write_u64(hash);
        self.value.ord().hash(state);
    }
}

impl PartialEq for Filter {
    fn eq(&self, other: &Self) -> bool {
        self.key == other.key && self.value.ord() == other.value.ord()
    }
}

/// Filter widget
pub struct FilterWidget<'a> {
    selection: &'a mut Selection,
    series: &'a Series,
    percent: bool,
}

impl<'a> FilterWidget<'a> {
    pub fn new(selection: &'a mut Selection, series: &'a Series) -> Self {
        Self {
            selection,
            series,
            percent: false,
        }
    }

    pub fn percent(mut self, percent: bool) -> Self {
        self.percent = percent;
        self
    }
}

impl Widget for FilterWidget<'_> {
    fn ui(self, ui: &mut Ui) -> Response {
        let title = if self.selection.filter == Default::default() {
            FUNNEL_X
        } else {
            ui.visuals_mut().widgets.inactive = ui.visuals().widgets.active;
            FUNNEL
        };
        let response = ui
            .menu_button(title, |ui| -> PolarsResult<()> {
                ui.heading(format!(
                    "{} {}",
                    ui.localize(&format!(
                        "{}.abbreviation",
                        self.selection.composition.text(),
                    )),
                    ui.localize("settings-filter?case=lower"),
                ));
                // Key
                ui.labeled_separator("Key");
                match self.selection.composition {
                    MMC | NMC | UMC => {
                        let series = self.series.unique()?.sort(Default::default())?;
                        ui.add(ColumnWidget {
                            indices: vec![0, 1, 2],
                            selection: self.selection,
                            series,
                        });
                    }
                    SMC | TMC => {
                        let fields = self.series.struct_()?.fields_as_series();
                        let series = fields[0].unique()?.sort(Default::default())?;
                        ui.add(ColumnWidget {
                            indices: vec![0, 1, 2],
                            selection: self.selection,
                            series,
                        });
                    }
                    SPC | TPC => {
                        let fields = self.series.struct_()?.fields_as_series();
                        ui.columns_const(|ui: &mut [Ui; 2]| -> PolarsResult<()> {
                            let series = fields[0].unique()?.sort(Default::default())?;
                            ui[0].add(ColumnWidget {
                                indices: vec![0, 2],
                                selection: self.selection,
                                series,
                            });
                            let series = fields[1].unique()?.sort(Default::default())?;
                            ui[1].add(ColumnWidget {
                                indices: vec![1],
                                selection: self.selection,
                                series,
                            });
                            Ok(())
                        })?;
                    }
                    MSC | NSC | SSC | TSC | USC => {
                        let fields = self.series.struct_()?.fields_as_series();
                        ui.columns_const(|ui: &mut [Ui; 3]| -> PolarsResult<()> {
                            for index in 0..3 {
                                let series = fields[index].unique()?.sort(Default::default())?;
                                ui[index].add(ColumnWidget {
                                    indices: vec![index],
                                    selection: self.selection,
                                    series,
                                });
                            }
                            Ok(())
                        })?;
                    }
                }
                // Value
                ui.separator();
                ui.horizontal(|ui| {
                    ui.label("Value");
                    ui.add(
                        Slider::new(&mut self.selection.filter.value, 0.0..=1.0)
                            .clamping(SliderClamping::Always)
                            .logarithmic(true)
                            .custom_formatter(|mut value, _| {
                                if self.percent {
                                    value *= 100.0;
                                }
                                AnyValue::Float64(value).to_string()
                            })
                            .custom_parser(|value| {
                                let mut parsed = value.parse::<f64>().ok()?;
                                if self.percent {
                                    parsed /= 100.0;
                                }
                                Some(parsed)
                            }),
                    );
                });
                Ok(())
            })
            .response;
        // ui.menu_button(title, |ui| -> PolarsResult<()> {
        //     ui.heading(format!(
        //         "{} {}",
        //         ui.localize(&format!(
        //             "{}.abbreviation",
        //             self.selection.composition.text(),
        //         )),
        //         ui.localize("settings-filter?case=lower"),
        //     ));
        //     // Key
        //     Grid::new(ui.next_auto_id()).show(ui, |ui| -> PolarsResult<()> {
        //         let widgets = if ui.visuals().dark_mode {
        //             Widgets::dark()
        //         } else {
        //             Widgets::light()
        //         };
        //         ui.visuals_mut().widgets.inactive.weak_bg_fill = widgets.hovered.weak_bg_fill;
        //         ui.visuals_mut().widgets.hovered.bg_stroke = widgets.hovered.bg_stroke;
        //         ui.label("Key");
        //         if ui.button(PLUS).clicked() {
        //             self.selection.filter.temp.push(0.0..0.0);
        //         }
        //         ui.end_row();
        //         match self.selection.composition {
        //             MMC => {
        //                 self.selection.filter.temp.retain_mut(|range| {
        //                     ui.label("");
        //                     ui.horizontal(|ui| {
        //                         let mut keep = true;
        //                         if ui.button(MINUS).clicked() {
        //                             keep = false;
        //                         }
        //                         ui.add(
        //                             DragValue::new(&mut range.start)
        //                                 .range(0.0..=range.end)
        //                                 .clamp_existing_to_range(true),
        //                         );
        //                         ui.add(
        //                             DragValue::new(&mut range.end)
        //                                 .range(range.start..=f64::MAX)
        //                                 .clamp_existing_to_range(true),
        //                         );
        //                         ui.end_row();
        //                         keep
        //                     })
        //                     .inner
        //                 });
        //             }
        //             NMC | UMC => {}
        //             SMC | TMC => {
        //                 let fields = self.series.struct_()?.fields_as_series();
        //                 let series = fields[0].unique()?.sort(Default::default())?;
        //                 ui.add(ColumnWidget {
        //                     indices: vec![0, 1, 2],
        //                     selection: self.selection,
        //                     series,
        //                 });
        //             }
        //             SPC | TPC => {}
        //             MSC | NSC | SSC | TSC | USC => {}
        //         }
        //         Ok(())
        //     });
        //     // Value
        //     ui.separator();
        //     Ok(())
        // });
        response
    }
}

struct ColumnWidget<'a> {
    indices: Vec<usize>,
    selection: &'a mut Selection,
    series: Series,
}

impl<'a> Widget for ColumnWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let stereospecific_numbers = self.indices.iter().map(|index| index + 1).format(",");
        ui.heading(format!("sn-{stereospecific_numbers}"));
        ui.separator();
        let max_scroll_height = ui.spacing().combo_height;
        let height = TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);
        if let Err(error) = ScrollArea::vertical()
            .id_salt(ui.next_auto_id())
            .max_height(max_scroll_height)
            .show_rows(
                ui,
                height,
                self.series.len(),
                |ui, range| -> PolarsResult<()> {
                    for index in range {
                        let key = self.series.get(index)?.into_static();
                        let text = key.str_value();
                        let value = self.selection.filter.key.entry(key.clone()).or_default();
                        let first = self.indices[0];
                        let response = ui.toggle_value(&mut value[first], text);
                        for &index in &self.indices[1..] {
                            value[index] = value[first];
                        }
                        if *value == DEFAULT {
                            self.selection.filter.key.remove(&key);
                        }
                        response.context_menu(|ui| {
                            if ui.button(format!("{FUNNEL} Select all")).clicked() {
                                for key in self.series.iter() {
                                    let value = self
                                        .selection
                                        .filter
                                        .key
                                        .entry(key.into_static())
                                        .or_default();
                                    for &index in &self.indices {
                                        value[index] = true;
                                    }
                                }
                                ui.close_menu();
                            }
                            if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                                self.selection.filter.key.retain(|_, value| {
                                    for &index in &self.indices {
                                        value[index] = false;
                                    }
                                    *value != DEFAULT
                                });
                                ui.close_menu();
                            }
                        });
                    }
                    Ok(())
                },
            )
            .inner
        {
            error!(%error);
            ui.error_with_details_on_hover(error.to_string());
        }
        ui.allocate_response(Default::default(), Sense::hover())
    }
}

// struct ColumnWidget<'a> {
//     header: &'a str,
//     selection: &'a mut Selection,
//     series: Series,
// }

// impl<'a> Widget for ColumnWidget<'a> {
//     fn ui(self, ui: &mut Ui) -> Response {
//         let max_scroll_height = ui.spacing().combo_height;
//         let height = TextStyle::Body
//             .resolve(ui.style())
//             .size
//             .max(ui.spacing().interact_size.y);
//         let id_salt = ui.next_auto_id();
//         TableBuilder::new(ui)
//             .id_salt(id_salt)
//             .columns(Column::remainder(), 2)
//             .max_scroll_height(max_scroll_height)
//             .vscroll(true)
//             .header(height, |mut header| {
//                 header.col(|ui| {
//                     ui.heading(HASH);
//                 });
//                 header.col(|ui| {
//                     ui.heading(self.header);
//                 });
//             })
//             .body(|body| {
//                 body.rows(height, self.series.len(), |mut row| {
//                     let index = row.index();
//                     row.col(|ui| {
//                         ui.label(index.to_string());
//                     });
//                     row.col(|ui| match self.series.get(index) {
//                         Ok(value) => {
//                             let text = value.str_value();
//                             let contains = self.selection.filter.key.contains(&value);
//                             let mut selected = contains;
//                             let response = ui.toggle_value(&mut selected, text);
//                             if selected && !contains {
//                                 self.selection.filter.key.insert(value.into_static());
//                             } else if !selected && contains {
//                                 self.selection.filter.key.remove(&value.into_static());
//                             }
//                             response.context_menu(|ui| {
//                                 if ui.button(format!("{FUNNEL} Select all")).clicked() {
//                                     for key in self.series.iter() {
//                                         self.selection
//                                             .filter
//                                             .key
//                                             .entry(key.into_static())
//                                             .or_insert();
//                                     }
//                                     ui.close_menu();
//                                 }
//                                 if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
//                                     self.selection.filter.key.clear();
//                                     ui.close_menu();
//                                 }
//                             });
//                         }
//                         Err(error) => {
//                             error!(%error);
//                             ui.error_with_details_on_hover(error.to_string());
//                         }
//                     });
//                 });
//             });
//         ui.allocate_response(Default::default(), Sense::hover())
//     }
// }

// /// Extension methods for [`Response`]
// trait ResponseExt {
//     fn or_union(&mut self, other: Response);

//     fn unwrap_and_union(self, other: Response) -> Response;
// }

// impl ResponseExt for Option<Response> {
//     fn or_union(&mut self, other: Response) {
//         *self = Some(self.take().unwrap_and_union(other));
//     }

//     fn unwrap_and_union(self, other: Response) -> Response {
//         match self {
//             Some(outer_response) => outer_response | other,
//             None => other,
//         }
//     }
// }
