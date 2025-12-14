use crate::{
    app::{MAX_PRECISION, states::calculation::ID_SOURCE},
    assets::CHRISTIE,
    text::Text,
};
use egui::{
    ComboBox, Grid, Popup, PopupCloseBehavior, Response, RichText, ScrollArea, Slider,
    SliderClamping, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_dnd::dnd;
use egui_ext::LabeledSeparator;
use egui_l20n::prelude::*;
use egui_phosphor::regular::{ARROWS_CLOCKWISE, BOOKMARK, BROWSERS, DOTS_SIX_VERTICAL};
use lipid::prelude::*;
use ordered_float::OrderedFloat;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, Formatter},
    iter::zip,
    ops::{Deref, DerefMut},
};
use tracing::instrument;

pub(crate) const STEREOSPECIFIC_NUMBERS: [StereospecificNumbers; 3] = [
    StereospecificNumbers::OneAndTwoAndTree,
    StereospecificNumbers::OneAndThree,
    StereospecificNumbers::Two,
];

/// Calculation settings
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,

    // Display
    pub(crate) standard_deviation: bool,
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
            standard_deviation: true,
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
        ui.visuals_mut().button_frame = true;

        self.precision(ui);
        self.significant(ui);
        self.percent(ui);
        self.display_standard_deviation(ui);
        self.normalize_factors(ui);
        self.sticky(ui);
        self.truncate(ui);

        ui.labeled_separator("Parameters");
        self.standard(ui);

        ui.labeled_separator(ui.localize("Normalization"))
            .on_hover_localized("Normalization.hover");
        self.weighted(ui);
        self.christie(ui);

        // Threshold
        ui.labeled_separator(ui.localize("Threshold"))
            .on_hover_localized("Threshold.hover");
        self.is_auto_threshold(ui);
        self.auto_threshold(ui);
        self.manual_threshold(ui);
        self.sort_thresholded(ui);
        self.filter_thresholded(ui);

        if self.index.is_none() {
            // Statistics
            ui.labeled_separator(ui.localize("Statistics"));
            self.ddof(ui);
        }

        // Correlations
        ui.collapsing(ui.localize("Correlation"), |ui| {
            self.auto_size_correlations_table(ui);
            self.stereospecific_numbers(ui);
            self.correlation(ui);
            self.chaddock(ui);
        });

        // Indices
        ui.collapsing(ui.localize("Index?PluralCategory=other"), |ui| {
            self.indices(ui);
        });
    }

    // Precision
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

    /// Standard deviation
    fn display_standard_deviation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StandardDeviation"))
                .on_hover_localized("StandardDeviation.hover");
            ui.checkbox(&mut self.standard_deviation, ());
        });
    }

    /// Normalize factors
    fn normalize_factors(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("NormalizeFactors"))
                .on_hover_localized("NormalizeFactors.hover");
            ui.checkbox(&mut self.normalize_factors, ());
        });
    }

    /// Sticky columns
    fn sticky(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StickyColumns"))
                .on_hover_localized("StickyColumns.hover");
            Slider::new(&mut self.table.sticky_columns, 0..=8).ui(ui);
        });
    }

    /// Truncate headers
    fn truncate(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("TruncateHeaders"))
                .on_hover_localized("TruncateHeaders.hover");
            ui.checkbox(&mut self.table.truncate_headers, ());
        });
    }

    /// Standard
    fn standard(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Standard"))
                .on_hover_localized("Standard.hover");
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
                            .on_hover_localized("Standard?OptionCategory=none");
                    });
            })
            .response
            .on_hover_ui(|ui| {
                ui.label(ui.localize(self.standard.hover_text()));
            });
            if ui.button((BOOKMARK, "17:0")).clicked() {
                self.standard = Standard(Some("Margaric".to_owned()));
            };
        });
    }

    /// Is auto threshold
    fn is_auto_threshold(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("IsAutoThreshold"))
                .on_hover_localized("IsAutoThreshold.hover");
            ui.checkbox(&mut self.threshold.is_auto, ());
        });
    }

    /// Auto threshold
    fn auto_threshold(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("AutoThreshold"))
                .on_hover_localized("AutoThreshold.hover");
            if Slider::new(&mut self.threshold.auto.0, 0.0..=1.0)
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
            // if ui
            //     .button((BOOKMARK, if self.percent { "0.1%" } else { "0.001" }))
            //     .clicked()
            // {
            //     self.threshold.auto.0 = 0.001;
            //     self.threshold.is_auto = true;
            // }
            if ui
                .button((BOOKMARK, if self.percent { "0.25%" } else { "0.0025" }))
                .clicked()
            {
                self.threshold.auto.0 = 0.0025;
                self.threshold.is_auto = true;
            }
            // if ui
            //     .button((BOOKMARK, if self.percent { "1%" } else { "0.01" }))
            //     .clicked()
            // {
            //     self.threshold.auto.0 = 0.01;
            //     self.threshold.is_auto = true;
            // }
        });
    }

    /// Manual threshold
    fn manual_threshold(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("ManualThreshold"))
                .on_hover_localized("ManualThreshold.hover");
            let selected_text = format_list_truncated!(
                zip(&self.threshold.manual, &self.fatty_acids)
                    .filter_map(|(keep, fatty_acid)| keep.then_some(fatty_acid)),
                1
            );
            ComboBox::from_id_salt("ManualThreshold")
                .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
                .selected_text(&selected_text)
                .show_ui(ui, |ui| {
                    for (fatty_acid, selected) in zip(&self.fatty_acids, &mut self.threshold.manual)
                    {
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
        });
    }

    /// Filter thresholded
    fn filter_thresholded(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("FilterThreshold"))
                .on_hover_localized("FilterThreshold.hover");
            ui.checkbox(&mut self.threshold.filter, ());
        });
    }

    /// Sort thresholded
    fn sort_thresholded(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            // Sort by minor major
            ui.label(ui.localize("SortByMinorMajor"))
                .on_hover_localized("SortByMinorMajor.hover");
            ui.checkbox(&mut self.sort_by_minor_major, ());
        });
    }

    /// Weighted
    fn weighted(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Normalize_Weighted"))
                .on_hover_localized("Normalize_Weighted.hover");
            ui.checkbox(&mut self.weighted, ());
        });
    }

    /// Christie factors
    fn christie(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Normalize_Christie"))
                .on_hover_localized("Normalize_Christie.hover");
            ui.checkbox(&mut self.christie, ());
            ui.add_enabled_ui(self.christie, |ui| {
                MenuButton::new(BROWSERS)
                    .config(
                        MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside),
                    )
                    .ui(ui, |ui| {
                        ScrollArea::vertical().show(ui, |ui| {
                            _ = self.christie_content(ui);
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
        ui.horizontal(|ui| {
            ui.label(ui.localize("DeltaDegreesOfFreedom.abbreviation"))
                .on_hover_localized("DeltaDegreesOfFreedom")
                .on_hover_localized("DeltaDegreesOfFreedom.hover");
            Slider::new(&mut self.ddof, 0..=2)
                .update_while_editing(false)
                .ui(ui);
        });
    }

    /// Stereospecific numbers
    fn stereospecific_numbers(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("StereospecificNumber?number=many"))
                .on_hover_localized("StereospecificNumber.abbreviation?number=other");
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
        });
    }

    /// Auto size correlations table
    fn auto_size_correlations_table(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("AutoSizeCorrelationsTable"))
                .on_hover_localized("AutoSizeCorrelationsTable.hover");
            ui.toggle_value(&mut self.auto_size_correlations_table, ARROWS_CLOCKWISE);
        });
    }

    /// Correlation
    fn correlation(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
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
        });
    }

    /// Chaddock
    fn chaddock(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Chaddock"))
                .on_hover_localized("Chaddock.hover");
            ui.checkbox(&mut self.chaddock, ());
        });
    }

    /// Indices
    fn indices(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            ui.label(ui.localize("Indices"))
                .on_hover_localized("Indices.hover");
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
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate_headers: bool,
}

