use super::Selection;
use crate::{
    app::text::Text,
    special::composition::{
        Composition, MNC, MSC, NNC, NSC, SNC, SPC, SSC, TNC, TPC, TSC, UNC, USC,
    },
};
use ahash::RandomState;
use egui::{
    CentralPanel, Response, ScrollArea, Sense, Slider, SliderClamping, TextStyle, TopBottomPanel,
    Ui, Widget, emath::Float as _,
};
use egui_ext::LabeledSeparator as _;
use egui_extras::{Column, TableBuilder};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{FUNNEL, FUNNEL_X, HASH};
use lipid::triacylglycerol::Stereospecificity;
use polars::prelude::*;
use re_ui::UiExt as _;
use serde::{Deserialize, Serialize};
use std::{
    collections::HashSet,
    hash::{Hash, Hasher},
    ops::BitXor,
};
use tracing::error;

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
            // Key
            ui.labeled_separator("Key");
            let series = |index| -> PolarsResult<Series> {
                let series = if let Some(r#struct) = self.series.try_struct() {
                    &r#struct.fields_as_series()[index]
                } else {
                    self.series
                };
                Ok(series.unique()?.sort(Default::default())?)
            };
            ui.columns_const(|ui: &mut [Ui; 3]| -> PolarsResult<()> {
                ui[0].add(ColumnWidget1 {
                    header: "sn1",
                    selection: self.selection,
                    series: series(0)?,
                });
                ui[1].add(ColumnWidget1 {
                    header: "sn2",
                    selection: self.selection,
                    series: series(1)?,
                });
                ui[2].add(ColumnWidget1 {
                    header: "sn3",
                    selection: self.selection,
                    series: series(2)?,
                });
                Ok(())
            })?;
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

struct ColumnWidget1<'a> {
    header: &'a str,
    selection: &'a mut Selection,
    series: Series,
}

impl<'a> Widget for ColumnWidget1<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        ui.heading(self.header);
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
                        let value = self.series.get(index)?.into_static();
                        let text = value.str_value();
                        let contains = self.selection.filter.key.contains(&value);
                        println!("contains: {value} {contains}");
                        let mut selected = contains;
                        let response = ui.toggle_value(&mut selected, text);
                        if selected && !contains {
                            println!("selected value: {value}");
                            println!("self.selection.filter.key: {:?}", self.selection.filter.key);
                            self.selection.filter.key.insert(value);
                        } else if !selected && contains {
                            println!("!selected value: {value}");
                            self.selection.filter.key.remove(&value);
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

struct ColumnWidget<'a> {
    header: &'a str,
    selection: &'a mut Selection,
    series: Series,
}

impl<'a> Widget for ColumnWidget<'a> {
    fn ui(self, ui: &mut Ui) -> Response {
        let max_scroll_height = ui.spacing().combo_height;
        let height = TextStyle::Body
            .resolve(ui.style())
            .size
            .max(ui.spacing().interact_size.y);
        let id_salt = ui.next_auto_id();
        TableBuilder::new(ui)
            .id_salt(id_salt)
            .columns(Column::remainder(), 2)
            .max_scroll_height(max_scroll_height)
            .vscroll(true)
            .header(height, |mut header| {
                header.col(|ui| {
                    ui.heading(HASH);
                });
                header.col(|ui| {
                    ui.heading(self.header);
                });
            })
            .body(|body| {
                body.rows(height, self.series.len(), |mut row| {
                    let index = row.index();
                    row.col(|ui| {
                        ui.label(index.to_string());
                    });
                    row.col(|ui| match self.series.get(index) {
                        Ok(value) => {
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
                        }
                        Err(error) => {
                            error!(%error);
                            ui.error_with_details_on_hover(error.to_string());
                        }
                    });
                });
            });
        ui.allocate_response(Default::default(), Sense::hover())
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
