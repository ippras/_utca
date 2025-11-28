use super::ID_SOURCE;
use crate::{
    app::{MAX_PRECISION, states::ColumnFilter},
    assets::CHRISTIE,
    text::Text,
};
use egui::{
    ComboBox, Grid, Popup, PopupCloseBehavior, Response, RichText, ScrollArea, Slider,
    SliderClamping, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
    emath::Float,
};
use egui_dnd::dnd;
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROWS_CLOCKWISE, BOOKMARK, BROWSERS, DOTS_SIX_VERTICAL};
use lipid::prelude::*;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    hash::{Hash, Hasher},
    iter::zip,
    ops::{Deref, DerefMut},
};
use tracing::instrument;

pub(crate) const STEREOSPECIFIC_NUMBERS: [StereospecificNumbers; 4] = [
    StereospecificNumbers::One,
    StereospecificNumbers::Two,
    StereospecificNumbers::Three,
    StereospecificNumbers::OneAndTwoAndTree,
];

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
    pub(crate) sort_by_minor_major: bool,
    pub(crate) standard: Standard,
    pub(crate) threshold: Threshold,
    pub(crate) unsigned: bool,
    pub(crate) weighted: bool,
    // Mutable
    pub(crate) fatty_acids: Vec<String>,

    // Correlations
    pub(crate) auto_size_correlations_table: bool,
    pub(crate) chaddock: bool, // Chaddock, R.E. (1925). Principles and methods of statistics. Boston, New York, 1925.
    pub(crate) correlation: Correlation,
    pub(crate) stereospecific_numbers: StereospecificNumbers,
    // Indices
    pub(crate) indices: Indices,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            index: None,
            // Display
            display_standard_deviation: true,
            normalize_factors: true,
            percent: true,
            precision: 1,
            significant: false,
            table: Table::new(),
            // General parameters
            ddof: 1,
            // Special parameters
            christie: false,
            normalize: Normalize::new(),
            sort_by_minor_major: false,
            standard: Standard(None),
            threshold: Threshold::new(),
            unsigned: true,
            weighted: false,
            // Mutable
            fatty_acids: Vec::new(),
            // Correlations
            auto_size_correlations_table: false,
            correlation: Correlation::Pearson,
            chaddock: false,
            stereospecific_numbers: StereospecificNumbers::OneAndTwoAndTree,
            // Indices
            indices: Indices::new(),
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        Grid::new(ui.auto_id_with(ID_SOURCE)).show(ui, |ui| {
            ui.visuals_mut().button_frame = true;

            self.precision(ui);
            ui.end_row();
            self.significant(ui);
            ui.end_row();
            self.percent(ui);
            ui.end_row();
            self.display_standard_deviation(ui);
            ui.end_row();
            self.normalize_factors(ui);
            ui.end_row();

            self.sticky(ui);
            ui.end_row();
            self.truncate(ui);
            ui.end_row();

            ui.heading("Parameters");
            ui.separator();
            ui.end_row();

            self.standard(ui);
            ui.end_row();

            ui.label(ui.localize("Normalization")).on_hover_ui(|ui| {
                ui.label(ui.localize("Normalization.hover"));
            });
            ui.separator();
            ui.end_row();
            self.weighted(ui);
            ui.end_row();
            self.christie(ui);
            ui.end_row();

            ui.label(ui.localize("Threshold")).on_hover_ui(|ui| {
                ui.label(ui.localize("Threshold.hover"));
            });
            ui.separator();
            ui.end_row();
            self.is_auto_threshold(ui);
            ui.end_row();
            self.auto_threshold(ui);
            ui.end_row();
            self.manual_threshold(ui);
            ui.end_row();
            self.sort_thresholded(ui);
            ui.end_row();
            self.filter_thresholded(ui);
            ui.end_row();

            if self.index.is_none() {
                ui.label(ui.localize("Statistics"));
                ui.separator();
                ui.end_row();

                self.ddof(ui);
                ui.end_row();
            }

            // Correlations
            ui.heading(ui.localize("Correlation"));
            ui.separator();
            ui.end_row();

            self.auto_size_correlations_table(ui);
            ui.end_row();
            self.stereospecific_numbers(ui);
            ui.end_row();
            self.correlation(ui);
            ui.end_row();
            self.chaddock(ui);
            ui.end_row();

            // Indices
            ui.heading(ui.localize("Index?PluralCategory=other"));
            ui.separator();
            ui.end_row();

            self.indices(ui);
            ui.end_row();
        });
    }

    // Precision
    fn precision(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Precision")).on_hover_ui(|ui| {
            ui.label(ui.localize("Precision.hover"));
        });
        Slider::new(&mut self.precision, 1..=MAX_PRECISION).ui(ui);
    }

    // Float precision
    fn significant(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Significant")).on_hover_ui(|ui| {
            ui.label(ui.localize("Significant.hover"));
        });
        ui.checkbox(&mut self.significant, ());
    }

    /// Percent
    fn percent(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Percent")).on_hover_ui(|ui| {
            ui.label(ui.localize("Percent.hover"));
        });
        ui.checkbox(&mut self.percent, ());
    }

    /// Standard deviation
    fn display_standard_deviation(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("StandardDeviation"))
            .on_hover_ui(|ui| {
                ui.label(ui.localize("StandardDeviation.hover"));
            });
        ui.checkbox(&mut self.display_standard_deviation, ());
    }

    /// Normalize factors
    fn normalize_factors(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("NormalizeFactors")).on_hover_ui(|ui| {
            ui.label(ui.localize("NormalizeFactors.hover"));
        });
        ui.checkbox(&mut self.normalize_factors, ());
    }

    /// Sticky columns
    fn sticky(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("StickyColumns")).on_hover_ui(|ui| {
            ui.label(ui.localize("StickyColumns.hover"));
        });
        Slider::new(&mut self.table.sticky_columns, 0..=8).ui(ui);
    }

    /// Truncate headers
    fn truncate(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("TruncateHeaders")).on_hover_ui(|ui| {
            ui.label(ui.localize("TruncateHeaders.hover"));
        });
        ui.checkbox(&mut self.table.truncate_headers, ());
    }

    /// Standard
    fn standard(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Standard")).on_hover_ui(|ui| {
            ui.label(ui.localize("Standard.hover"));
        });
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
    }

    /// Is auto threshold
    fn is_auto_threshold(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("IsAutoThreshold")).on_hover_ui(|ui| {
            ui.label(ui.localize("IsAutoThreshold.hover"));
        });
        ui.checkbox(&mut self.threshold.is_auto, ());
    }

    /// Auto threshold
    fn auto_threshold(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("AutoThreshold")).on_hover_ui(|ui| {
            ui.label(ui.localize("AutoThreshold.hover"));
        });
        ui.horizontal(|ui| {
            if Slider::new(&mut self.threshold.auto, 0.0..=1.0)
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
                .update_while_editing(false)
                .ui(ui)
                .changed()
            {
                self.threshold.is_auto = true;
            }
            if ui.button((BOOKMARK, "0.25")).clicked() {
                self.threshold.auto = 0.0025;
                self.threshold.is_auto = true;
            }
        });
    }

    /// Manual threshold
    fn manual_threshold(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("ManualThreshold")).on_hover_ui(|ui| {
            ui.label(ui.localize("ManualThreshold.hover"));
        });
        let selected_text = format_list_truncated!(
            zip(&self.threshold.manual, &self.fatty_acids)
                .filter_map(|(keep, fatty_acid)| keep.then_some(fatty_acid)),
            1
        );
        ComboBox::from_id_salt("ManualThreshold")
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .selected_text(&selected_text)
            .show_ui(ui, |ui| {
                for (fatty_acid, selected) in zip(&self.fatty_acids, &mut self.threshold.manual) {
                    if ui
                        .toggle_value(selected, fatty_acid)
                        .on_hover_text(fatty_acid)
                        .changed()
                    {
                        self.threshold.is_auto = false;
                    }
                }
            })
            .response
            .on_hover_ui(|ui| {
                ui.label(selected_text);
            });
    }

    /// Filter thresholded
    fn filter_thresholded(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("FilterThreshold")).on_hover_ui(|ui| {
            ui.label(ui.localize("FilterThreshold.hover"));
        });
        ui.checkbox(&mut self.threshold.filter, ());
    }

    /// Sort thresholded
    fn sort_thresholded(&mut self, ui: &mut Ui) {
        // Sort by minor major
        ui.label(ui.localize("SortByMinorMajor")).on_hover_ui(|ui| {
            ui.label(ui.localize("SortByMinorMajor.hover"));
        });
        ui.checkbox(&mut self.sort_by_minor_major, ());
    }

    /// Weighted
    fn weighted(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Normalize_Weighted"))
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Normalize_Weighted.hover"));
            });
        ui.checkbox(&mut self.weighted, ());
    }

    /// Christie factors
    fn christie(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Normalize_Christie"))
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Normalize_Christie.hover"));
            });
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.christie, ());
            ui.add_enabled_ui(self.christie, |ui| {
                MenuButton::new(BROWSERS)
                    .config(
                        MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside),
                    )
                    .ui(ui, |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            let _ = self.christie_content(ui);
                        });
                    });
            });
        });
    }

    #[instrument(skip(self, ui), err)]
    fn christie_content(&mut self, ui: &mut Ui) -> PolarsResult<Response> {
        let height = CHRISTIE.data.data_frame.height();
        let fatty_acid = CHRISTIE.data.data_frame[FATTY_ACID].fatty_acid();
        let factor = CHRISTIE.data.data_frame["Factor"].f64()?;
        let inner_response =
            Grid::new(ui.auto_id_with(ID_SOURCE)).show(ui, |ui| -> PolarsResult<()> {
                for index in 0..height {
                    ui.label(fatty_acid.delta()?.get(index).unwrap_or_default());
                    ui.label(
                        factor
                            .get(index)
                            .map_or_default(|factor| factor.to_string()),
                    );
                    ui.end_row();
                }
                Ok(())
            });
        inner_response.inner?;
        Ok(inner_response.response)
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
        Slider::new(&mut self.ddof, 0..=2)
            .update_while_editing(false)
            .ui(ui);
    }

    /// Stereospecific numbers
    fn stereospecific_numbers(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("StereospecificNumber?number=many"))
            .on_hover_ui(|ui| {
                ui.label(ui.localize("StereospecificNumber.abbreviation?number=other"));
            });
        ComboBox::from_id_salt(ui.auto_id_with("StereospecificNumbers"))
            .selected_text(ui.localize(self.stereospecific_numbers.text()))
            .show_ui(ui, |ui| {
                for stereospecific_number in STEREOSPECIFIC_NUMBERS {
                    ui.selectable_value(
                        &mut self.stereospecific_numbers,
                        stereospecific_number,
                        ui.localize(stereospecific_number.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(stereospecific_number.hover_text()));
                    });
                }
            })
            .response
            .on_hover_ui(|ui| {
                ui.label(ui.localize(self.stereospecific_numbers.hover_text()));
            });
    }

    /// Auto size correlations table
    fn auto_size_correlations_table(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("AutoSizeCorrelationsTable"))
            .on_hover_ui(|ui| {
                ui.label(ui.localize("AutoSizeCorrelationsTable.hover"));
            });
        ui.toggle_value(&mut self.auto_size_correlations_table, ARROWS_CLOCKWISE);
    }

    /// Correlation
    fn correlation(&mut self, ui: &mut Ui) {
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
    }

    /// Chaddock
    fn chaddock(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Chaddock")).on_hover_ui(|ui| {
            ui.label(ui.localize("Chaddock.hover"));
        });
        ui.checkbox(&mut self.chaddock, ());
    }

    /// Indices
    fn indices(&mut self, ui: &mut Ui) {
        ui.label(ui.localize("Indices")).on_hover_ui(|ui| {
            ui.label(ui.localize("Indices.hover"));
        });
        let selected_text = format_list_truncated!(
            self.indices
                .0
                .iter()
                .filter(|index| index.visible)
                .map(|index| ui.localize(&index.name)),
            1
        );
        ComboBox::from_id_salt(ui.auto_id_with("Indices"))
            .selected_text(selected_text)
            .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            .show_ui(ui, |ui| self.indices.show(ui));
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
    pub(crate) column_filter: ColumnFilter,
}

