use super::Selection;
use crate::{
    app::text::Text,
    special::composition::{
        Composition, MNC, MSC, NNC, NSC, SNC, SPC, SSC, TNC, TPC, TSC, UNC, USC,
    },
};
use ahash::RandomState;
use egui::{Response, Sense, Slider, SliderClamping, TextStyle, Ui, Widget, emath::Float as _};
use egui_ext::LabeledSeparator as _;
use egui_extras::{Column, TableBuilder};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{FUNNEL, FUNNEL_X, HASH};
use lipid::triacylglycerol::Stereospecificity;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    ops::BitXor,
};

/// Filter
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct Filter {
    pub key: HashSet<AnyValue<'static>>,
    pub value: f64,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            key: HashSet::new(),
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
        ui.menu_button(title, |ui| -> PolarsResult<()> {
            ui.heading(format!(
                "{} {}",
                ui.localize(self.selection.composition.text()),
                ui.localize("settings-filter?case=lower"),
            ));
            // let column = match self.selection.composition {
            //     MNC | MSC => &self.data_frame["Mass"],
            //     NNC | NSC => &self.data_frame["EquivalentCarbonNumber"],
            //     SNC | SPC | SSC => &self.data_frame["Species"],
            //     TNC | TPC | TSC => &self.data_frame["Type"],
            //     UNC | USC => &self.data_frame["Unsaturation"],
            // };
            let series = match self.selection.composition {
                MNC => self.series,
                // MSC => self.series.tag(),
                // MSC => &self.data_frame["Mass"],
                // NNC | NSC => &self.data_frame["EquivalentCarbonNumber"],
                // SNC | SPC | SSC => &self.data_frame["Species"],
                // TNC | TPC | TSC => &self.data_frame["Type"],
                // UNC | USC => &self.data_frame["Unsaturation"],
                _ => self.series,
            };
            // let series = &series.unique()?.sort(Default::default())?;
            // Key
            ui.labeled_separator("Key");
            let max_scroll_height = ui.spacing().combo_height;
            let height = TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);
            TableBuilder::new(ui)
                .column(Column::auto().auto_size_this_frame(true))
                .columns(Column::remainder(), 3)
                .max_scroll_height(max_scroll_height)
                .vscroll(true)
                .header(height, |mut header| {
                    header.col(|ui| {
                        ui.heading(HASH);
                    });
                    header.col(|ui| {
                        ui.heading("SN-1");
                    });
                    header.col(|ui| {
                        ui.heading("SN-2");
                    });
                    header.col(|ui| {
                        ui.heading("SN-3");
                    });
                })
                .body(|body| {
                    let try_series = |number| -> PolarsResult<Series> {
                        let series = if let Some(r#struct) = self.series.try_struct() {
                            &r#struct.fields_as_series()[0] // TODO
                        } else {
                            self.series
                        };
                        Ok(series.unique()?.sort(Default::default())?)
                    };
                    let series = try_series();
                    body.rows(height, series.len(), |mut row| {
                        let index = row.index();
                        row.col(|ui| {
                            ui.label(index.to_string());
                        });
                        match self.selection.composition {
                            MNC | NNC | UNC => {
                                row.col(|ui| {
                                    ui.add(StereospecificNumberWidget {
                                        number: None,
                                        selection: self.selection,
                                        index,
                                        series,
                                    });
                                });
                            }
                            _ => {
                                row.col(|ui| {
                                    ui.add(StereospecificNumberWidget {
                                        number: Some(0),
                                        selection: self.selection,
                                        index,
                                        series,
                                    });
                                });
                                row.col(|ui| {
                                    ui.add(StereospecificNumberWidget {
                                        number: Some(1),
                                        selection: self.selection,
                                        index,
                                        series,
                                    });
                                });
                                row.col(|ui| {
                                    ui.add(StereospecificNumberWidget {
                                        number: Some(2),
                                        selection: self.selection,
                                        index,
                                        series,
                                    });
                                });
                            }
                        }
                    });
                });
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
        .response
    }
}

struct StereospecificNumberWidget<'a> {
    number: Option<usize>,
    selection: &'a mut Selection,
    index: usize,
    series: &'a Series,
}

impl StereospecificNumberWidget<'_> {
    fn show(self, ui: &mut Ui) -> PolarsResult<Response> {
        let value = self.series.get(self.index)?;
        let text = value.str_value();
        let contains = self.selection.filter.key.contains(&value);
        let mut selected = contains;
        let response = ui.toggle_value(&mut selected, text);
        if selected && !contains {
            self.selection.filter.key.insert(value.into_static());
        } else if !selected && contains {
            self.selection.filter.key.remove(&value.into_static());
        }
        response.context_menu(|ui| {
            if ui.button(format!("{FUNNEL} Select all")).clicked() {
                for key in self.series.iter() {
                    self.selection
                        .filter
                        .key
                        .entry(key.into_static())
                        .or_insert();
                }
                ui.close_menu();
            }
            if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                self.selection.filter.key.clear();
                ui.close_menu();
            }
        });
        Ok(response)
        // let value = if let Some(number) = self.number {
        //     match value {
        //         AnyValue::Array(series, length) => {
        //             let t = series
        //                 .array()?
        //                 .get_as_series(number)
        //                 .unwrap()
        //                 .get(self.index);
        //         }
        //         _ => unreachable!(),
        //     }
        //     // let value = value
        //     //     ._iter_struct_av()
        //     //     .nth(number)
        //     //     .unwrap_or_default()
        //     //     .into_static();
        //     // let text = value.str_value();
        //     // let response = ui.toggle_value(&mut value, text);
        // } else {
        //     self.series.get(self.index)?
        // };
    }
}

impl<'a> Widget for StereospecificNumberWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        self.show(ui).unwrap()
        // let mut value = self.series.get(self.index).unwrap_or_default();
        // self.selection.filter.key.entry(key.into_static()).or_insert();
        // let value = if let Some(number) = self.number {
        //     self.series
        //         .get(self.index)
        //         .unwrap_or_default()
        //         ._iter_struct_av()
        //         .nth(number)
        //         .clone()
        //         .unwrap_or_default()
        // } else {
        //     self.series.get(self.index).unwrap_or_default()
        // };
        // let text = value.str_value();
        // match self.selection.composition.stereospecificity {
        //     Some(stereospecificity) => todo!(),
        //     None => todo!(),
        // }

        // let response = ui.toggle_value(&mut value[self.number], text);
        // response.context_menu(|ui| {
        //     if ui.button(format!("{FUNNEL} Select all")).clicked() {
        //         for key in self.series.iter() {
        //             let av_values: Vec<_> = key._iter_struct_av().collect();
        //             let value = self.filter.key.entry(key.into_static()).or_default();
        //             value[self.number] = true;
        //         }
        //         ui.close_menu();
        //     }
        //     if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
        //         for value in self.filter.key.values_mut() {
        //             value[self.number] = false;
        //         }
        //         ui.close_menu();
        //     }
        // });
        // response
    }
}

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
