use self::windows::Windows;
use crate::app::MAX_PRECISION;
use egui::{
    ComboBox, Context, Grid, Id, Key, Popup, PopupCloseBehavior, RichText, Slider, Ui, Widget,
    emath::Float as _,
};
use egui_dnd::dnd;
use egui_ext::LabeledSeparator;
#[cfg(feature = "markdown")]
use egui_ext::Markdown;
use egui_l20n::UiExt as _;
use egui_phosphor::regular::DOTS_SIX_VERTICAL;
use serde::{Deserialize, Serialize};
use std::{
    hash::{Hash, Hasher},
    ops::{Deref, DerefMut},
};

pub(crate) const ID_SOURCE: &str = "Calculation";

const STEREOSPECIFIC_NUMBERS: [StereospecificNumbers; 4] = [
    StereospecificNumbers::Sn1,
    StereospecificNumbers::Sn2,
    StereospecificNumbers::Sn3,
    StereospecificNumbers::Sn123,
];

/// State
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) reset_table_state: bool,
    pub(crate) settings: Settings,
    pub(crate) windows: Windows,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            reset_table_state: false,
            settings: Settings::new(),
            windows: Windows::new(),
        }
    }
}

impl State {
    pub(crate) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| {
            data.get_persisted_mut_or_insert_with(id, || Self::new())
                .clone()
        })
    }

    pub(crate) fn remove(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.remove::<Self>(id);
        });
    }

    pub(crate) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.insert_persisted(id, self);
        });
    }
}

