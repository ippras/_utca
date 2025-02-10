pub(crate) use self::filter::{Filter, FilterWidget};

use crate::{
    app::{MAX_PRECISION, text::Text},
    r#const::relative_atomic_mass::{H, LI, NA, NH4},
    special::composition::{COMPOSITIONS, Composition},
};
use egui::{
    ComboBox, DragValue, Grid, Key, KeyboardShortcut, Modifiers, RichText, Slider, Ui, emath::Float,
};
use egui_ext::LabeledSeparator;
use egui_l20n::UiExt;
use egui_phosphor::regular::{MINUS, PLUS};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

/// Composition settings
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,

    pub(crate) confirmed: Confirmable,
    pub(super) unconfirmed: Confirmable,
}

impl Settings {
    pub(crate) fn new(index: Option<usize>) -> Self {
        Self {
            index: index,
            percent: true,
            precision: 1,
            resizable: false,
            sticky_columns: 0,

            confirmed: Confirmable::new(),
            unconfirmed: Confirmable::new(),
        }
    }

    pub(crate) fn show(&mut self, ui: &mut Ui, data_frame: &DataFrame) {
        Grid::new("Composition").show(ui, |ui| {
            // Precision
            ui.label(ui.localize("settings-precision"));
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // Percent
            ui.label(ui.localize("settings-percent"));
            ui.checkbox(&mut self.percent, "");
            ui.end_row();

            // Sticky
            ui.label(ui.localize("settings-sticky_columns"));
            ui.add(Slider::new(
                &mut self.sticky_columns,
                0..=self.unconfirmed.selections.len() * 2 + 1,
            ));
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Compose
            ui.label(ui.localize("settings-compose"));
            if ui.button(PLUS).clicked() {
                self.unconfirmed.selections.push_front(Selection::new());
            }
            ui.end_row();
            let mut index = 0;
            self.unconfirmed.selections.retain_mut(|selection| {
                let mut keep = true;
                ui.label("");
                ui.horizontal(|ui| {
                    // Delete
                    keep = !ui.button(MINUS).clicked();
                    ComboBox::from_id_salt(ui.next_auto_id())
                        .selected_text(ui.localize(selection.composition.text()))
                        .show_ui(ui, |ui| {
                            for composition in COMPOSITIONS {
                                ui.selectable_value(
                                    &mut selection.composition,
                                    composition,
                                    ui.localize(composition.text()),
                                )
                                .on_hover_text(ui.localize(composition.hover_text()));
                            }
                        })
                        .response
                        .on_hover_text(selection.composition.hover_text());
                    // Filter
                    let series = &data_frame["Keys"].struct_().unwrap().fields_as_series()[index];
                    ui.add(FilterWidget::new(selection, series).percent(self.percent));
                });
                ui.end_row();
                index += 1;
                keep
            });

            // Method
            ui.label(ui.localize("settings-method"));
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::G))
            }) {
                self.unconfirmed.method = Method::Gunstone;
            }
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::W))
            }) {
                self.unconfirmed.method = Method::VanderWal;
            }
            ComboBox::from_id_salt("method")
                .selected_text(self.unconfirmed.method.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.unconfirmed.method,
                        Method::Gunstone,
                        Method::Gunstone.text(),
                    )
                    .on_hover_text(Method::Gunstone.hover_text());
                    ui.selectable_value(
                        &mut self.unconfirmed.method,
                        Method::VanderWal,
                        Method::VanderWal.text(),
                    )
                    .on_hover_text(Method::VanderWal.hover_text());
                })
                .response
                .on_hover_text(self.unconfirmed.method.hover_text());
            ui.end_row();

            // Adduct
            ui.label(ui.localize("settings-adduct"));
            ui.horizontal(|ui| {
                let adduct = &mut self.unconfirmed.adduct;
                ui.add(
                    DragValue::new(adduct)
                        .range(0.0..=f64::MAX)
                        .speed(1.0 / 10f64.powi(self.unconfirmed.round_mass as _))
                        .custom_formatter(|n, _| {
                            format!("{n:.*}", self.unconfirmed.round_mass as _)
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
                &mut self.unconfirmed.round_mass,
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
            ui.checkbox(&mut self.unconfirmed.show_filtered, "");
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
                .selected_text(self.unconfirmed.sort.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.unconfirmed.sort, Sort::Key, Sort::Key.text())
                        .on_hover_text(Sort::Key.hover_text());
                    ui.selectable_value(
                        &mut self.unconfirmed.sort,
                        Sort::Value,
                        Sort::Value.text(),
                    )
                    .on_hover_text(Sort::Value.hover_text());
                })
                .response
                .on_hover_text(self.unconfirmed.sort.hover_text());
            ui.end_row();
            // Order
            ui.label(ui.localize("settings-order"));
            ComboBox::from_id_salt("order")
                .selected_text(self.unconfirmed.order.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.unconfirmed.order,
                        Order::Ascending,
                        Order::Ascending.text(),
                    )
                    .on_hover_text(Order::Ascending.hover_text());
                    ui.selectable_value(
                        &mut self.unconfirmed.order,
                        Order::Descending,
                        Order::Descending.text(),
                    )
                    .on_hover_text(Order::Descending.hover_text());
                })
                .response
                .on_hover_text(self.unconfirmed.order.hover_text());
            ui.end_row();

            if self.index.is_none() {
                // Statistic
                ui.separator();
                ui.labeled_separator(RichText::new(ui.localize("settings-statistic")).heading());
                ui.end_row();

                // https://numpy.org/devdocs/reference/generated/numpy.std.html
                ui.label(ui.localize("settings-ddof"));
                ui.add(Slider::new(&mut self.unconfirmed.ddof, 0..=2));
                ui.end_row();
            }

            ui.separator();
            ui.separator();
        });
    }
}

/// Composition confirmable settings
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Confirmable {
    pub(crate) adduct: f64,
    pub(crate) ddof: u8,
    pub(crate) selections: VecDeque<Selection>,
    pub(crate) join: Join,
    pub(crate) method: Method,
    pub(crate) order: Order,
    pub(crate) round_mass: u32,
    pub(crate) show_filtered: bool,
    pub(crate) sort: Sort,
}

impl Confirmable {
    pub(crate) const fn new() -> Self {
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
        }
    }
}

impl Default for Confirmable {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Confirmable {
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
