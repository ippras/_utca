use std::ops::{Deref, DerefMut};

use super::ID_SOURCE;
use crate::{
    app::{MAX_PRECISION, states::ColumnFilter},
    text::Text,
};
use egui::{
    ComboBox, Grid, Popup, PopupCloseBehavior, RichText, Slider, SliderClamping, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_dnd::dnd;
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{BOOKMARK, DOTS_SIX_VERTICAL, FUNNEL};
use polars::prelude::AnyValue;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};

/// Calculation settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,

    // Display
    pub(crate) display_standard_deviation: bool,
    pub(crate) normalize_factors: bool,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) table: Table,

    // General parameters
    pub(crate) ddof: u8,
    // Special parameters
    pub(crate) christie: bool,
    pub(crate) normalize: Normalize,
    pub(crate) standard: Standard,
    pub(crate) unsigned: bool,
    pub(crate) weighted: bool,
    // Mutable
    pub(crate) fatty_acids: Vec<String>,

    // Correlations
    pub(crate) correlation: Correlation,
    pub(crate) chaddock: bool, // Chaddock, R.E. (1925). Principles and methods of statistics. Boston, New York, 1925.
    // Indices
    pub(crate) indices: Indices,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            index: Some(0),
            // Display
            display_standard_deviation: true,
            normalize_factors: false,
            percent: true,
            precision: 1,
            significant: false,
            table: Table::new(),
            // General parameters
            ddof: 1,
            // Special parameters
            christie: false,
            normalize: Normalize::new(),
            standard: Standard(None),
            unsigned: true,
            weighted: false,
            // Mutable
            fatty_acids: Vec::new(),
            // Correlations
            correlation: Correlation::Pearson,
            chaddock: false,
            // Indices
            indices: Indices::new(),
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        Grid::new(ui.auto_id_with(ID_SOURCE)).show(ui, |ui| {
            ui.visuals_mut().button_frame = true;

            // Precision
            ui.label(ui.localize("Precision")).on_hover_ui(|ui| {
                ui.label(ui.localize("Precision.hover"));
            });
            ui.add(Slider::new(&mut self.precision, 1..=MAX_PRECISION));
            ui.end_row();

            // Significant
            ui.label(ui.localize("Significant")).on_hover_ui(|ui| {
                ui.label(ui.localize("Significant.hover"));
            });
            ui.checkbox(&mut self.significant, ());
            ui.end_row();

            // Percent
            ui.label(ui.localize("Percent")).on_hover_ui(|ui| {
                ui.label(ui.localize("Percent.hover"));
            });
            ui.checkbox(&mut self.percent, ());
            ui.end_row();

            // Standard deviation
            ui.label(ui.localize("StandardDeviation"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("StandardDeviation.hover"));
                });
            ui.checkbox(&mut self.display_standard_deviation, ());
            ui.end_row();

            // Normalize factors
            ui.label(ui.localize("NormalizeFactors")).on_hover_ui(|ui| {
                ui.label(ui.localize("NormalizeFactors.hover"));
            });
            ui.checkbox(&mut self.normalize_factors, ());
            ui.end_row();

            ui.heading("Table");
            ui.separator();
            ui.end_row();

            // Reset
            ui.label(ui.localize("ResetTable")).on_hover_ui(|ui| {
                ui.label(ui.localize("ResetTable.hover"));
            });
            ui.toggle_value(&mut self.table.reset_state, ());
            ui.end_row();

            // Resizable
            ui.label(ui.localize("Resizable")).on_hover_ui(|ui| {
                ui.label(ui.localize("Resizable.hover"));
            });
            ui.checkbox(&mut self.table.resizable, ());
            ui.end_row();

            // Sticky
            ui.label(ui.localize("StickyColumns")).on_hover_ui(|ui| {
                ui.label(ui.localize("StickyColumns.hover"));
            });
            Slider::new(&mut self.table.sticky_columns, 0..=8).ui(ui);
            ui.end_row();

            // Truncate
            ui.label(ui.localize("TruncateHeaders")).on_hover_ui(|ui| {
                ui.label(ui.localize("TruncateHeaders.hover"));
            });
            ui.checkbox(&mut self.table.truncate_headers, ());
            ui.end_row();

            ui.heading("Parameters");
            ui.separator();
            ui.end_row();

            ui.label(ui.localize("Standard"));
            ui.horizontal(|ui| {
                ComboBox::from_id_salt("Standard")
                    .selected_text(self.standard.text())
                    .show_ui(ui, |ui| {
                        for fatty_acid in &self.fatty_acids {
                            ui.selectable_value(
                                &mut self.standard,
                                Standard(Some(fatty_acid.clone())),
                                fatty_acid,
                            )
                            .on_hover_text(fatty_acid);
                        }
                        ui.selectable_value(&mut self.standard, Standard(None), "-")
                            .on_hover_ui(|ui| {
                                ui.label(ui.localize("Standard?OptionCategory=none"));
                            });
                    })
                    .response
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(self.standard.hover_text()));
                    });
                if ui.button((BOOKMARK, "17:0")).clicked() {
                    self.standard = Standard(Some("Margaric".to_owned()))
                };
            });
            ui.end_row();

            // Filter
            ui.label(ui.localize("FilterTableRows")).on_hover_ui(|ui| {
                ui.label(ui.localize("FilterTableRows.hover"));
            });
            MenuButton::new(FUNNEL)
                .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
                .ui(ui, |ui| {
                    ui.horizontal(|ui| {
                        Slider::new(&mut self.table.row_filter, 0.0..=1.0)
                            .clamping(SliderClamping::Always)
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
                            })
                            .logarithmic(true)
                            .ui(ui);
                        if ui.button((BOOKMARK, "0.25")).clicked() {
                            self.table.row_filter = 0.0025;
                        }
                    });
                });
            ui.end_row();
            ui.label(ui.localize("FilterTableColumns"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("FilterTableColumns.hover"));
                });
            MenuButton::new(FUNNEL)
                .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
                .ui(ui, |ui| {
                    // self.table.filter.show(ui);
                });
            ui.end_row();

            ui.label(ui.localize("Normalize_Weighted"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("Normalize_Weighted.hover"));
                });
            ui.checkbox(&mut self.weighted, ());
            ui.end_row();

            if self.index.is_none() {
                ui.label(ui.localize("Statistics"));
                ui.separator();
                ui.end_row();

                // https://numpy.org/devdocs/reference/generated/numpy.std.html
                ui.label(ui.localize("DeltaDegreesOfFreedom.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("DeltaDegreesOfFreedom"));
                    })
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("DeltaDegreesOfFreedom.hover"));
                    });
                ui.add(Slider::new(&mut self.ddof, 0..=2));
                ui.end_row();
            }

            // Correlations
            ui.heading(ui.localize("Correlation"));
            ui.separator();
            ui.end_row();

            ui.label(ui.localize("Correlation?PluralCategory=other"));
            ComboBox::from_id_salt("Correlation")
                .selected_text(self.correlation.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.correlation,
                        Correlation::Pearson,
                        Correlation::Pearson.text(),
                    )
                    .on_hover_text(Correlation::Pearson.hover_text());
                    ui.selectable_value(
                        &mut self.correlation,
                        Correlation::SpearmanRank,
                        Correlation::SpearmanRank.text(),
                    )
                    .on_hover_text(Correlation::SpearmanRank.hover_text());
                })
                .response
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(self.correlation.hover_text()));
                });
            ui.end_row();

            ui.label(ui.localize("Chaddock")).on_hover_ui(|ui| {
                ui.label(ui.localize("Chaddock.hover"));
            });
            ui.checkbox(&mut self.chaddock, ());
            ui.end_row();

            // Indices
            ui.heading(ui.localize("Index?PluralCategory=other"));
            ui.separator();
            ui.end_row();

            ui.label(ui.localize("Indices")).on_hover_ui(|ui| {
                ui.label(ui.localize("Indices.hover"));
            });
            let selected_text = format_list_truncated!(
                self.indices
                    .0
                    .iter()
                    .filter(|index| index.visible)
                    .map(|index| ui.localize(&format!("Indices_{}", index.name))),
                1
            );
            ComboBox::from_id_salt(ui.auto_id_with("Indices"))
                .selected_text(selected_text)
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                .show_ui(ui, |ui| self.indices.show(ui));
            ui.end_row();
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Correlation
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Correlation {
    Pearson,
    SpearmanRank,
}

