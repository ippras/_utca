use egui::{Context, Id, Response, RichText, Tooltip, Ui};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, GEAR, PENCIL, SIGMA, SLIDERS_HORIZONTAL,
};

/// Extension methods for [`Ui`]
pub trait UiExt {
    /// Edit
    fn edit(&mut self, selected: &mut bool);

    /// Indices
    fn indices(&mut self, selected: &mut bool);

    /// Parameters
    fn parameters(&mut self, selected: &mut bool);

    /// Reset
    fn reset(&mut self, selected: &mut bool);

    /// Resize
    fn resize(&mut self, selected: &mut bool);

    /// Settings
    fn settings(&mut self, selected: &mut bool);
}

impl UiExt for Ui {
    fn edit(&mut self, selected: &mut bool) {
        self.toggle_value(selected, RichText::new(PENCIL).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Edit"));
            });
    }

    fn indices(&mut self, selected: &mut bool) {
        self.toggle_value(selected, RichText::new(SIGMA).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Index?PluralCategory=other"));
            });
    }

    fn parameters(&mut self, selected: &mut bool) {
        self.toggle_value(selected, RichText::new(GEAR).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Parameters"));
            });
    }

    fn reset(&mut self, selected: &mut bool) {
        self.toggle_value(selected, RichText::new(ARROWS_CLOCKWISE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("ResetTable"));
            });
    }

    fn resize(&mut self, selected: &mut bool) {
        self.toggle_value(selected, RichText::new(ARROWS_HORIZONTAL).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("ResizeTable"));
            });
    }

    fn settings(&mut self, selected: &mut bool) {
        self.toggle_value(selected, RichText::new(SLIDERS_HORIZONTAL).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Settings"));
            });
    }
}

/// Extension methods for [`Response`]
pub trait ResponseExt: Sized {
    fn try_on_hover_ui<E>(
        self,
        add_contents: impl Fn(&mut Ui) -> Result<(), E>,
    ) -> Result<Self, E> {
        Ok(self
            .try_on_disabled_hover_ui(&add_contents)?
            .try_on_enabled_hover_ui(&add_contents)?)
    }

    fn try_on_enabled_hover_ui<E>(
        self,
        add_contents: impl FnOnce(&mut Ui) -> Result<(), E>,
    ) -> Result<Self, E>;

    fn try_on_disabled_hover_ui<E>(
        self,
        add_contents: impl FnOnce(&mut Ui) -> Result<(), E>,
    ) -> Result<Self, E>;
}

impl ResponseExt for Response {
    fn try_on_enabled_hover_ui<E>(
        self,
        add_contents: impl FnOnce(&mut Ui) -> Result<(), E>,
    ) -> Result<Self, E> {
        if let Some(inner_response) = Tooltip::for_enabled(&self).show(add_contents) {
            inner_response.inner?;
        }
        Ok(self)
    }

    fn try_on_disabled_hover_ui<E>(
        self,
        add_contents: impl FnOnce(&mut Ui) -> Result<(), E>,
    ) -> Result<Self, E> {
        if let Some(inner_response) = Tooltip::for_disabled(&self).show(add_contents) {
            inner_response.inner?;
        }
        Ok(self)
    }
}

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
