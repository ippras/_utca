use super::Filter;
use crate::{
    app::text::Text,
    special::composition::{
        Composition, ECNC, MC, PECNC, PMC, PSC, PTC, PUC, SC, SECNC, SMC, SSC, STC, SUC, TC, UC,
    },
};
use egui::{Response, Slider, SliderClamping, TextStyle, Ui, Widget};
use egui_ext::LabeledSeparator as _;
use egui_extras::{Column, TableBuilder};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{FUNNEL, FUNNEL_X, HASH};
use lipid::triacylglycerol::Stereospecificity;
use polars::prelude::*;

/// Filter widget
pub struct FilterWidget<'a> {
    filter: &'a mut Filter,
    composition: &'a Composition,
    data_frame: &'a DataFrame,
    percent: bool,
}

impl<'a> FilterWidget<'a> {
    pub fn new(
        filter: &'a mut Filter,
        composition: &'a Composition,
        data_frame: &'a DataFrame,
    ) -> Self {
        Self {
            filter,
            composition,
            data_frame,
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
        let title = if *self.filter == Default::default() {
            FUNNEL_X
        } else {
            ui.visuals_mut().widgets.inactive = ui.visuals().widgets.active;
            FUNNEL
        };
        ui.menu_button(title, |ui| -> PolarsResult<()> {
            ui.heading(format!(
                "{} {}",
                self.composition.text(),
                ui.localize("settings-filter?case=lower"),
            ));
            let column = match *self.composition {
                ECNC | PECNC | SECNC => &self.data_frame["EquivalentCarbonNumber"],
                MC | PMC | SMC => &self.data_frame["Mass"],
                SC | PSC | SSC => &self.data_frame["Species"],
                TC | PTC | STC => &self.data_frame["Type"],
                UC | PUC | SUC => &self.data_frame["Unsaturation"],
            };
            let column = column.unique()?.sort(Default::default())?;
            let stereospecificity = self.composition.stereospecificity;
            // Key
            ui.labeled_separator("Key");
            let max_scroll_height = ui.spacing().combo_height;
            let height = TextStyle::Body
                .resolve(ui.style())
                .size
                .max(ui.spacing().interact_size.y);
            let count = match stereospecificity {
                None => 1,
                Some(Stereospecificity::Positional) => 2,
                Some(Stereospecificity::Stereo) => 3,
            };
            TableBuilder::new(ui)
                .auto_shrink(false)
                .column(Column::auto().auto_size_this_frame(true))
                .columns(Column::remainder(), count)
                .max_scroll_height(max_scroll_height)
                .vscroll(true)
                .header(height, |mut header| {
                    header.col(|ui| {
                        ui.heading(HASH);
                    });
                    match stereospecificity {
                        None => {
                            header.col(|ui| {
                                ui.heading("SN-1,2,3");
                            });
                        }
                        Some(Stereospecificity::Positional) => {
                            header.col(|ui| {
                                ui.heading("SN-1,3");
                            });
                            header.col(|ui| {
                                ui.heading("SN-2");
                            });
                        }
                        Some(Stereospecificity::Stereo) => {
                            header.col(|ui| {
                                ui.heading("SN-1");
                            });
                            header.col(|ui| {
                                ui.heading("SN-2");
                            });
                            header.col(|ui| {
                                ui.heading("SN-3");
                            });
                        }
                    }
                })
                .body(|body| {
                    body.rows(height, column.len(), |mut row| {
                        let index = row.index();
                        row.col(|ui| {
                            ui.label(index.to_string());
                        });
                        let mut response = None;
                        match stereospecificity {
                            None => {
                                row.col(|ui| {
                                    let value = column.get(index).unwrap();
                                    let contains = self.filter.key.contains(&value);
                                    let mut selected = contains;
                                    response.insert_or_union(
                                        ui.toggle_value(&mut selected, value.str_value()),
                                    );
                                    if selected && !contains {
                                        self.filter.key.push(value.into_static());
                                    } else if !selected && contains {
                                        self.filter.remove(&value);
                                    }
                                });
                            }
                            Some(Stereospecificity::Positional) => {
                                row.col(|ui| {
                                    let value = column.get(index).unwrap();
                                    let contains = self.filter.key.contains(&value);
                                    let mut selected = contains;
                                    response.insert_or_union(
                                        ui.toggle_value(&mut selected, value.str_value()),
                                    );
                                    if selected && !contains {
                                        self.filter.key.push(value.into_static());
                                    } else if !selected && contains {
                                        self.filter.remove(&value);
                                    }
                                });
                                row.col(|ui| {
                                    let value = column.get(index).unwrap();
                                    let contains = self.filter.key.contains(&value);
                                    let mut selected = contains;
                                    response.insert_or_union(
                                        ui.toggle_value(&mut selected, value.str_value()),
                                    );
                                    if selected && !contains {
                                        self.filter.key.push(value.into_static());
                                    } else if !selected && contains {
                                        self.filter.remove(&value);
                                    }
                                });
                            }
                            Some(Stereospecificity::Stereo) => {
                                row.col(|ui| {
                                    let value = column.get(index).unwrap();
                                    let contains = self.filter.key.contains(&value);
                                    let mut selected = contains;
                                    response.insert_or_union(
                                        ui.toggle_value(&mut selected, value.str_value()),
                                    );
                                    if selected && !contains {
                                        self.filter.key.push(value.into_static());
                                    } else if !selected && contains {
                                        self.filter.remove(&value);
                                    }
                                });
                                row.col(|ui| {
                                    let value = column.get(index).unwrap();
                                    let contains = self.filter.key.contains(&value);
                                    let mut selected = contains;
                                    response.insert_or_union(
                                        ui.toggle_value(&mut selected, value.str_value()),
                                    );
                                    if selected && !contains {
                                        self.filter.key.push(value.into_static());
                                    } else if !selected && contains {
                                        self.filter.remove(&value);
                                    }
                                });
                                row.col(|ui| {
                                    let value = column.get(index).unwrap();
                                    let contains = self.filter.key.contains(&value);
                                    let mut selected = contains;
                                    response.insert_or_union(
                                        ui.toggle_value(&mut selected, value.str_value()),
                                    );
                                    if selected && !contains {
                                        self.filter.key.push(value.into_static());
                                    } else if !selected && contains {
                                        self.filter.remove(&value);
                                    }
                                });
                            }
                        }
                        response
                            .unwrap_and_union(row.response())
                            .context_menu(|ui| {
                                if ui.button(format!("{FUNNEL} Select all")).clicked() {
                                    self.filter.key = column
                                        .as_materialized_series()
                                        .iter()
                                        .map(AnyValue::into_static)
                                        .collect();
                                    ui.close_menu();
                                }
                                if ui.button(format!("{FUNNEL_X} Unselect all")).clicked() {
                                    self.filter.key = Vec::new();
                                    ui.close_menu();
                                }
                            });
                    });
                });
            // Value
            ui.separator();
            ui.horizontal(|ui| {
                ui.label("Value");
                ui.add(
                    Slider::new(&mut self.filter.value, 0.0..=1.0)
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

/// Extension methods for [`Response`]
trait ResponseExt {
    fn insert_or_union(&mut self, other: Response);

    fn unwrap_and_union(self, other: Response) -> Response;
}

impl ResponseExt for Option<Response> {
    fn insert_or_union(&mut self, other: Response) {
        *self = Some(self.take().unwrap_and_union(other));
    }

    fn unwrap_and_union(self, other: Response) -> Response {
        match self {
            Some(outer_response) => outer_response | other,
            None => other,
        }
    }
}