impl Text for Correlation {
    fn text(&self) -> &'static str {
        match self {
            Self::Pearson => "Pearson",
            Self::SpearmanRank => "SpearmanRank",
        }
    }
    fn hover_text(&self) -> &'static str {
        match self {
            Self::Pearson => "Pearson.hover",
            Self::SpearmanRank => "SpearmanRank.hover",
        }
    }
}

/// Calculation table settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Table {
    pub(crate) reset_state: bool,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate_headers: bool,
    pub(crate) row_filter: f64,
    pub(crate) column_filter: ColumnFilter,
}

impl Table {
    pub(crate) fn new() -> Self {
        Self {
            row_filter: 0.0,
            reset_state: false,
            resizable: false,
            sticky_columns: 0,
            truncate_headers: false,
            column_filter: ColumnFilter::new(),
        }
    }
}

/// Standard
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Standard(Option<String>);

impl Standard {
    pub(crate) fn as_deref(&self) -> Option<&str> {
        self.0.as_deref()
    }
}

impl Text for Standard {
    fn text(&self) -> &str {
        match &self.0 {
            Some(standard) => standard,
            None => "-",
        }
    }

    fn hover_text(&self) -> &str {
        match &self.0 {
            Some(standard) => standard,
            None => "Standard?OptionCategory=none",
        }
    }
}