/// Settings
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct Settings {
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    #[serde(skip)]
    pub(crate) resizable: bool,
    pub(crate) truncate: bool,
    // Table settings
    #[serde(skip)]
    pub(crate) editable: bool,
    pub(crate) sticky: usize,
    // Metrics settings
    pub(crate) chaddock: bool,

    pub(crate) parameters: Parameters,
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            percent: true,
            precision: 2,
            resizable: false,
            truncate: true,

            editable: false,
            sticky: 0,

            chaddock: true,

            parameters: Parameters::new(),
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Settings");
        Grid::new(id_salt).show(ui, |ui| {
            // Precision
            ui.label(ui.localize("Precision")).on_hover_ui(|ui| {
                ui.label(ui.localize("Precision.hover"));
            });
            Slider::new(&mut self.precision, 1..=MAX_PRECISION).ui(ui);
            ui.end_row();

            // Percent
            let mut response = ui.label(ui.localize("Percent"));
            response |= ui.checkbox(&mut self.percent, "");
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("Percent.hover"));
            });
            ui.end_row();

            // Truncate
            let mut response = ui.label(ui.localize("Truncate"));
            response |= ui.checkbox(&mut self.truncate, "");
            response.on_hover_ui(|ui| {
                ui.label(ui.localize("Truncate.hover"));
            });
            ui.end_row();

            ui.separator();
            ui.labeled_separator(ui.localize("Parameters"));
            ui.end_row();

            // Display
            ui.label(ui.localize("Display")).on_hover_ui(|ui| {
                ui.label(ui.localize("Display.hover"));
            });
            ComboBox::from_id_salt(ui.auto_id_with(id_salt))
                .selected_text(ui.localize(self.parameters.display.text()))
                .show_ui(ui, |ui| {
                    ui.selectable_value(
                        &mut self.parameters.display,
                        Display::StereospecificNumbers,
                        ui.localize(Display::StereospecificNumbers.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Display::StereospecificNumbers.hover_text()));
                    });
                    ui.selectable_value(
                        &mut self.parameters.display,
                        Display::Indices,
                        ui.localize(Display::Indices.text()),
                    )
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize(Display::Indices.hover_text()));
                    });
                })
                .response
                .on_hover_ui(|ui| {
                    ui.label(ui.localize(self.parameters.display.hover_text()));
                });
            ui.end_row();

            // Stereospecific numbers
            ui.label(ui.localize("StereospecificNumber?number=many"))
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("StereospecificNumber.abbreviation?number=other"));
                });
            ComboBox::from_id_salt(ui.auto_id_with(id_salt))
                .selected_text(ui.localize(self.parameters.stereospecific_numbers.text()))
                .show_ui(ui, |ui| {
                    for stereospecific_number in STEREOSPECIFIC_NUMBERS {
                        ui.selectable_value(
                            &mut self.parameters.stereospecific_numbers,
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
                    ui.label(ui.localize(self.parameters.stereospecific_numbers.hover_text()));
                });
            ui.end_row();

            // Threshold
            ui.label(ui.localize("Threshold")).on_hover_ui(|ui| {
                ui.label(ui.localize("Threshold.hover"));
            });
            let number_formatter = ui.style().number_formatter.clone();
            let mut threshold = self.parameters.threshold;
            let response = Slider::new(&mut threshold, 0.0..=1.0)
                .custom_formatter(|mut value, decimals| {
                    if self.percent {
                        value *= 100.0;
                    }
                    number_formatter.format(value, decimals)
                })
                .custom_parser(|value| {
                    let mut value = value.parse().ok()?;
                    if self.percent {
                        value /= 100.0;
                    }
                    Some(value)
                })
                .logarithmic(true)
                .update_while_editing(false)
                .ui(ui);
            if (response.drag_stopped() || response.lost_focus())
                && !ui.input(|input| input.key_pressed(Key::Escape))
            {
                self.parameters.threshold = threshold;
            }
            ui.end_row();

            // // Sort
            // ui.label(ui.localize("Sort")).on_hover_ui(|ui| {
            //     ui.label(ui.localize("Sort.hover"));
            // });
            // ComboBox::from_id_salt(ui.auto_id_with(id_salt))
            //     .selected_text(ui.localize(self.parameters.sort.text()))
            //     .show_ui(ui, |ui| {
            //         ui.selectable_value(
            //             &mut self.parameters.sort,
            //             Sort::Key,
            //             ui.localize(Sort::Key.text()),
            //         )
            //         .on_hover_text(ui.localize(Sort::Key.hover_text()));
            //         ui.selectable_value(
            //             &mut self.parameters.sort,
            //             Sort::Value,
            //             ui.localize(Sort::Value.text()),
            //         )
            //         .on_hover_text(ui.localize(Sort::Value.hover_text()));
            //     })
            //     .response
            //     .on_hover_text(ui.localize(self.parameters.sort.hover_text()));
            // ui.end_row();

            // ui.separator();
            // ui.labeled_separator(ui.localize("Metric?PluralCategory=other"));
            // ui.end_row();

            // // Metric
            // ui.label(ui.localize("Metric?PluralCategory=one"))
            //     .on_hover_text(ui.localize("Metric.hover"));
            // #[allow(unused_variables)]
            // let response = ComboBox::from_id_salt(ui.auto_id_with(id_salt))
            //     .selected_text(ui.localize(self.parameters.metric.text()))
            //     .show_ui(ui, |ui| {
            //         for (index, metric) in METRICS.into_iter().enumerate() {
            //             if SEPARATORS.contains(&index) {
            //                 ui.separator();
            //             }
            //             #[allow(unused_variables)]
            //             let response = ui.selectable_value(
            //                 &mut self.parameters.metric,
            //                 metric,
            //                 ui.localize(metric.text()),
            //             );
            //             #[cfg(feature = "markdown")]
            //             response.on_hover_ui(|ui| {
            //                 ui.markdown(metric.hover_markdown());
            //             });
            //         }
            //     })
            //     .response;
            // #[cfg(feature = "markdown")]
            // response.on_hover_ui(|ui| {
            //     ui.markdown(self.parameters.metric.hover_markdown());
            // });
            // ui.end_row();

            // // Chaddock
            // let mut response = ui.label(ui.localize("Chaddock"));
            // response |= ui.checkbox(&mut self.chaddock, "");
            // response.on_hover_ui(|ui| {
            //     ui.label(ui.localize("Chaddock.hover"));
            // });
            // ui.end_row();

            // ui.separator();
            // ui.labeled_separator(ui.localize("Indices"));
            // ui.end_row();

            // // Indices
            // ui.label(ui.localize("Indices")).on_hover_ui(|ui| {
            //     ui.label(ui.localize("Indices.hover"));
            // });
            // let selected_text = format_list_truncated!(
            //     self.parameters
            //         .indices
            //         .0
            //         .iter()
            //         .filter(|index| index.visible)
            //         .map(|index| ui.localize(&format!("Indices_{}", index.name))),
            //     1
            // );
            // ComboBox::from_id_salt(ui.auto_id_with(id_salt))
            //     .selected_text(selected_text)
            //     .close_behavior(PopupCloseBehavior::CloseOnClickOutside)
            //     .show_ui(ui, |ui| self.parameters.indices.show(ui));
            // ui.end_row();
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

/// Fatty acids parameters
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub(crate) struct Parameters {
    pub(crate) display: Display,
    pub(crate) stereospecific_numbers: StereospecificNumbers,
    pub(crate) threshold: f64,
    pub(crate) indices: Indices,
}

impl Parameters {
    pub(crate) fn new() -> Self {
        Self {
            display: Display::StereospecificNumbers,
            stereospecific_numbers: StereospecificNumbers::Sn123,
            threshold: 0.0,
            indices: Indices::new(),
        }
    }
}

impl Default for Parameters {
    fn default() -> Self {
        Self::new()
    }
}

impl Hash for Parameters {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.display.hash(state);
        self.threshold.ord().hash(state);
    }
}

/// Display parameter
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum Display {
    StereospecificNumbers,
    Indices,
}

impl Display {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::StereospecificNumbers => "StereospecificNumber?number=many",
            Self::Indices => "Indices",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self {
            Self::StereospecificNumbers => "StereospecificNumber.abbreviation?number=other",
            Self::Indices => "Indices.hover",
        }
    }
}

/// Stereospecific numbers
#[derive(Clone, Copy, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) enum StereospecificNumbers {
    Sn123,
    Sn1,
    Sn2,
    Sn3,
}

impl StereospecificNumbers {
    pub(crate) fn text(&self) -> &'static str {
        match self {
            Self::Sn123 => "StereospecificNumber.abbreviation?number=123",
            Self::Sn1 => "StereospecificNumber.abbreviation?number=1",
            Self::Sn2 => "StereospecificNumber.abbreviation?number=2",
            Self::Sn3 => "StereospecificNumber.abbreviation?number=3",
        }
    }

    pub(crate) fn hover_text(&self) -> &'static str {
        match self {
            Self::Sn123 => "StereospecificNumber?number=123",
            Self::Sn1 => "StereospecificNumber?number=1",
            Self::Sn2 => "StereospecificNumber?number=2",
            Self::Sn3 => "StereospecificNumber?number=3",
        }
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
        ])
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
                    let mut label = RichText::new(&index.name);
                    if !visible {
                        label = label.weak();
                    }
                    let response = ui.label(label);
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

mod windows;
