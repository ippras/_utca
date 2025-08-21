use super::ID_SOURCE;
use crate::{app::MAX_PRECISION, utils::egui::State};
use egui::{Context, Grid, Id, Slider, Ui};
use egui_l20n::{ResponseExt, UiExt as _};
use serde::{Deserialize, Serialize};

// #[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
// pub(crate) struct State {
//     pub(crate) parameters: Parameters,
//     pub(crate) windows: Windows,
//     pub(crate) reset_table_state: bool,
// }

// impl State {
//     pub(crate) fn new() -> Self {
//         Self {
//             parameters: Parameters::new(),
//             windows: Windows::new(),
//             reset_table_state: false,
//         }
//     }
// }

/// Calculation parameters state
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Parameters {
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) resizable: bool,
    pub(crate) sticky_columns: usize,
    pub(crate) truncate_headers: bool,
}

impl Parameters {
    pub(crate) fn new() -> Self {
        Self {
            percent: true,
            precision: 1,
            resizable: false,
            sticky_columns: 0,
            truncate_headers: false,
        }
    }
}

impl Parameters {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
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
        });
    }
}

impl State for Parameters {
    fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| {
            data.get_persisted_mut_or_insert_with(id, || Parameters::new())
                .clone()
        })
    }

    fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.insert_persisted(id, self);
        });
    }

    fn reset(ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.insert_persisted(id, Parameters::new());
        })
    }
}

/// Calculation windows state
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Windows {
    pub(crate) open_christie: bool,
    pub(crate) open_indices: bool,
    pub(crate) open_parameters: bool,
    pub(crate) open_settings: bool,
}

impl Windows {
    pub(crate) fn new() -> Self {
        Self {
            open_christie: false,
            open_indices: false,
            open_parameters: false,
            open_settings: false,
        }
    }
}
