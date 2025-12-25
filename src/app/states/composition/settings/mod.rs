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
    ScrollArea, Slider, Ui, Vec2b, Widget as _,
    containers::menu::{MenuButton, MenuConfig},
    emath::Float,
};
use egui_dnd::dnd;
use egui_ext::LabeledSeparator;
use egui_l20n::prelude::*;
use egui_phosphor::regular::{BOOKMARK, CHART_BAR, DOTS_SIX_VERTICAL, ERASER, MINUS, PLUS, TABLE};
use indexmap::IndexMap;
use serde::{Deserialize, Serialize};
use std::{
    borrow::Cow,
    hash::{Hash, Hasher},
};

/// Composition settings
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,

    pub(crate) standard_deviation: bool,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) significant: bool,
    pub(crate) sticky_columns: usize,

    pub(crate) view: View,

    // Parameters
    pub(crate) ddof: u8,
    pub(crate) adduct: f64,
    pub(crate) method: Method,
    pub(crate) order: Order,
    pub(crate) round_mass: u32,
    pub(crate) compositions: Vec<Composition>,
    pub(crate) show_filtered: bool,
    pub(crate) sort: Sort,
    // Gunstone method
    pub(crate) discriminants: Discriminants,

    pub(crate) symmetry: Symmetry,
}

impl Settings {
    pub(crate) fn new() -> Self {
        let mut compositions = Vec::new();
        compositions.push(Composition::new());
        Self {
            index: None,
            // Display
            standard_deviation: true,
            percent: true,
            precision: 1,
            resizable: false,
            significant: false,
            sticky_columns: 0,

            view: View::Table,

            // Parameters
            ddof: 1,
            adduct: 0.0,
            compositions,
            method: Method::VanderWal,
            order: Order::Descending,
            round_mass: 2,
            show_filtered: false,
            sort: Sort::Value,
            // Gunstone method
            discriminants: Discriminants::new(),

            symmetry: Symmetry::new(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui) {
        ScrollArea::vertical().show(ui, |ui| {
            self.precision(ui);
            self.significant(ui);
            self.percent(ui);
            self.standard_deviation(ui);
            self.sticky_columns(ui);

            // // Filter
            // ui.label(ui.localize("Filter?case=title"));
            // for (index, selection) in &mut self.unconfirmed.selections.iter_mut().enumerate() {
            //     let series = &data_frame["Keys"].struct_().unwrap().fields_as_series()[index];
            //     FilterWidget::new(selection, series).percent(self.percent).ui(ui);
            // }
            //

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
            if self.method == Method::Gunstone {
                ui.labeled_separator(ui.localize("Gunstone"));
                self.discriminants(ui);
            }
            self.compose(ui);

            // Mass
            ui.labeled_separator(ui.localize("Mass"));
            self.adduct(ui);
            self.round_mass(ui);

            // View
            ui.labeled_separator(ui.localize("View"));

            self.show_filtered(ui);

            // Sort
            ui.labeled_separator(ui.localize("Sort"));
            self.sort(ui);
            self.order(ui);

            if self.index.is_none() {
                // Statistic
                ui.labeled_separator(ui.localize("Statistics"));
                self.ddof(ui);
            }

            // Symmetry
            ui.collapsing(ui.localize("Symmetry"), |ui| {
                self.symmetry(ui);
            });
        });
    }

    /// Precision
    fn precision(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Precision"))
                .on_hover_localized("Precision.hover");
            Slider::new(&mut self.precision, 1..=MAX_PRECISION).ui(ui);
        });
    }

