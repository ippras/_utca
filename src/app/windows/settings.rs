use crate::{app::identifiers::GITHUB_TOKEN, utils::egui::State};
use constcat::concat;
use egui::{Button, Context, Id, Response, TextEdit, Ui, Window};
use egui_phosphor::regular::{EYE, GEAR};
use serde::{Deserialize, Serialize};

const ID_SOURCE: &str = "Settings";

/// Settings window
#[derive(Clone, Debug, Deserialize, Hash, PartialEq, Serialize)]
pub(crate) struct SettingsWindow {
    pub(crate) open: bool,
}

impl SettingsWindow {
    pub(crate) fn new() -> Self {
        Self { open: false }
    }

    const fn title(&self) -> &'static str {
        concat!(GEAR, " Settings")
    }

    pub(crate) fn show(&mut self, ctx: &Context) {
        Window::new(self.title())
            .open(&mut self.open)
            .resizable([true, false])
            .show(ctx, ui);
    }
}

impl Default for SettingsWindow {
    fn default() -> Self {
        Self::new()
    }
}

fn ui(ui: &mut Ui) -> Response {
    let id_salt = Id::new(ID_SOURCE);
    let id = ui.make_persistent_id(id_salt);
    let token_id = Id::new(GITHUB_TOKEN);
    // Load
    let mut state = SettingsState::load(ui.ctx(), id);
    let mut token = ui.data_mut(|data| data.get_persisted::<String>(token_id).unwrap_or_default());
    let response = ui
        .horizontal(|ui| {
            ui.label("Token");
            ui.add(TextEdit::singleline(&mut token).password(!state.password));
            let response = ui
                .add(Button::selectable(state.password, EYE))
                .on_hover_text("Show/hide token");
            if response.clicked() {
                state.password = !state.password;
            }
            response
        })
        .inner;
    // Store
    ui.data_mut(|data| data.insert_persisted(token_id, token));
    state.store(ui.ctx(), id);
    response
}

/// Settings state
#[derive(Clone, Default, Deserialize, Serialize)]
pub(crate) struct SettingsState {
    pub(crate) token: String,
    pub(crate) password: bool,
}

impl State for SettingsState {
    fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| data.get_persisted_mut_or_default::<Self>(id).clone())
    }

    fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| data.insert_persisted(id, self));
    }

    fn reset(ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.remove::<Self>(id);
        });
    }
}

// pub fn password(password: &mut String) -> impl Widget {
//     move |ui: &mut Ui| {
//         let id_salt = Id::new(ID_SOURCE).with("Token");
//         let id = ui.make_persistent_id(id_salt);
//         let mut show = ui.data_mut(|data| data.get_temp::<bool>(id).unwrap_or(false));
//         let response = ui
//             .with_layout(Layout::right_to_left(Align::Center), |ui| {
//                 let response = ui
//                     .add(SelectableLabel::new(show, "👁"))
//                     .on_hover_text("Show/hide token");
//                 if response.clicked() {
//                     show = !show;
//                 }
//                 ui.add_sized(
//                     ui.available_size(),
//                     TextEdit::singleline(password).password(!show),
//                 );
//             })
//             .response;
//         ui.data_mut(|data| data.insert_temp(id, show));
//         response
//     }
// }

// pub fn password(password: &mut String) -> impl Widget {
//     move |ui: &mut Ui| {
//         let id_salt = Id::new(ID_SOURCE).with("Token");
//         let id = ui.make_persistent_id(id_salt);
//         let mut show = ui.data_mut(|data| data.get_temp::<bool>(id).unwrap_or(false));
//         let response = ui
//             .with_layout(Layout::right_to_left(Align::Center), |ui| {
//                 let response = ui
//                     .add(SelectableLabel::new(show, "👁"))
//                     .on_hover_text("Show/hide token");
//                 if response.clicked() {
//                     show = !show;
//                 }
//                 ui.add_sized(
//                     ui.available_size(),
//                     TextEdit::singleline(password).password(!show),
//                 );
//             })
//             .response;
//         ui.data_mut(|data| data.insert_temp(id, show));
//         response
//     }
// }
