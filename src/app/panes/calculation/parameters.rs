use super::ID_SOURCE;
use crate::app::panes::calculation::state::Windows;
use egui::{ComboBox, Grid, Key, KeyboardShortcut, Modifiers, RichText, Slider, Ui};
use egui_ext::LabeledSeparator;
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_l20n::{ResponseExt, UiExt as _};
use egui_phosphor::regular::BROWSERS;
use serde::{Deserialize, Serialize};

/// Calculation parameters
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Parameters {
    pub(crate) index: Option<usize>,

    pub(crate) weighted: bool,
    pub(crate) from: From,
    pub(crate) normalize: Normalize,
    pub(crate) unsigned: bool,
    pub(crate) christie: bool,
    pub(crate) ddof: u8,
}

impl Parameters {
    pub(crate) fn new(index: Option<usize>) -> Self {
        Self {
            index,
            weighted: false,
            from: From::StereospecificNumbers2,
            normalize: Normalize::new(),
            unsigned: true,
            christie: false,
            ddof: 1,
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::new(None)
    }
}

impl Parameters {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        Grid::new(ID_SOURCE).show(ui, |ui| {
            // Calculate
            ui.label(ui.localize("CalculateFrom"))
                .on_hover_localized("CalculateFrom.hover");
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::Num1))
            }) {
                self.from = From::StereospecificNumbers12_23;
            }
            if ui.input_mut(|input| {
                input.consume_shortcut(&KeyboardShortcut::new(Modifiers::CTRL, Key::Num2))
            }) {
                self.from = From::StereospecificNumbers2;
            }
            ComboBox::from_id_salt("1|3")
                .selected_text(ui.localize(self.from.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.from,
                        From::StereospecificNumbers12_23,
                        ui.localize(From::StereospecificNumbers12_23.text()),
                    )
                    .on_hover_localized(From::StereospecificNumbers12_23.hover_text());
                    ui.selectable_value(
                        &mut self.from,
                        From::StereospecificNumbers2,
                        ui.localize(From::StereospecificNumbers2.text()),
                    )
                    .on_hover_localized(From::StereospecificNumbers2.hover_text());
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
                #[cfg(feature = "markdown")]
                response.on_hover_ui(|ui| {
                    ui.markdown(r#"$$\frac{S}{\sum{(S \cdot M)}}$$"#);
                });
            } else {
                #[cfg(feature = "markdown")]
                response.on_hover_ui(|ui| {
                    ui.markdown(r#"$$\frac{S}{\sum{S}}$$"#);
                });
            }
            ui.end_row();

            // Christie
            let mut response = ui.label(ui.localize("Normalize-Christie"));
            ui.horizontal(|ui| {
                response |= ui.checkbox(&mut self.christie, "");
                let mut windows = Windows::load(ui.ctx());
                ui.toggle_value(
                    &mut windows.open_christie,
                    RichText::new(BROWSERS).heading(),
                );
                windows.store(ui.ctx());
                response.on_hover_localized("Normalize-Christie.hover");
            });
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
    StereospecificNumbers12_23,
    StereospecificNumbers2,
}

impl From {
    pub(crate) fn text(self) -> &'static str {
        match self {
            Self::StereospecificNumbers12_23 => "CalculateFrom-Sn12Sn23",
            Self::StereospecificNumbers2 => "CalculateFrom-Sn2",
        }
    }

    pub(crate) fn hover_text(self) -> &'static str {
        match self {
            Self::StereospecificNumbers12_23 => "CalculateFrom-Sn12Sn23.hover",
            Self::StereospecificNumbers2 => "CalculateFrom-Sn2.hover",
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