    // Significant
    fn significant(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Significant"))
                .on_hover_localized("Significant.hover");
            ui.checkbox(&mut self.significant, ());
        });
    }

    /// Percent
    fn percent(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Percent"))
                .on_hover_localized("Percent.hover");
            ui.checkbox(&mut self.percent, ());
        });
    }

    /// Sticky columns
    fn sticky_columns(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StickyColumns"))
                .on_hover_localized("StickyColumns.hover");
            Slider::new(
                &mut self.sticky_columns,
                0..=self.compositions.len() * 2 + 1,
            )
            .ui(ui);
        });
    }

    /// Standard deviation
    fn standard_deviation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StandardDeviation"))
                .on_hover_localized("StandardDeviation.hover");
            ui.checkbox(&mut self.standard_deviation, ());
        });
    }

    /// Method
    fn method(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Method"));
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::G))
            }) {
                self.method = Method::Gunstone;
            }
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::W))
            }) {
                self.method = Method::VanderWal;
            }
            ComboBox::from_id_salt("Method")
                .selected_text(ui.localize(self.method.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.method,
                        Method::Gunstone,
                        ui.localize(Method::Gunstone.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Method::Gunstone.hover_text()));
                    });
                    ui.selectable_value(
                        &mut self.method,
                        Method::MartinezForce,
                        ui.localize(Method::MartinezForce.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Method::MartinezForce.hover_text()));
                    });
                    ui.selectable_value(
                        &mut self.method,
                        Method::VanderWal,
                        ui.localize(Method::VanderWal.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Method::VanderWal.hover_text()));
                    });
                })
                .response
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(self.method.hover_text()));
                });
        });
    }

    /// Discriminants
    fn discriminants(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Discriminants"));
            self.discriminants.show(ui);
        });
    }

    /// Compose
    fn compose(&mut self, ui: &mut Ui) {
        Grid::new(ui.next_auto_id()).show(ui, |ui| {
            // Compose
            ui.label(ui.localize("Compose"));
            // Plus, bookmarks
            let compositions: Vec<_> = COMPOSITIONS
                .iter()
                .filter(|composition| !self.compositions.contains(composition))
                .collect();
            ui.horizontal(|ui| {
                MenuButton::new(PLUS)
                    .config(
                        MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside),
                    )
                    .ui(ui, |ui| {
                        let max_height = ui.spacing().combo_height;
                        ScrollArea::vertical()
                            .max_height(max_height)
                            .show(ui, |ui| {
                                let id = ui.auto_id_with(Id::new("Composition"));
                                let mut current_value = ui.data_mut(|data| {
                                    data.get_temp::<Option<Composition>>(id).flatten()
                                });
                                for &&selected_value in &compositions {
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
                                        self.compositions.push(selected_value);
                                        current_value = None;
                                    }
                                }
                                ui.data_mut(|data| data.insert_temp(id, current_value));
                            });
                    });
                if ui
                    .button((
                        BOOKMARK,
                        ui.localize(SPECIES_POSITIONAL.abbreviation_text()),
                    ))
                    .clicked()
                {
                    self.compositions = vec![SPECIES_POSITIONAL];
                };
                if ui
                    .button((BOOKMARK, ui.localize(SPECIES_MONO.abbreviation_text())))
                    .clicked()
                {
                    self.compositions = vec![SPECIES_MONO];
                };
                if ui
                    .button((BOOKMARK, ui.localize(TYPE_POSITIONAL.abbreviation_text())))
                    .clicked()
                {
                    self.compositions = vec![TYPE_POSITIONAL];
                };
            });
            ui.end_row();
            // Minus, selections
            let mut delete = None;
            ui.label("");
            ui.vertical(|ui| {
                let response = dnd(ui, ui.next_auto_id()).show_vec(
                    &mut self.compositions,
                    |ui, current_value, handle, state| {
                        ui.horizontal(|ui| {
                            handle.ui(ui, |ui| {
                                ui.label(DOTS_SIX_VERTICAL);
                            });
                            // Delete
                            delete = delete.or(ui.button(MINUS).clicked().then_some(state.index));
                            ComboBox::from_id_salt(ui.next_auto_id())
                                .selected_text(ui.localize(current_value.text()))
                                .show_ui(ui, |ui| {
                                    for selected_value in &compositions {
                                        ui.selectable_value(
                                            current_value,
                                            **selected_value,
                                            ui.localize(selected_value.text()),
                                        )
                                        .on_hover_ui(
                                            |ui| {
                                                ui.label(ui.localize(selected_value.hover_text()));
                                            },
                                        );
                                    }
                                })
                                .response
                                .on_hover_ui(|ui| {
                                    ui.label(ui.localize(current_value.hover_text()));
                                });
                        });
                    },
                );
                if let Some(index) = delete {
                    self.compositions.remove(index);
                }
                if response.is_drag_finished() {
                    response.update_vec(&mut self.compositions);
                }
                // Если пуст, то вставляет значение по умолчанию (не может быть пустым).
                if self.compositions.is_empty() {
                    self.compositions.push(Composition::new());
                }
            });
        });
    }

    /// Adduct
    fn adduct(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Adduct"));
            let adduct = &mut self.adduct;
            DragValue::new(adduct)
                .range(0.0..=f64::MAX)
                .speed(1.0 / 10f64.powi(self.round_mass as _))
                .custom_formatter(|n, _| format!("{n:.*}", self.round_mass as _))
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
        ui.horizontal(|ui| {
            ui.label(ui.localize("RoundMass"));
            Slider::new(&mut self.round_mass, 1..=MAX_PRECISION as _).ui(ui);
        });
    }

    /// Show filtered
    fn show_filtered(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("ShowFiltered"))
                .on_hover_localized("ShowFiltered.hover");
            ui.checkbox(&mut self.show_filtered, "");
        });
    }

    /// Sort
    fn sort(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Sort"));
            ComboBox::from_id_salt("Sort")
                .selected_text(ui.localize(self.sort.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort, Sort::Key, ui.localize(Sort::Key.text()))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize(Sort::Key.hover_text()));
                        });
                    ui.selectable_value(
                        &mut self.sort,
                        Sort::Value,
                        ui.localize(Sort::Value.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Sort::Value.hover_text()));
                    });
                })
                .response
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(self.sort.hover_text()));
                });
        });
    }

    /// Order
    fn order(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Order"));
            ComboBox::from_id_salt("Order")
                .selected_text(ui.localize(self.order.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.order, Order::Ascending, Order::Ascending.text())
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize(Order::Ascending.hover_text()));
                        });
                    ui.selectable_value(
                        &mut self.order,
                        Order::Descending,
                        ui.localize(Order::Descending.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Order::Descending.hover_text()));
                    });
                })
                .response
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(self.order.hover_text()));
                });
        });
    }

    // https://numpy.org/devdocs/reference/generated/numpy.std.html
    /// DDOF
    fn ddof(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("DeltaDegreesOfFreedom.abbreviation"))
                .on_hover_localized("DeltaDegreesOfFreedom")
                .on_hover_localized("DeltaDegreesOfFreedom.hover");
            Slider::new(&mut self.ddof, 0..=2)
                .update_while_editing(false)
                .ui(ui);
        });
    }

    /// Symmetry
    fn symmetry(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("SymmetryA"))
                .on_hover_localized("SymmetryA.hover");
            let mut checked = self.symmetry.a.is_some();
            if ui
                .checkbox(&mut checked, ())
                .on_hover_localized("Standard?OptionCategory=none")
                .changed()
            {
                self.symmetry.a = if checked { Some(String::new()) } else { None };
            }
            ui.add_enabled_ui(checked, |ui| {
                if let Some(text) = &mut self.symmetry.a {
                    ui.text_edit_singleline(text);
                } else {
                    ui.text_edit_singleline(&mut String::new());
                }
            });
        });
        ui.horizontal(|ui| {
            ui.label(ui.localize("SymmetryB"))
                .on_hover_localized("SymmetryB.hover");
            let mut checked = self.symmetry.b.is_some();
            if ui
                .checkbox(&mut checked, ())
                .on_hover_localized("Standard?OptionCategory=none")
                .changed()
            {
                self.symmetry.b = if checked { Some(String::new()) } else { None };
            }
            ui.add_enabled_ui(checked, |ui| {
                if let Some(text) = &mut self.symmetry.b {
                    ui.text_edit_singleline(text);
                } else {
                    ui.text_edit_singleline(&mut String::new());
                }
            });
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
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

/// Symmetry
#[derive(Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
pub(crate) struct Symmetry {
    pub(crate) a: Option<String>,
    pub(crate) b: Option<String>,
}

impl Symmetry {
    pub(crate) fn new() -> Self {
        Self { a: None, b: None }
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
