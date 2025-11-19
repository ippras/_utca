pub(crate) use self::{
    composition::{
        COMPOSITIONS, Composition, ECN_MONO, ECN_STEREO, MASS_MONO, MASS_STEREO, SPECIES_MONO,
        SPECIES_POSITIONAL, SPECIES_STEREO, TYPE_MONO, TYPE_POSITIONAL, TYPE_STEREO,
        UNSATURATION_MONO, UNSATURATION_STEREO,
    },
    filter::{Filter, FilterWidget},
};

use super::ID_SOURCE;
use crate::{
    app::MAX_PRECISION,
    r#const::relative_atomic_mass::{H, LI, NA, NH4},
    text::Text,
    utils::HashedDataFrame,
};
use egui::{
    ComboBox, DragValue, Grid, Id, Key, KeyboardShortcut, Modifiers, PopupCloseBehavior,
    ScrollArea, Slider, Ui, Vec2b, Widget as _, emath::Float,
};
use egui_l20n::UiExt;
use egui_phosphor::regular::{CHART_BAR, ERASER, MINUS, PLUS, TABLE};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

/// Composition settings
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,

    pub(crate) percent: bool,
    pub(crate) float_precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,

    pub(crate) view: View,
    pub(crate) parameters: Parameters,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            index: None,

            percent: true,
            float_precision: 1,
            resizable: false,
            sticky_columns: 0,

            view: View::Table,
            parameters: Parameters::new(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, target: &HashedDataFrame) {
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new(ui.auto_id_with(ID_SOURCE)).show(ui, |ui| {
                self.percent(ui);
                ui.end_row();
                self.float_precision(ui);
                ui.end_row();
                self.sticky_columns(ui);
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();

                // Compose
                ui.label(ui.localize("Compose"));
                ui.menu_button(PLUS, |ui| {
                    let id_salt = "Composition";
                    let id = ui.auto_id_with(Id::new(id_salt));

                    let mut current_value =
                        ui.data_mut(|data| data.get_temp::<Option<Composition>>(id).flatten());
                    let max_height = ui.spacing().combo_height;
                    ScrollArea::vertical()
                        .max_height(max_height)
                        .show(ui, |ui| {
                            for selected_value in COMPOSITIONS {
                                let response = ui
                                    .selectable_value(
                                        &mut current_value,
                                        Some(selected_value),
                                        ui.localize(selected_value.text()),
                                    )
                                    .on_hover_ui(|ui| {
                                        ui.label(ui.localize(selected_value.hover_text()));
                                    });
                                if response.clicked() {
                                    self.parameters.selections.push_front(Selection {
                                        composition: selected_value,
                                        filter: Default::default(),
                                    });
                                    current_value = None;
                                    ui.close();
                                }
                            }
                        });
                    ui.data_mut(|data| data.insert_temp(id, current_value));
                });
                ui.end_row();
                let mut index = 0;
                self.parameters.selections.retain_mut(|selection| {
                    let mut keep = true;
                    ui.label("");
                    ui.horizontal(|ui| {
                        // Delete
                        keep = !ui.button(MINUS).clicked();
                        ComboBox::from_id_salt(ui.next_auto_id())
                            .selected_text(ui.localize(selection.composition.text()))
                            .show_ui(ui, |ui| {
                                for composition in COMPOSITIONS {
                                    if ui
                                        .selectable_value(
                                            &mut selection.composition,
                                            composition,
                                            ui.localize(composition.text()),
                                        )
                                        .on_hover_ui(|ui| {
                                            ui.label(ui.localize(composition.hover_text()));
                                        })
                                        .changed()
                                    {
                                        selection.filter = Default::default();
                                    }
                                }
                            })
                            .response
                            .on_hover_ui(|ui| {
                                ui.label(ui.localize(selection.composition.hover_text()));
                            });
                        // Filter
                        // let data_frame = ui.memory_mut(|memory| {
                        //     memory
                        //         .caches
                        //         .cache::<UniqueCompositionComputed>()
                        //         .get(UniqueCompositionKey {
                        //             data_frame,
                        //             selections: &self.confirmable.selections,
                        //         })
                        // });
                        // if let Some(series) = ui.memory_mut(|memory| {
                        //     memory
                        //         .caches
                        //         .cache::<UniqueCompositionComputed>()
                        //         .get(UniqueCompositionKey { data_frame, index })
                        // }) {
                        //     // ui.add(FilterWidget::new(selection, &series).percent(self.percent));
                        // }
                        if let Some(series) = target["Keys"]
                            .struct_()
                            .unwrap()
                            .fields_as_series()
                            .get(index)
                        {
                            let series = series.unique().unwrap().sort(Default::default()).unwrap();
                            FilterWidget::new(selection, &series)
                                .percent(self.percent)
                                .ui(ui);
                        }
                    });
                    ui.end_row();
                    index += 1;
                    keep
                });

                // // Filter
                // ui.label(ui.localize("Filter?case=title"));
                // for (index, selection) in &mut self.unconfirmed.selections.iter_mut().enumerate() {
                //     let series = &data_frame["Keys"].struct_().unwrap().fields_as_series()[index];
                //     ui.add(FilterWidget::new(selection, series).percent(self.percent));
                // }
                // ui.end_row();

                // if enabled {
                //     let series =
                //         &data_frame["Keys"].struct_().unwrap().fields_as_series()[index];
                //     ui.add_enabled(
                //         enabled,
                //         FilterWidget::new(selection, series).percent(self.percent),
                //     );
                // } else {
                //     ui.add_enabled(
                //         enabled,
                //         FilterWidget::new(
                //             selection,
                //             &Series::new_empty(PlSmallStr::EMPTY, &DataType::Null),
                //         ),
                //     );
                // }

                self.method(ui);
                ui.end_row();

                ui.label(ui.localize("Discriminants"));
                self.parameters.discriminants.show(ui);
                ui.end_row();

                self.adduct(ui);
                ui.end_row();
                self.round_mass(ui);
                ui.end_row();

                // View
                ui.heading(ui.localize("View"));
                ui.separator();
                ui.end_row();

                ui.label(ui.localize("ShowFiltered")).on_hover_ui(|ui| {
                    ui.label(ui.localize("ShowFiltered.hover"));
                });
                ui.checkbox(&mut self.parameters.show_filtered, "");
                ui.end_row();

                // // Join
                // ui.label(ui.localize("Join"));
                // ComboBox::from_id_salt("join")
                //     .selected_text(self.join.text())
                //     .show_ui(ui, |ui| {
                //         ui.selectable_value(&mut self.join, Join::Left, Join::Left.text())
                //             .on_hover_ui(|ui| {Join::Left.hover_text();});
                //         ui.selectable_value(&mut self.join, Join::And, Join::And.text())
                //             .on_hover_ui(|ui| {Join::And.hover_text();});
                //         ui.selectable_value(&mut self.join, Join::Or, Join::Or.text())
                //             .on_hover_ui(|ui| {Join::Or.hover_text();});
                //     })
                //     .response
                //     .on_hover_ui(|ui| {self.join.hover_text();});
                // ui.end_row();

                ui.heading(ui.localize("Sort"));
                ui.separator();
                ui.end_row();

                self.sort(ui);
                ui.end_row();
                self.order(ui);
                ui.end_row();

                if self.index.is_none() {
                    // Statistic
                    ui.label(ui.localize("Statistic"));
                    ui.separator();
                    ui.end_row();

                    self.ddof(ui);
                    ui.end_row();
                }
            });
        });
    }

    /// Float precision
    fn float_precision(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Precision")).on_hover_ui(|ui| {
            ui.label(ui.localize("Precision.hover"));
        });
        Slider::new(&mut self.float_precision, 1..=MAX_PRECISION).ui(ui);
    }

    /// Percent
    fn percent(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Percent")).on_hover_ui(|ui| {
            ui.label(ui.localize("Percent.hover"));
        });
        ui.checkbox(&mut self.percent, ());
    }

    /// Sticky columns
    fn sticky_columns(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("StickyColumns")).on_hover_ui(|ui| {
            ui.label(ui.localize("StickyColumns.hover"));
        });
        Slider::new(
            &mut self.sticky_columns,
            0..=self.parameters.selections.len() * 2 + 1,
        )
        .ui(ui);
    }

    /// Method
    fn method(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Method"));
        if ui.input_mut(|input| {
            input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::G))
        }) {
            self.parameters.method = Method::Gunstone;
        }
        if ui.input_mut(|input| {
            input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::W))
        }) {
            self.parameters.method = Method::VanderWal;
        }
        ComboBox::from_id_salt("Method")
            .selected_text(ui.localize(self.parameters.method.text()))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.parameters.method,
                    Method::Gunstone,
                    ui.localize(Method::Gunstone.text()),
                )
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(Method::Gunstone.hover_text()));
                });
                ui.selectable_value(
                    &mut self.parameters.method,
                    Method::MartinezForce,
                    ui.localize(Method::MartinezForce.text()),
                )
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(Method::MartinezForce.hover_text()));
                });
                ui.selectable_value(
                    &mut self.parameters.method,
                    Method::VanderWal,
                    ui.localize(Method::VanderWal.text()),
                )
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(Method::VanderWal.hover_text()));
                });
            })
            .response
            .on_hover_ui(|ui| {
                ui.label(ui.localize(self.parameters.method.hover_text()));
            });
    }

    /// Adduct
    fn adduct(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Adduct"));
        ui.horizontal(|ui| {
            let adduct = &mut self.parameters.adduct;
            DragValue::new(adduct)
                .range(0.0..=f64::MAX)
                .speed(1.0 / 10f64.powi(self.parameters.round_mass as _))
                .custom_formatter(|n, _| format!("{n:.*}", self.parameters.round_mass as _))
                .ui(ui)
                .on_hover_text(format!("{adduct}"));
            ComboBox::from_id_salt(ui.auto_id_with("Adduct"))
                .selected_text(match *adduct {
                    0.0 => "-",
                    H => "H",
                    NH4 => "NH4",
                    NA => "Na",
                    LI => "Li",
                    _ => "",
                })
                .show_ui(ui, |ui| {
                    ui.selectable_value(adduct, 0.0, "-");
                    ui.selectable_value(adduct, H, "H");
                    ui.selectable_value(adduct, NH4, "NH4");
                    ui.selectable_value(adduct, NA, "Na");
                    ui.selectable_value(adduct, LI, "Li");
                });
        });
    }

    /// Round mass
    fn round_mass(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("RoundMass"));
        Slider::new(&mut self.parameters.round_mass, 0..=MAX_PRECISION as _).ui(ui);
    }

    /// Sort
    fn sort(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Sort"));
        ComboBox::from_id_salt("Sort")
            .selected_text(ui.localize(self.parameters.sort.text()))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.parameters.sort,
                    Sort::Key,
                    ui.localize(Sort::Key.text()),
                )
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(Sort::Key.hover_text()));
                });
                ui.selectable_value(
                    &mut self.parameters.sort,
                    Sort::Value,
                    ui.localize(Sort::Value.text()),
                )
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(Sort::Value.hover_text()));
                });
            })
            .response
            .on_hover_ui(|ui| {
                ui.label(ui.localize(self.parameters.sort.hover_text()));
            });
    }

    /// Order
    fn order(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Order"));
        ComboBox::from_id_salt("Order")
            .selected_text(ui.localize(self.parameters.order.text()))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut self.parameters.order,
                    Order::Ascending,
                    Order::Ascending.text(),
                )
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(Order::Ascending.hover_text()));
                });
                ui.selectable_value(
                    &mut self.parameters.order,
                    Order::Descending,
                    ui.localize(Order::Descending.text()),
                )
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(Order::Descending.hover_text()));
                });
            })
            .response
            .on_hover_ui(|ui| {
                ui.label(ui.localize(self.parameters.order.hover_text()));
            });
    }

    // https://numpy.org/devdocs/reference/generated/numpy.std.html
    /// DDOF
    fn ddof(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("DeltaDegreesOfFreedom.abbreviation"))
            .on_hover_ui(|ui| {
                ui.label(ui.localize("DeltaDegreesOfFreedom"));
            })
            .on_hover_ui(|ui| {
                ui.label(ui.localize("DeltaDegreesOfFreedom.hover"));
            });
        Slider::new(&mut self.parameters.ddof, 0..=2)
            .update_while_editing(false)
            .ui(ui);
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Composition parameters
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Parameters {
    pub(crate) adduct: f64,
    pub(crate) ddof: u8,
    pub(crate) selections: VecDeque<Selection>,
    pub(crate) method: Method,
    pub(crate) order: Order,
    pub(crate) round_mass: u32,
    pub(crate) show_filtered: bool,
    pub(crate) sort: Sort,

    pub(crate) discriminants: Discriminants,
}

