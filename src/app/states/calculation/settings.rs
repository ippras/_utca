use super::ID_SOURCE;
use crate::app::{MAX_PRECISION, states::ColumnFilter};
use egui::{
    Grid, PopupCloseBehavior, Slider, SliderClamping, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROWS_CLOCKWISE, DATABASE, FUNNEL};
use polars::prelude::AnyValue;
use serde::{Deserialize, Serialize};

/// Calculation settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,

    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) display_standard_deviation: bool,
    pub(crate) normalize_factors: bool,
    pub(crate) table: Table,

    pub(crate) parameters: Parameters,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            index: Some(0),

            percent: true,
            precision: 1,
            significant: false,
            display_standard_deviation: false,
            normalize_factors: false,
            table: Table::new(),

            parameters: Parameters::new(),
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
                        if ui.button(ARROWS_CLOCKWISE).clicked() {
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
            ui.checkbox(&mut self.parameters.weighted, ());
            ui.end_row();
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
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

/// Calculation parameters
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Parameters {
    pub(crate) weighted: bool,
    pub(crate) normalize: Normalize,
    pub(crate) unsigned: bool,
    pub(crate) christie: bool,
    pub(crate) ddof: u8,
}

impl Parameters {
    pub(crate) fn new() -> Self {
        Self {
            weighted: false,
            normalize: Normalize::new(),
            unsigned: true,
            christie: false,
            ddof: 1,
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
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
