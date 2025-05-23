pub(crate) use self::filter::{Filter, FilterWidget};

use crate::{
    app::{
        MAX_PRECISION,
        computers::{UniqueCompositionComputed, UniqueCompositionKey},
        text::Text,
    },
    r#const::relative_atomic_mass::{H, LI, NA, NH4},
    special::composition::{COMPOSITIONS, Composition},
    utils::Hashed,
};
use egui::{
    ComboBox, DragValue, Grid, Id, Key, KeyboardShortcut, Modal, Modifiers, PopupCloseBehavior,
    Response, RichText, ScrollArea, Sides, Slider, Ui, emath::Float, util::hash,
};
use egui_ext::LabeledSeparator;
use egui_l20n::UiExt;
use egui_phosphor::regular::{ARROWS_CLOCKWISE, ERASER, MINUS, PLUS};
use indexmap::IndexMap;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, BTreeSet, HashMap, VecDeque},
    hash::{Hash, Hasher},
    ops::Deref,
};

/// Composition settings
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,

    pub(crate) index: Option<usize>,
    pub(crate) special: Special,
}

impl Settings {
    pub(crate) fn new(index: Option<usize>) -> Self {
        Self {
            percent: true,
            precision: 1,
            resizable: false,
            sticky_columns: 0,

            index: index,
            special: Special::new(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, target: &Hashed<DataFrame>) {
        ScrollArea::vertical().show(ui, |ui| {
            Grid::new("Composition").show(ui, |ui| {
                // Percent
                ui.label(ui.localize("settings-percent"));
                ui.checkbox(&mut self.percent, "");
                ui.end_row();

                // Precision
                ui.label(ui.localize("settings-precision"));
                ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
                ui.end_row();

                // Sticky
                ui.label(ui.localize("settings-sticky_columns"));
                ui.add(Slider::new(
                    &mut self.sticky_columns,
                    0..=self.special.selections.len() * 2 + 1,
                ));
                ui.end_row();

                ui.separator();
                ui.separator();
                ui.end_row();

                // Compose
                ui.label(ui.localize("settings-compose"));
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
                                    .on_hover_text(ui.localize(selected_value.hover_text()));
                                if response.clicked() {
                                    self.special.selections.push_front(Selection {
                                        composition: selected_value,
                                        filter: Default::default(),
                                    });
                                    current_value = None;
                                    ui.close_menu();
                                }
                            }
                        });
                    ui.data_mut(|data| data.insert_temp(id, current_value));
                });
                ui.end_row();
                let mut index = 0;
                self.special.selections.retain_mut(|selection| {
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
                                        .on_hover_text(ui.localize(composition.hover_text()))
                                        .changed()
                                    {
                                        selection.filter = Default::default();
                                    }
                                }
                            })
                            .response
                            .on_hover_text(ui.localize(selection.composition.hover_text()));
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
                            ui.add(FilterWidget::new(selection, &series).percent(self.percent));
                        }
                    });
                    ui.end_row();
                    index += 1;
                    keep
                });

                // // Filter
                // ui.label(ui.localize("settings-filter?case=title"));
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

                // Method
                ui.label(ui.localize("settings-method"));
                if ui.input_mut(|input| {
                    input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::G))
                }) {
                    self.special.method = Method::Gunstone;
                }
                if ui.input_mut(|input| {
                    input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::W))
                }) {
                    self.special.method = Method::VanderWal;
                }
                ComboBox::from_id_salt("method")
                    .selected_text(self.special.method.text())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.special.method,
                            Method::Gunstone,
                            Method::Gunstone.text(),
                        )
                        .on_hover_text(Method::Gunstone.hover_text());
                        ui.selectable_value(
                            &mut self.special.method,
                            Method::VanderWal,
                            Method::VanderWal.text(),
                        )
                        .on_hover_text(Method::VanderWal.hover_text());
                    })
                    .response
                    .on_hover_text(self.special.method.hover_text());
                ui.end_row();

                ui.label(ui.localize("discriminants"));
                self.special.discriminants.show(ui);
                ui.end_row();

                // Adduct
                ui.label(ui.localize("settings-adduct"));
                ui.horizontal(|ui| {
                    let adduct = &mut self.special.adduct;
                    ui.add(
                        DragValue::new(adduct)
                            .range(0.0..=f64::MAX)
                            .speed(1.0 / 10f64.powi(self.special.round_mass as _))
                            .custom_formatter(|n, _| {
                                format!("{n:.*}", self.special.round_mass as _)
                            }),
                    )
                    .on_hover_text(format!("{adduct}"));
                    ComboBox::from_id_salt(ui.auto_id_with("Adduct"))
                        .selected_text(match *adduct {
                            H => "H",
                            NH4 => "NH4",
                            NA => "Na",
                            LI => "Li",
                            _ => "",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(adduct, 0.0, "None");
                            ui.selectable_value(adduct, H, "H");
                            ui.selectable_value(adduct, NH4, "NH4");
                            ui.selectable_value(adduct, NA, "Na");
                            ui.selectable_value(adduct, LI, "Li");
                        });
                });
                ui.end_row();

                // Round mass
                ui.label(ui.localize("settings-round_mass"));
                ui.add(Slider::new(
                    &mut self.special.round_mass,
                    0..=MAX_PRECISION as _,
                ));
                ui.end_row();

                // View
                ui.separator();
                ui.labeled_separator(RichText::new(ui.localize("settings-view")).heading());
                ui.end_row();

                ui.label(ui.localize("settings-show_filtered"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("settings-show_filtered.hover"));
                    });
                ui.checkbox(&mut self.special.show_filtered, "");
                ui.end_row();

                // // Join
                // ui.label(ui.localize("settings-join"));
                // ComboBox::from_id_salt("join")
                //     .selected_text(self.join.text())
                //     .show_ui(ui, |ui| {
                //         ui.selectable_value(&mut self.join, Join::Left, Join::Left.text())
                //             .on_hover_text(Join::Left.hover_text());
                //         ui.selectable_value(&mut self.join, Join::And, Join::And.text())
                //             .on_hover_text(Join::And.hover_text());
                //         ui.selectable_value(&mut self.join, Join::Or, Join::Or.text())
                //             .on_hover_text(Join::Or.hover_text());
                //     })
                //     .response
                //     .on_hover_text(self.join.hover_text());
                // ui.end_row();

                ui.separator();
                ui.labeled_separator(RichText::new(ui.localize("settings-sort")).heading());
                ui.end_row();

                // Sort
                ui.label(ui.localize("settings-sort"));
                ComboBox::from_id_salt("sort")
                    .selected_text(self.special.sort.text())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.special.sort, Sort::Key, Sort::Key.text())
                            .on_hover_text(Sort::Key.hover_text());
                        ui.selectable_value(
                            &mut self.special.sort,
                            Sort::Value,
                            Sort::Value.text(),
                        )
                        .on_hover_text(Sort::Value.hover_text());
                    })
                    .response
                    .on_hover_text(self.special.sort.hover_text());
                ui.end_row();
                // Order
                ui.label(ui.localize("settings-order"));
                ComboBox::from_id_salt("order")
                    .selected_text(self.special.order.text())
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.special.order,
                            Order::Ascending,
                            Order::Ascending.text(),
                        )
                        .on_hover_text(Order::Ascending.hover_text());
                        ui.selectable_value(
                            &mut self.special.order,
                            Order::Descending,
                            Order::Descending.text(),
                        )
                        .on_hover_text(Order::Descending.hover_text());
                    })
                    .response
                    .on_hover_text(self.special.order.hover_text());
                ui.end_row();

                if self.index.is_none() {
                    // Statistic
                    ui.separator();
                    ui.labeled_separator(
                        RichText::new(ui.localize("settings-statistic")).heading(),
                    );
                    ui.end_row();

                    // https://numpy.org/devdocs/reference/generated/numpy.std.html
                    ui.label(ui.localize("settings-ddof"));
                    ui.add(Slider::new(&mut self.special.ddof, 0..=2));
                    ui.end_row();
                }

                ui.separator();
                ui.separator();
            });
        });
    }
}

