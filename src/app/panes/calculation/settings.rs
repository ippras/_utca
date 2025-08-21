use super::{ID_SOURCE, State};
use crate::app::MAX_PRECISION;
use egui::{ComboBox, Grid, Key, KeyboardShortcut, Modifiers, RichText, Slider, Ui};
use egui_ext::{LabeledSeparator, Markdown as _};
use egui_l20n::{ResponseExt, UiExt as _};
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

    pub(crate) weighted: bool,
    pub(crate) from: From,
    pub(crate) normalize: Normalize,
    pub(crate) unsigned: bool,
    pub(crate) christie: bool,
    pub(crate) ddof: u8,

    pub(crate) factors: bool,
    pub(crate) theoretical: bool,
}

impl Settings {
    pub(crate) fn new(index: Option<usize>) -> Self {
        Self {
            index,
            percent: true,
            precision: 1,
            resizable: false,
            sticky_columns: 0,
            truncate_headers: false,
            weighted: false,
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
        Grid::new(ID_SOURCE).show(ui, |ui| {
            // Precision
            ui.label(ui.localize("Precision"))
                .on_hover_localized("Precision.hover");
            ui.add(Slider::new(&mut self.precision, 0..=MAX_PRECISION));
            ui.end_row();

            // Percent
            ui.label(ui.localize("Percent"))
                .on_hover_localized("Percent.hover");
            ui.checkbox(&mut self.percent, "");
            ui.end_row();

            // Sticky
            ui.label(ui.localize("StickyColumns"))
                .on_hover_localized("StickyColumns.hover");
            ui.add(Slider::new(&mut self.sticky_columns, 0..=14));
            ui.end_row();

            // Truncate
            ui.label(ui.localize("TruncateHeaders"))
                .on_hover_localized("TruncateHeaders.hover");
            ui.checkbox(&mut self.truncate_headers, "");
            ui.end_row();

            ui.separator();
            ui.separator();
            ui.end_row();

            // Calculate
            ui.label(ui.localize("CalculateFrom"))
                .on_hover_localized("CalculateFrom.hover");
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
            ComboBox::from_id_salt("1|3")
                .selected_text(ui.localize(self.from.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.from,
                        From::Dag1223,
                        ui.localize(From::Dag1223.text()),
                    )
                    .on_hover_localized(From::Dag1223.hover_text());
                    ui.selectable_value(&mut self.from, From::Mag2, ui.localize(From::Mag2.text()))
                        .on_hover_localized(From::Mag2.hover_text());
                })
                .response
                .on_hover_localized(self.from.hover_text());
            ui.end_row();

            // Unsigned
            ui.label(ui.localize("Unsigned"))
                .on_hover_localized("Unsigned.hover");
            ui.checkbox(&mut self.unsigned, ui.localize("Theoretical"));
            ui.end_row();

            // Normalize
            ui.label(ui.localize("Normalize"))
                .on_hover_localized("Normalize.hover");
            ui.checkbox(
                &mut self.normalize.experimental,
                ui.localize("Normalize-Experimental"),
            )
            .on_hover_localized("Normalize-Experimental.hover");
            ui.end_row();
            ui.label("");
            ui.checkbox(
                &mut self.normalize.theoretical,
                ui.localize("Normalize-Theoretical"),
            )
            .on_hover_localized("Normalize-Theoretical.hover");
            ui.end_row();
            ui.label("");
            let response = ui
                .checkbox(&mut self.weighted, ui.localize("Normalize-Weighted"))
                .on_hover_localized("Normalize-Weighted.hover");
            if self.weighted {
                response.on_hover_ui(|ui| {
                    ui.markdown(r#"$$\frac{S}{\sum{(S \cdot M)}}$$"#);
                });
            } else {
                response.on_hover_ui(|ui| {
                    ui.markdown(r#"$$\frac{S}{\sum{S}}$$"#);
                });
            }
            ui.end_row();

            // Christie
            let mut response = ui.label(ui.localize("Normalize-Christie"));
            ui.horizontal(|ui| {
                response |= ui.checkbox(&mut self.christie, "");
                ui.toggle_value(
                    &mut state.windows.open_christie,
                    RichText::new(BROWSERS).heading(),
                );
                response.on_hover_localized("Normalize-Christie.hover");
            });
            ui.end_row();

            ui.separator();
            ui.labeled_separator(RichText::new(ui.localize("Show")).heading());
            ui.end_row();

            // Factors
            ui.label(ui.localize("Show-Factors"))
                .on_hover_localized("Show-Factors.hover");
            ui.checkbox(&mut self.factors, "");
            ui.end_row();

            // Theoretical
            ui.label(ui.localize("Show-Theoretical"))
                .on_hover_localized("Show-Theoretical.hover");
            ui.checkbox(&mut self.theoretical, "");
            ui.end_row();

            if self.index.is_none() {
                ui.separator();
                ui.labeled_separator(RichText::new(ui.localize("Statistics")).heading());
                ui.end_row();

                // ui.label(ui.localize("Merge"));
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
                ui.label(ui.localize("DeltaDegreesOfFreedom.abbreviation"))
                    .on_hover_localized("DeltaDegreesOfFreedom")
                    .on_hover_localized("DeltaDegreesOfFreedom.hover");
                ui.add(Slider::new(&mut self.ddof, 0..=2));
                ui.end_row();
            }
        });
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
            Self::Dag1223 => "CalculateFrom-Sn12Sn23",
            Self::Mag2 => "CalculateFrom-Sn2",
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::Dag1223 => "CalculateFrom-Sn12Sn23.hover",
            Self::Mag2 => "CalculateFrom-Sn2.hover",
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