impl Table {
    pub(crate) fn new() -> Self {
        Self {
            resizable: false,
            sticky_columns: 0,
            truncate_headers: false,
        }
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

/// Stereospecific numbers
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum StereospecificNumbers {
    OneAndTwoAndTree,
    OneAndThree,
    Two,
}

impl StereospecificNumbers {
    pub(crate) fn id(&self) -> &'static str {
        match self {
            Self::OneAndTwoAndTree => STEREOSPECIFIC_NUMBERS123,
            Self::OneAndThree => STEREOSPECIFIC_NUMBERS13,
            Self::Two => STEREOSPECIFIC_NUMBERS2,
        }
    }

    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::OneAndTwoAndTree => "StereospecificNumber.abbreviation?number=123",
            Self::OneAndThree => "StereospecificNumber.abbreviation?number=1",
            Self::Two => "StereospecificNumber.abbreviation?number=2",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self {
            Self::OneAndTwoAndTree => "StereospecificNumber?number=123",
            Self::OneAndThree => "StereospecificNumber?number=1",
            Self::Two => "StereospecificNumber?number=2",
        }
    }
}

impl Display for StereospecificNumbers {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::OneAndTwoAndTree => f.write_str(STEREOSPECIFIC_NUMBERS123),
            Self::OneAndThree => f.write_str(STEREOSPECIFIC_NUMBERS13),
            Self::Two => f.write_str(STEREOSPECIFIC_NUMBERS2),
        }
    }
}

/// Threshold
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Threshold {
    pub(crate) auto: OrderedFloat<f64>,
    pub(crate) filter: bool,
    pub(crate) is_auto: bool,
    pub(crate) manual: Vec<bool>,
}

impl Threshold {
    pub(crate) fn new() -> Self {
        Self {
            auto: OrderedFloat(0.0),
            filter: false,
            is_auto: true,
            manual: Vec::new(),
        }
    }
}