/// Normalize
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Normalize {
    pub(crate) experimental: bool,
    pub(crate) theoretical: bool,
}

impl Normalize {
    pub(crate) fn new() -> Self {
        Self {
            experimental: true,
            theoretical: true,
        }
    }
}

impl Default for Normalize {
    fn default() -> Self {
        Self::new()
    }
}

/// Indices
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Indices(Vec<Index>);

impl Indices {
    pub(crate) fn new() -> Self {
        Self(vec![
            Index::new("Saturated"),
            Index::new("Monounsaturated"),
            Index::new("Polyunsaturated"),
            Index::new("Unsaturated"),
            Index::new("Unsaturated?index=-9"),
            Index::new("Unsaturated?index=-6"),
            Index::new("Unsaturated?index=-3"),
            Index::new("Unsaturated?index=9"),
            Index::new("Trans"),
            Index::new("EicosapentaenoicAndDocosahexaenoic"),
            Index::new("FishLipidQuality"),
            Index::new("HealthPromotingIndex"),
            Index::new("HypocholesterolemicToHypercholesterolemic"),
            Index::new("IndexOfAtherogenicity"),
            Index::new("IndexOfThrombogenicity"),
            Index::new("LinoleicToAlphaLinolenic"),
            Index::new("Polyunsaturated-6ToPolyunsaturated-3"),
            Index::new("PolyunsaturatedToSaturated"),
            Index::new("UnsaturationIndex"),
        ])
    }

    pub(crate) fn iter_visible(&self) -> impl Iterator<Item = &str> {
        self.0
            .iter()
            .filter_map(|index| index.visible.then_some(&*index.name))
    }
}

impl Deref for Indices {
    type Target = Vec<Index>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Indices {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl Indices {
    fn show(&mut self, ui: &mut Ui) {
        let mut visible_all = None;
        let response = dnd(ui, ui.auto_id_with("Indices")).show(
            self.iter_mut(),
            |ui, index, handle, _state| {
                ui.horizontal(|ui| {
                    let visible = index.visible;
                    handle.ui(ui, |ui| {
                        ui.label(DOTS_SIX_VERTICAL);
                    });
                    ui.checkbox(&mut index.visible, "");
                    let mut text = RichText::new(ui.localize(&format!("Indices_{}", index.name)));
                    if !visible {
                        text = text.weak();
                    }
                    let response = ui.label(text);
                    Popup::context_menu(&response)
                        .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                        .show(|ui| {
                            if ui.button("Show all").clicked() {
                                visible_all = Some(true);
                            }
                            if ui.button("Hide all").clicked() {
                                visible_all = Some(false);
                            }
                        });
                });
            },
        );
        if response.is_drag_finished() {
            response.update_vec(self.as_mut_slice());
        }
        if let Some(visible) = visible_all {
            for index in &mut self.0 {
                index.visible = visible;
            }
        }
    }
}

/// Index
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Index {
    pub(crate) name: String,
    pub(crate) visible: bool,
}

impl Index {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_owned(),
            visible: true,
        }
    }
}