impl Parameters {
    pub(crate) fn new() -> Self {
        let mut selections = VecDeque::new();
        selections.push_back(Selection::new());
        Self {
            adduct: 0.0,
            ddof: 1,
            selections,
            method: Method::VanderWal,
            order: Order::Descending,
            round_mass: 2,
            show_filtered: false,
            sort: Sort::Value,

            discriminants: Discriminants::new(),
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Parameters {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.adduct.ord().hash(state);
        self.ddof.hash(state);
        self.selections.hash(state);
        self.method.hash(state);
        self.order.hash(state);
        self.round_mass.hash(state);
        self.show_filtered.hash(state);
        self.sort.hash(state);
        self.discriminants.hash(state);
    }
}

#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Discriminants(pub(crate) IndexMap<String, [f64; 3]>);

impl Discriminants {
    pub(crate) fn new() -> Self {
        Self(IndexMap::new())
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ComboBox::from_id_salt("Discriminants")
                .selected_text(self.0.len().to_string())
                .close_behavior(PopupCloseBehavior::IgnoreClicks)
                .show_ui(ui, |ui| {
                    Grid::new("Composition").show(ui, |ui| {
                        for (key, values) in &mut self.0 {
                            ui.label(key);
                            DragValue::new(&mut values[0])
                                .range(0.0..=f64::MAX)
                                .speed(0.1)
                                .ui(ui);
                            DragValue::new(&mut values[1])
                                .range(0.0..=f64::MAX)
                                .speed(0.1)
                                .ui(ui);
                            DragValue::new(&mut values[2])
                                .range(0.0..=f64::MAX)
                                .speed(0.1)
                                .ui(ui);
                            ui.end_row();
                        }
                    });
                })
                .response;
            if ui.button(ERASER).clicked() {
                for values in self.0.values_mut() {
                    *values = [1.0; 3]
                }
            }
        });
    }
}