impl Table {
    pub(crate) fn new() -> Self {
        Self {
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
            Index::new("Conjugated"),
            Index::new("Saturated"),
            Index::new("Monounsaturated"),
            Index::new("Polyunsaturated"),
            Index::new("Unsaturated"),
            Index::new("Unsaturated-9"),
            Index::new("Unsaturated-6"),
            Index::new("Unsaturated-3"),
            Index::new("Unsaturated9"),
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
            Index::new("IodineValue"),
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
                    let mut text = RichText::new(ui.localize(&index.name));
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

/// Stereospecific numbers
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum StereospecificNumbers {
    One,
    Two,
    Three,
    OneAndTwoAndTree,
}

impl StereospecificNumbers {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::One => "StereospecificNumber.abbreviation?number=1",
            Self::Two => "StereospecificNumber.abbreviation?number=2",
            Self::Three => "StereospecificNumber.abbreviation?number=3",
            Self::OneAndTwoAndTree => "StereospecificNumber.abbreviation?number=123",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self {
            Self::One => "StereospecificNumber?number=1",
            Self::Two => "StereospecificNumber?number=2",
            Self::Three => "StereospecificNumber?number=3",
            Self::OneAndTwoAndTree => "StereospecificNumber?number=123",
        }
    }
}

impl Display for StereospecificNumbers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::One => f.write_str(STEREOSPECIFIC_NUMBERS13),
            Self::Two => f.write_str(STEREOSPECIFIC_NUMBERS2),
            Self::Three => f.write_str(STEREOSPECIFIC_NUMBERS13),
            Self::OneAndTwoAndTree => f.write_str(STEREOSPECIFIC_NUMBERS123),
        }
    }
}

/// Threshold
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Threshold {
    pub(crate) auto: f64,
    pub(crate) filter: bool,
    pub(crate) is_auto: bool,
    pub(crate) manual: Vec<bool>,
}

impl Threshold {
    pub(crate) fn new() -> Self {
        Self {
            auto: 0.0,
            filter: false,
            is_auto: true,
            manual: Vec::new(),
        }
    }
}

impl Hash for Threshold {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.auto.ord().hash(state);
        self.filter.hash(state);
        self.is_auto.hash(state);
        self.manual.hash(state);
    }
}
