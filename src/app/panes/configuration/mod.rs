use self::{settings::Settings, state::State, table::TableView};
use super::PaneDelegate;
use crate::{app::identifiers::CALCULATE, export::parquet::save};
use anyhow::Result;
use egui::{CursorIcon, Id, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CALCULATOR, ERASER, FLOPPY_DISK, GEAR, LIST, NOTE_PENCIL,
    PENCIL, TAG, TRASH,
};
use lipid::prelude::*;
use metadata::{MetaDataFrame, egui::MetadataWidget};
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tracing::instrument;

const ID_SOURCE: &str = "Configuration";

pub(crate) static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::from_iter([
        Field::new("Label".into(), DataType::String),
        field!(FATTY_ACID),
        Field::new("Triacylglycerol".into(), DataType::Float64),
        Field::new("Diacylglycerol1223".into(), DataType::Float64),
        Field::new("Monoacylglycerol2".into(), DataType::Float64),
    ])
});

/// Configuration pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    frames: Vec<MetaDataFrame>,
    settings: Settings,
    state: State,
}

impl Pane {
    pub(crate) fn new(frames: Vec<MetaDataFrame>) -> Self {
        Self {
            frames,
            settings: Settings::new(),
            state: State::new(),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        NOTE_PENCIL
    }

    pub(crate) fn title(&self) -> String {
        self.title_with_separator(" ")
    }

    fn title_with_separator(&self, separator: &str) -> String {
        self.frames[self.settings.index]
            .meta
            .format(separator)
            .to_string()
    }

    fn hash(&self) -> u64 {
        hash(&self.frames)
    }
}

impl Pane {
    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui.heading(Self::icon()).on_hover_ui(|ui| {
            ui.label(ui.localize("Configuration"));
        });
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.hash()))
            .on_hover_ui(|ui| MetadataWidget::new(&self.frames[self.settings.index].meta).show(ui))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            for index in 0..self.frames.len() {
                if ui
                    .selectable_value(
                        &mut self.settings.index,
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
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("ResetTable"));
            })
            .clicked()
        {
            self.state.reset_table_state = true;
        }
        // Resize
        ui.toggle_value(
            &mut self.settings.resizable,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("ResizeTable"));
        });
        // Edit
        ui.toggle_value(&mut self.settings.editable, RichText::new(PENCIL).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Edit"));
            });
        // Clear
        ui.add_enabled_ui(
            self.settings.editable && self.frames[self.settings.index].data.height() > 0,
            |ui| {
                if ui
                    .button(RichText::new(ERASER).heading())
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("ClearTable"));
                    })
                    .clicked()
                {
                    let data_frame = &mut self.frames[self.settings.index].data;
                    *data_frame = data_frame.clear();
                }
            },
        );
        // Delete
        ui.add_enabled_ui(self.settings.editable && self.frames.len() > 1, |ui| {
            if ui
                .button(RichText::new(TRASH).heading())
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("DeleteTable"));
                })
                .clicked()
            {
                self.frames.remove(self.settings.index);
                self.settings.index = 0;
            }
        });
        ui.separator();
        // Settings
        ui.toggle_value(
            &mut self.state.open_settings_window,
            RichText::new(GEAR).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("Settings"));
        });
        ui.separator();
        // Save
        if ui
            .button(RichText::new(FLOPPY_DISK).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Save"));
            })
            .on_hover_text(format!("{}.utca.parquet", self.title_with_separator(".")))
            .clicked()
        {
            let _ = self.save();
        }
        // if ui
        //     .button(RichText::new("JSON").heading())
        //     .on_hover_ui(|ui| {
        //         ui.label(ui.localize("Save"));
        //     })
        //     .on_hover_text(format!("{}.utca.json", self.title()))
        //     .clicked()
        // {
        //     let mut file = std::fs::File::create(format!("{}.utca.json", self.title())).unwrap();
        //     JsonWriter::new(&mut file)
        //         .with_json_format(JsonFormat::JsonLines)
        //         .finish(&mut self.frames[self.settings.index].data)
        //         .unwrap();
        // }
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
                data.insert_temp(
                    Id::new(CALCULATE),
                    (self.frames.clone(), self.settings.index),
                );
            });
        }
        ui.separator();
        response
    }

    fn body_content_meta(&mut self, ui: &mut Ui, index: usize) {
        ui.style_mut().visuals.collapsing_header_frame = true;
        ui.collapsing(RichText::new(format!("{TAG} Metadata")).heading(), |ui| {
            MetadataWidget::new(&mut self.frames[index].meta)
                .with_writable(true)
                .show(ui);
        });
    }

    fn body_content_data(&mut self, ui: &mut Ui, index: usize) {
        let data_frame = &mut self.frames[index].data;
        TableView::new(data_frame, &self.settings, &mut self.state).show(ui);
    }

    #[instrument(skip(self), err)]
    fn save(&mut self) -> Result<()> {
        let name = format!("{}.utca.parquet", self.title_with_separator("."));
        save(&mut self.frames[self.settings.index], &name)?;
        Ok(())
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui) {
        self.settings(ui);
    }

    fn settings(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Configuration settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .default_pos(ui.next_widget_position())
            .open(&mut self.state.open_settings_window)
            .show(ui.ctx(), |ui| self.settings.show(ui));
    }
}

impl PaneDelegate for Pane {
    fn header(&mut self, ui: &mut Ui) -> Response {
        self.header_content(ui)
    }

    fn body(&mut self, ui: &mut Ui) {
        if self.settings.editable {
            self.body_content_meta(ui, self.settings.index);
        }
        self.body_content_data(ui, self.settings.index);
        self.windows(ui);
    }
}

pub(crate) mod settings;

mod state;
mod table;
