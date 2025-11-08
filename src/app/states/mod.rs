use self::windows::Windows;
use egui::{ComboBox, Context, Grid, Id, Sense, Ui};
use egui_l20n::UiExt as _;
use serde::{Deserialize, Serialize};

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
}

impl Settings {
    pub(crate) fn new() -> Self {
        Self {
            percent: true,
            precision: 2,
        }
    }
}

impl Settings {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new("Settings");
        Grid::new(id_salt).show(ui, |ui| {
            // Language
            ui.label(ui.localize("Language"));
            let mut current_value = ui.language_identifier();
            ComboBox::from_id_salt(id_salt.with("Language"))
                .selected_text(current_value.to_string())
                .show_ui(ui, |ui| {
                    let mut response = ui.allocate_response(Default::default(), Sense::click());
                    for selected_value in ui.language_identifiers() {
                        let text = selected_value.to_string();
                        response |= ui.selectable_value(&mut current_value, selected_value, text);
                    }
                    if response.changed() {
                        ui.set_language_identifier(current_value);
                    }
                });
            ui.end_row();
        });
    }
}

impl Default for Settings {
    fn default() -> Self {
        Self::new()
    }
}

pub mod calculation;

mod windows;
