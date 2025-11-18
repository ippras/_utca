use self::table::TableView;
use super::{Behavior, MARGIN};
use crate::{
    app::{identifiers::CALCULATE, states::configuration::State},
    export,
    utils::{HashedDataFrame, HashedMetaDataFrame, egui::UiExt as _},
};
use egui::{
    CentralPanel, CursorIcon, Frame, Id, MenuBar, Response, RichText, ScrollArea, TextStyle,
    TextWrapMode, TopBottomPanel, Ui, Window, util::hash,
};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    CALCULATOR, ERASER, FLOPPY_DISK, LIST, NOTE_PENCIL, SLIDERS_HORIZONTAL, TAG, TRASH, X,
};
use egui_tiles::{TileId, UiResponse};
use lipid::prelude::*;
use metadata::egui::MetadataWidget;
use polars::prelude::*;
use polars_utils::format_list_truncated;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{Display, from_fn},
    sync::LazyLock,
};

const ID_SOURCE: &str = "Configuration";

pub(crate) static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::from_iter([
        Field::new("Label".into(), DataType::String),
        field!(FATTY_ACID),
        Field::new(STEREOSPECIFIC_NUMBERS123.into(), DataType::Float64),
        Field::new(STEREOSPECIFIC_NUMBERS2.into(), DataType::Float64),
    ])
});

/// Configuration pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    id: Option<Id>,
    frames: Vec<HashedMetaDataFrame>,
}

impl Pane {
    pub(crate) fn new(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self { id: None, frames }
    }

    pub(crate) fn title(&self, index: Option<usize>) -> String {
        self.title_with_separator(index, " ")
    }

    fn title_with_separator(&self, index: Option<usize>, separator: &str) -> String {
        match index {
            Some(index) => self.frames[index].meta.format(separator).to_string(),
            None => {
                format_list_truncated!(
                    self.frames.iter().map(|frame| frame.meta.format(separator)),
                    2
                )
            }
        }
    }

    fn id(&self) -> impl Display {
        from_fn(|f| {
            if let Some(id) = self.id {
                write!(f, "{id:?}-")?;
            }
            write!(f, "{}", hash(&self.frames))
        })
    }
}

impl Pane {
    pub(super) fn ui(
        &mut self,
        ui: &mut Ui,
        behavior: &mut Behavior,
        tile_id: TileId,
    ) -> UiResponse {
        let id = *self.id.get_or_insert_with(|| ui.next_auto_id());
        let mut state = State::load(ui.ctx(), id);
        let response = TopBottomPanel::top(ui.auto_id_with("Pane"))
            .show_inside(ui, |ui| {
                MenuBar::new()
                    .ui(ui, |ui| {
                        ScrollArea::horizontal()
                            .show(ui, |ui| {
                                ui.set_height(
                                    ui.text_style_height(&TextStyle::Heading) + 4.0 * MARGIN.y,
                                );
                                ui.visuals_mut().button_frame = false;
                                if ui.button(RichText::new(X).heading()).clicked() {
                                    behavior.close = Some(tile_id);
                                }
                                ui.separator();
                                self.top(ui, &mut state)
                            })
                            .inner
                    })
                    .inner
            })
            .inner;
        CentralPanel::default()
            .frame(Frame::central_panel(&ui.style()))
            .show_inside(ui, |ui| {
                self.central(ui, &mut state);
                self.windows(ui, &mut state);
            });
        if behavior.close == Some(tile_id) {
            state.remove(ui.ctx(), id);
        } else {
            state.store(ui.ctx(), id);
        }
        if response.dragged() {
            UiResponse::DragStarted
        } else {
            UiResponse::None
        }
    }

    fn top(&mut self, ui: &mut Ui, state: &mut State) -> Response {
        let mut response = ui.heading(NOTE_PENCIL).on_hover_ui(|ui| {
            ui.label(ui.localize("Configuration"));
        });
        response |= ui.heading(self.title(Some(state.settings.index)));
        response = response
            .on_hover_text(self.id().to_string())
            .on_hover_ui(|ui| MetadataWidget::new(&self.frames[state.settings.index].meta).show(ui))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            // ui.set_max_width(ui.spacing().tooltip_width);
            for index in 0..self.frames.len() {
                if ui
                    .selectable_value(
                        &mut state.settings.index,
                        index,
                        self.frames[index].meta.format(" ").to_string(),
                    )
                    .clicked()
                {
                    ui.close();
                }
            }
        })
        .response
        .on_hover_ui(|ui| {
            ui.label(ui.localize("List"));
        });
        ui.separator();
        // Reset
        ui.reset_button(&mut state.reset_table);
        // Resize
        ui.resize_button(&mut state.settings.resize_table);
        // Edit
        ui.edit(&mut state.settings.edit_table);
        // Clear
        ui.add_enabled_ui(
            state.settings.edit_table && self.frames[state.settings.index].data.height() > 0,
            |ui| {
                if ui
                    .button(RichText::new(ERASER).heading())
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("ClearTable"));
                    })
                    .clicked()
                {
                    let data_frame = &mut self.frames[state.settings.index].data;
                    *data_frame = HashedDataFrame::EMPTY;
                }
            },
        );
        // Delete
        ui.add_enabled_ui(state.settings.edit_table && self.frames.len() > 1, |ui| {
            if ui
                .button(RichText::new(TRASH).heading())
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("DeleteTable"));
                })
                .clicked()
            {
                self.frames.remove(state.settings.index);
                state.settings.index = 0;
            }
        });
        ui.separator();
        // Settings
        ui.settings_button(&mut state.windows.open_settings);
        ui.separator();
        // Save
        let name = self.frames[state.settings.index].meta.format(".");
        if ui
            .button(RichText::new(FLOPPY_DISK).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Save"));
            })
            .on_hover_text(format!("{name}"))
            .clicked()
        {
            let _ = export::ron::save(
                &self.frames[state.settings.index],
                &format!("{name}.utca.ron"),
            );
        }
        ui.separator();
        // Calculation
        if ui
            .button(RichText::new(CALCULATOR).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Calculation"));
            })
            .clicked()
        {
            ui.data_mut(|data| {
                data.insert_temp(Id::new(CALCULATE), self.frames.clone());
            });
        }
        ui.separator();
        response
    }

    fn central(&mut self, ui: &mut Ui, state: &mut State) {
        if state.settings.edit_table {
            self.meta(ui, state);
        }
        self.data(ui, state);
    }

    fn meta(&mut self, ui: &mut Ui, state: &mut State) {
        ui.style_mut().visuals.collapsing_header_frame = true;
        ui.collapsing(RichText::new(format!("{TAG} Metadata")).heading(), |ui| {
            MetadataWidget::new(&mut self.frames[state.settings.index].meta)
                .with_writable(true)
                .show(ui);
        });
    }

    fn data(&mut self, ui: &mut Ui, state: &mut State) {
        let data_frame = &mut self.frames[state.settings.index].data;
        TableView::new(data_frame, state).show(ui);
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui, state: &mut State) {
        self.settings_window(ui, state);
    }

    fn settings_window(&mut self, ui: &mut Ui, state: &mut State) {
        if let Some(inner_response) =
            Window::new(format!("{SLIDERS_HORIZONTAL} Configuration settings"))
                .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
                .default_pos(ui.next_widget_position())
                .open(&mut state.windows.open_settings)
                .show(ui.ctx(), |ui| {
                    state.settings.show(ui);
                })
        {
            inner_response
                .response
                .on_hover_text(self.title(Some(state.settings.index)).to_string())
                .on_hover_text(self.id().to_string());
        }
    }
}

mod table;