impl FromIterator<String> for Discriminants {
    fn from_iter<T: IntoIterator<Item = String>>(iter: T) -> Self {
        Self(iter.into_iter().map(|key| (key, [1.0; 3])).collect())
    }
}

impl Hash for Discriminants {
    fn hash<H: Hasher>(&self, state: &mut H) {
        for (key, values) in &self.0 {
            key.hash(state);
            for value in values {
                value.ord().hash(state);
            }
        }
    }
}

// /// Join
// #[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
// pub(crate) enum Join {
//     Left,
//     And,
//     Or,
// }
// impl Join {
//     pub(crate) fn text(self) -> &'static str {
//         match self {
//             Self::Left => "left",
//             Self::And => "and",
//             Self::Or => "or",
//         }
//     }
//     pub(crate) fn hover_text(self) -> &'static str {
//         match self {
//             Self::Left => "left.description",
//             Self::And => "and.description",
//             Self::Or => "or.description",
//         }
//     }
// }
// impl From<Join> for JoinType {
//     fn from(value: Join) -> Self {
//         match value {
//             Join::Left => JoinType::Left,
//             Join::And => JoinType::Inner,
//             Join::Or => JoinType::Full,
//         }
//     }
// }

/// Method
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Method {
    Gunstone,
    MartinezForce,
    VanderWal,
}