/// Composition confirmable settings
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Special {
    pub(crate) adduct: f64,
    pub(crate) ddof: u8,
    pub(crate) selections: VecDeque<Selection>,
    pub(crate) join: Join,
    pub(crate) method: Method,
    pub(crate) order: Order,
    pub(crate) round_mass: u32,
    pub(crate) show_filtered: bool,
    pub(crate) sort: Sort,

    pub(crate) discriminants: Discriminants,
}

impl Special {
    pub(crate) fn new() -> Self {
        Self {
            adduct: 0.0,
            ddof: 1,
            selections: VecDeque::new(),
            join: Join::Left,
            method: Method::VanderWal,
            order: Order::Descending,
            round_mass: 2,
            show_filtered: false,
            sort: Sort::Value,

            discriminants: Discriminants::new(),
        }
    }
}

impl Default for Special {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Special {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.adduct.ord().hash(state);
        self.ddof.hash(state);
        self.selections.hash(state);
        self.join.hash(state);
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
                            ui.add(
                                DragValue::new(&mut values[0])
                                    .range(0.0..=f64::MAX)
                                    .speed(0.1),
                            );
                            ui.add(
                                DragValue::new(&mut values[1])
                                    .range(0.0..=f64::MAX)
                                    .speed(0.1),
                            );
                            ui.add(
                                DragValue::new(&mut values[2])
                                    .range(0.0..=f64::MAX)
                                    .speed(0.1),
                            );
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

/// Join
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Join {
    Left,
    And,
    Or,
}

impl Join {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Left => "left",
            Self::And => "and",
            Self::Or => "or",
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::Left => "left.description",
            Self::And => "and.description",
            Self::Or => "or.description",
        }
    }
}

impl From<Join> for JoinType {
    fn from(value: Join) -> Self {
        match value {
            Join::Left => JoinType::Left,
            Join::And => JoinType::Inner,
            Join::Or => JoinType::Full,
        }
    }
}

/// Method
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Method {
    Gunstone,
    VanderWal,
}

impl Method {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::Gunstone => "gunstone",
            Self::VanderWal => "vander_wal",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self {
            Self::Gunstone => "gunstone.description",
            Self::VanderWal => "vander_wal.description",
        }
    }
}

/// Sort
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Sort {
    Key,
    Value,
}

impl Sort {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Key => "key",
            Self::Value => "value",
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::Key => "key.description",
            Self::Value => "value.description",
        }
    }
}

/// Order
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) enum Order {
    Ascending,
    Descending,
}

impl Order {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Ascending => "ascending",
            Self::Descending => "descending",
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::Ascending => "ascending.description",
            Self::Descending => "descending.description",
        }
    }
}

/// Selection
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct Selection {
    pub(crate) composition: Composition,
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

mod filter;
