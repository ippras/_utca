use ahash::HashSet;
use egui::{
    Context, Id, PopupCloseBehavior, RichText, Sides, Ui,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_dnd::dnd;
use egui_phosphor::regular::{DOTS_SIX_VERTICAL, EYE, EYE_SLASH, GEAR, SLIDERS_HORIZONTAL};
use serde::{Deserialize, Serialize};

/// State
pub trait State: Sized {
    fn load(ctx: &Context, id: Id) -> Self;

    fn store(self, ctx: &Context, id: Id);

    fn reset(ctx: &Context, id: Id);
}

// /// Settings undoer
// pub(crate) type SettingsUndoer = Undoer<(String, bool)>;

// /// Settings state
// #[derive(Clone, Default, Deserialize, Serialize)]
// pub(crate) struct SettingsState {
//     /// Wrapped in Arc for cheaper clones.
//     #[serde(skip)]
//     pub(crate) undoer: Arc<Mutex<SettingsUndoer>>,
// }

// impl SettingsState {
//     pub(crate) fn undoer(&self) -> SettingsUndoer {
//         self.undoer.lock().clone()
//     }

//     #[allow(clippy::needless_pass_by_ref_mut)] // Intentionally hide interiority of mutability
//     pub(crate) fn set_undoer(&mut self, undoer: SettingsUndoer) {
//         *self.undoer.lock() = undoer;
//     }

//     pub(crate) fn clear_undoer(&mut self) {
//         self.set_undoer(SettingsUndoer::default());
//     }
// }

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Table {
    id: Id,
    columns: Vec<Column>,
}

impl Table {
    pub fn new(id: Id) -> Self {
        Self {
            id,
            columns: Vec::new(),
        }
    }

    pub fn update(&mut self, columns: &[&str]) {
        let mut has_columns = HashSet::default();
        for &name in columns {
            let id = Id::new(name);
            has_columns.insert(id);
            if !self.columns.iter().any(|column| column.name == name) {
                self.columns.push(Column::new(id, name.to_owned()));
            }
        }
        self.columns
            .retain(|column| has_columns.contains(&column.id));
    }

    pub fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| {
            data.get_persisted_mut_or_insert_with(id, || Self::new(id))
                .clone()
        })
    }

    pub fn store(self, ctx: &Context) {
        ctx.data_mut(|data| {
            data.insert_persisted(self.id, self);
        });
    }

    pub fn reset(self, ctx: &Context) {
        ctx.data_mut(|data| {
            data.insert_persisted(
                self.id,
                Self {
                    id: self.id,
                    columns: Vec::new(),
                },
            );
        });
    }

    pub fn visible_columns(&self) -> impl Iterator<Item = &Column> {
        self.columns.iter().filter(|column| column.visible)
    }

    pub fn visible_column_names(&self) -> impl Iterator<Item = &str> {
        self.visible_columns().map(|column| column.name.as_str())
    }

    pub fn visible_column_ids(&self) -> impl Iterator<Item = Id> + use<'_> {
        self.visible_columns().map(|column| column.id)
    }
}

impl Table {
    pub fn show(&mut self, ui: &mut Ui) {
        self.columns(ui);
    }

    pub fn columns(&mut self, ui: &mut Ui) {
        let response = dnd(ui, self.id.with("Columns")).show(
            self.columns.iter_mut(),
            |ui, item, handle, _state| {
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
            },
        );
        if response.is_drag_finished() {
            response.update_vec(self.columns.as_mut_slice());
        }
    }
}

#[derive(Clone, Debug, Deserialize, Hash, Serialize)]
pub struct Column {
    id: Id,
    name: String,
    visible: bool,
}

impl Column {
    pub fn new(id: Id, name: String) -> Self {
        Self {
            id,
            name,
            visible: true,
        }
    }
}