impl Text for Method {
    fn text(&self) -> &'static str {
        match self {
            Self::Gunstone => "Method-Gunstone",
            Self::MartinezForce => "Method-MartinezForce",
            Self::VanderWal => "Method-VanderWal",
        }
    }
    fn hover_text(&self) -> &'static str {
        match self {
            Self::Gunstone => "Method-Gunstone.hover",
            Self::MartinezForce => "Method-MartinezForce.hover",
            Self::VanderWal => "Method-VanderWal.hover",
        }
    }
}

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    Key,
    Value,
}

impl Text for Sort {
    fn text(&self) -> &'static str {
        match self {
            Sort::Key => "Sort-ByKey",
            Sort::Value => "Sort-ByValue",
        }
    }
    fn hover_text(&self) -> &'static str {
        match self {
            Sort::Key => "Sort-ByKey.hover",
            Sort::Value => "Sort-ByValue.hover",
        }
    }
}

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Order {
    Ascending,
    Descending,
}

impl Text for Order {
    fn text(&self) -> &'static str {
        match self {
            Order::Ascending => "Order-Ascending",
            Order::Descending => "Order-Descending",
        }
    }
    fn hover_text(&self) -> &'static str {
        match self {
            Order::Ascending => "Order-Ascending.hover",
            Order::Descending => "Order-Descending.hover",
        }
    }
}

/// Selection
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct Selection {
    pub(crate) composition: Composition,
    #[serde(skip)]
    pub(crate) filter: Filter,
}

impl Selection {
    pub(crate) fn new() -> Self {
        Self {
            composition: Composition::new(),
            filter: Filter::new(),
        }
    }
}

/// View
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) enum View {
    Plot,
    #[default]
    Table,
}

impl View {
    pub(crate) const fn icon(&self) -> &'static str {
        match self {
            Self::Plot => CHART_BAR,
            Self::Table => TABLE,
        }
    }

    pub(crate) const fn title(&self) -> &'static str {
        match self {
            Self::Plot => "Plot",
            Self::Table => "Table",
        }
    }
}

impl Text for View {
    fn text(&self) -> &'static str {
        match self {
            Self::Plot => "Plot",
            Self::Table => "Table",
        }
    }

    fn hover_text(&self) -> &'static str {
        match self {
            Self::Plot => "Plot.hover",
            Self::Table => "Table.hover",
        }
    }
}

/// Plot
pub(crate) struct Plot {
    pub(crate) allow_drag: Vec2b,
    pub(crate) allow_scroll: Vec2b,
    pub(crate) show_legend: bool,
}

impl Plot {
    pub(crate) fn new() -> Self {
        Self {
            allow_drag: Vec2b { x: false, y: false },
            allow_scroll: Vec2b { x: false, y: false },
            show_legend: true,
        }
    }
}

pub(super) mod composition;
pub(super) mod filter;
