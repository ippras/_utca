use egui::{Context, Id};

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

pub mod state;
