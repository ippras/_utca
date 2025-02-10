use super::State;
use crate::app::MAX_PRECISION;
use egui::{ComboBox, Grid, Key, KeyboardShortcut, Modifiers, RichText, Slider, Ui};
use egui_ext::LabeledSeparator;
use egui_l20n::UiExt as _;
use egui_phosphor::regular::BROWSERS;
use serde::{Deserialize, Serialize};

/// Calculation settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) index: Option<usize>,

    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate_headers: bool,

    pub(crate) fraction: Fraction,
    pub(crate) from: From,
    pub(crate) normalize: Normalize,
    pub(crate) unsigned: bool,
    pub(crate) christie: bool,
    pub(crate) ddof: u8,

    pub(crate) factors: bool,
    pub(crate) theoretical: bool,
}

impl Settings {
    pub(crate) const fn new(index: Option<usize>) -> Self {
        Self {
            index,
            percent: true,
            precision: 1,
            resizable: false,
            sticky_columns: 0,
            truncate_headers: false,
            fraction: Fraction::AsIs,
            from: From::Mag2,
            normalize: Normalize::new(),
            unsigned: true,
            christie: false,
            ddof: 1,
            factors: true,
            theoretical: true,
        }
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui, state: &mut State) {
        Grid::new("Calculation").show(ui, |ui| {
            // Precision
            let mut response = ui.label(ui.localize("settings-precision"));
            response |= ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("settings-precision.hover"));
            });
            ui.end_row();

            // Percent
            let mut response = ui.label(ui.localize("settings-percent"));
            response |= ui.checkbox(&mut self.percent, "");
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("settings-percent.hover"));
            });
            ui.end_row();

            // Sticky
            let mut response = ui.label(ui.localize("settings-sticky_columns"));
            response |= ui.add(Slider::new(&mut self.sticky_columns, 0..=14));
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("settings-sticky_columns.hover"));
            });
            ui.end_row();

            // Truncate
            let mut response = ui.label(ui.localize("settings-truncate_headers"));
            response |= ui.checkbox(&mut self.truncate_headers, "");
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("settings-truncate_headers.hover"));
            });
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Fraction
            let mut response = ui.label(ui.localize("settings-fraction"));
            let fraction = &mut self.fraction;
            response |= ComboBox::from_id_salt("fraction")
                .selected_text(ui.localize(fraction.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        fraction,
                        Fraction::AsIs,
                        ui.localize(Fraction::AsIs.text()),
                    )
                    .on_hover_text(Fraction::AsIs.hover_text());
                    ui.selectable_value(
                        fraction,
                        Fraction::Fraction,
                        ui.localize(Fraction::Fraction.text()),
                    )
                    .on_hover_text(Fraction::Fraction.hover_text());
                })
                .response
                .on_hover_text(fraction.hover_text());
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("settings-fraction.hover"));
            });
            ui.end_row();

            // Calculate
            ui.label(ui.localize("settings-from"))
                .on_hover_text(ui.localize("settings-from.hover"));
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::Num1))
            }) {
                self.from = From::Dag1223;
            }
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::Num2))
            }) {
                self.from = From::Mag2;
            }
            ComboBox::from_id_salt("1,3")
                .selected_text(self.from.text())
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.from, From::Dag1223, From::Dag1223.text())
                        .on_hover_text(From::Dag1223.hover_text());
                    ui.selectable_value(&mut self.from, From::Mag2, From::Mag2.text())
                        .on_hover_text(From::Mag2.hover_text());
                })
                .response
                .on_hover_text(self.from.hover_text());
            ui.end_row();

            // Signed
            ui.label(ui.localize("settings-unsigned"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("settings-unsigned.hover"));
                });
            ui.checkbox(&mut self.unsigned, ui.localize("settings-theoretical"));
            ui.end_row();

            // Normalize
            ui.label(ui.localize("settings-normalize"));
            ui.horizontal(|ui| {
                ui.checkbox(
                    &mut self.normalize.experimental,
                    ui.localize("settings-experimental"),
                );
                ui.checkbox(
                    &mut self.normalize.theoretical,
                    ui.localize("settings-theoretical"),
                );
            });
            ui.end_row();

            // Christie
            let mut response = ui.label(ui.localize("settings-christie"));
            ui.horizontal(|ui| {
                response |= ui.checkbox(&mut self.christie, "");
                ui.toggle_value(
                    &mut state.open_christie_window,
                    RichText::new(BROWSERS).heading(),
                );
                response.on_hover_ui(|ui| {
                    ui.label(ui.localize("settings-christie.hover"));
                });
            });
            ui.end_row();

            ui.separator();
            ui.labeled_separator(RichText::new(ui.localize("settings-show")).heading());
            ui.end_row();

            // Factors
            ui.label(ui.localize("settings-factors"));
            ui.checkbox(&mut self.factors, "");
            ui.end_row();

            // Theoretical
            ui.label(ui.localize("settings-theoretical"));
            ui.checkbox(&mut self.theoretical, "");
            ui.end_row();

            if self.index.is_none() {
                ui.separator();
                ui.labeled_separator(RichText::new(ui.localize("settings-statistic")).heading());
                ui.end_row();

                // ui.label(ui.localize("settings-merge"));
                // ui.checkbox(&mut self.merge, "");
                // ComboBox::from_id_salt("show")
                //     .selected_text(self.show.text())
                //     .show_ui(ui, |ui| {
                //         ui.selectable_value(&mut self.show, Show::Separate, Show::Separate.text())
                //             .on_hover_text(Show::Separate.hover_text());
                //         ui.selectable_value(&mut self.show, Show::Join, Show::Join.text())
                //             .on_hover_text(Show::Join.hover_text());
                //     })
                //     .response
                //     .on_hover_text(self.show.hover_text());
                // ui.end_row();

                // https://numpy.org/devdocs/reference/generated/numpy.std.html
                ui.label(ui.localize("settings-ddof")).on_hover_ui(|ui| {
                    ui.label(ui.localize("settings-ddof.hover"));
                });
                ui.add(Slider::new(&mut self.ddof, 0..=2));
                ui.end_row();
            }
        });
    }
}

/// Fraction
///
/// [wikipedia.org](https://en.wikipedia.org/wiki/Mole_fraction)
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Fraction {
    AsIs,
    Fraction,
}

impl Fraction {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::AsIs => "settings-as_is",
            Self::Fraction => "settings-pchelkin",
        }
    }

    // col(name) / (col(name) * col("FA").fa().mass() / lit(10)).sum()
    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::AsIs => "S / ∑ S",
            Self::Fraction => "S / ∑(S * M)",
        }
    }
}

/// From
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum From {
    Dag1223,
    Mag2,
}

impl From {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::Dag1223 => "from_dag",
            Self::Mag2 => "from_mag",
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::Dag1223 => "from_dag.hover",
            Self::Mag2 => "from_mag.hover",
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
    pub(crate) const fn new() -> Self {
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
