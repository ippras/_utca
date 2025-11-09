pub(crate) use self::windows::Windows;

use ahash::HashSet;
use egui::{ComboBox, Context, Grid, Id, RichText, Sense, Sides, Ui};
use egui_dnd::dnd;
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{DOTS_SIX_VERTICAL, EYE, EYE_SLASH};
use serde::{Deserialize, Serialize};

/// State
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) settings: Settings,
    pub(crate) windows: Windows,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
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

/// Column filter
#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct ColumnFilter {
    pub(crate) columns: Vec<Column>,
}

impl ColumnFilter {
    pub(crate) fn new() -> Self {
        Self {
            columns: Vec::new(),
        }
    }

    pub(crate) fn update(&mut self, columns: &[&str]) {
        let mut has_columns = HashSet::default();
        for &name in columns {
            has_columns.insert(name);
            if !self.columns.iter().any(|column| column.name == name) {
                self.columns.push(Column::new(name.to_owned()));
            }
        }
        self.columns
            .retain(|column| has_columns.contains(&*column.name));
    }

    pub(crate) fn visible_columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter().filter(|column| column.visible)
    }

    pub(crate) fn visible_column_names(&self) -> impl Iterator<Item = &str> {
        self.visible_columns().map(|column| column.name.as_str())
    }
}

impl ColumnFilter {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        self.columns(ui);
    }

    pub(crate) fn columns(&mut self, ui: &mut Ui) {
        let response =
            dnd(ui, ui.next_auto_id()).show(self.columns.iter_mut(), |ui, item, handle, _state| {
                let visible = item.visible;
                Sides::new().show(
                    ui,
                    |ui| {
                        handle.ui(ui, |ui| {
                            ui.label(DOTS_SIX_VERTICAL);
                        });
                        let mut label = RichText::new(&item.name);
                        if !visible {
                            label = label.weak();
                        }
                        ui.label(label);
                    },
                    |ui| {
                        if ui
                            .small_button(if item.visible { EYE } else { EYE_SLASH })
                            .clicked()
                        {
                            item.visible = !item.visible;
                        }
                    },
                );
            });
        if response.is_drag_finished() {
            response.update_vec(self.columns.as_mut_slice());
        }
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub(crate) struct Column {
    name: String,
    visible: bool,
}

impl Column {
    pub(crate) fn new(name: String) -> Self {
        Self {
            name,
            visible: true,
        }
    }
}

pub(crate) mod calculation;
pub(crate) mod composition;
pub(crate) mod configuration;

mod windows;
