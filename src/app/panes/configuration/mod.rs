use self::{
    parameters::Parameters,
    state::{Settings, Windows},
    table::TableView,
};
use super::PaneDelegate;
use crate::{app::identifiers::CALCULATE, export::parquet::save};
use anyhow::Result;
use egui::{CursorIcon, Id, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CALCULATOR, ERASER, FLOPPY_DISK, GEAR, LIST, NOTE_PENCIL,
    PENCIL, SLIDERS_HORIZONTAL, TAG, TRASH,
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
    parameters: Parameters,
}

impl Pane {
    pub(crate) fn new(frames: Vec<MetaDataFrame>) -> Self {
        Self {
            frames,
            parameters: Parameters::new(),
        }
    }

    pub(crate) const fn icon() -> &'static str {
        NOTE_PENCIL
    }

    pub(crate) fn title(&self) -> String {
        self.title_with_separator(" ")
    }

    fn title_with_separator(&self, separator: &str) -> String {
        self.frames[self.parameters.index]
            .meta
            .format(separator)
            .to_string()
    }

    fn hash(&self) -> u64 {
        hash(&self.frames)
    }
}

impl Pane {
    fn header_content(&mut self, ui: &mut Ui, settings: &mut Settings) -> Response {
        let mut windows = Windows::load(ui.ctx());
        let mut response = ui.heading(Self::icon()).on_hover_ui(|ui| {
            ui.label(ui.localize("Configuration"));
        });
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.hash()))
            .on_hover_ui(|ui| {
                MetadataWidget::new(&self.frames[self.parameters.index].meta).show(ui)
            })
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            for index in 0..self.frames.len() {
                if ui
                    .selectable_value(
                        &mut self.parameters.index,
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
            settings.reset_state = true;
        }
        // Resize
        ui.toggle_value(
            &mut settings.resize_table,
            RichText::new(ARROWS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("ResizeTable"));
        });
        // Edit
        ui.toggle_value(&mut settings.edit_table, RichText::new(PENCIL).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Edit"));
            });
        // Clear
        ui.add_enabled_ui(
            settings.edit_table && self.frames[self.parameters.index].data.height() > 0,
            |ui| {
                if ui
                    .button(RichText::new(ERASER).heading())
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("ClearTable"));
                    })
                    .clicked()
                {
                    let data_frame = &mut self.frames[self.parameters.index].data;
                    *data_frame = data_frame.clear();
                }
            },
        );
        // Delete
        ui.add_enabled_ui(settings.edit_table && self.frames.len() > 1, |ui| {
            if ui
                .button(RichText::new(TRASH).heading())
                .on_hover_ui(|ui| {
                    ui.label(ui.localize("DeleteTable"));
                })
                .clicked()
            {
                self.frames.remove(self.parameters.index);
                self.parameters.index = 0;
            }
        });
        ui.separator();
        // // Settings
        // ui.toggle_value(
        //     &mut self.state.open_settings_window,
        //     RichText::new(GEAR).heading(),
        // )
        // .on_hover_ui(|ui| {
        //     ui.label(ui.localize("Settings"));
        // });
        // ui.separator();

        // Settings
        ui.toggle_value(
            &mut windows.open_settings,
            RichText::new(SLIDERS_HORIZONTAL).heading(),
        )
        .on_hover_ui(|ui| {
            ui.label(ui.localize("Settings"));
        });
        ui.separator();
        // Parameters
        ui.toggle_value(&mut windows.open_parameters, RichText::new(GEAR).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("Parameters"));
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
                    (self.frames.clone(), self.parameters.index),
                );
            });
        }
        ui.separator();
        windows.store(ui.ctx());
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

    fn body_content_data(&mut self, ui: &mut Ui, index: usize, settings: &mut Settings) {
        let data_frame = &mut self.frames[index].data;
        TableView::new(data_frame, settings).show(ui);
    }

    #[instrument(skip(self), err)]
    fn save(&mut self) -> Result<()> {
        let name = format!("{}.utca.parquet", self.title_with_separator("."));
        save(&mut self.frames[self.parameters.index], &name)?;
        Ok(())
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui) {
        let windows = &mut Windows::load(ui.ctx());
        self.parameters(ui, windows);
        self.settings(ui, windows);
        windows.store(ui.ctx());
    }

    fn parameters(&mut self, ui: &mut Ui, windows: &mut Windows) {
        Window::new(format!("{GEAR} Configuration parameters"))
            .id(ui.auto_id_with(ID_SOURCE))
            .default_pos(ui.next_widget_position())
            .open(&mut windows.open_parameters)
            .show(ui.ctx(), |ui| self.parameters.show(ui));
    }

    fn settings(&mut self, ui: &mut Ui, windows: &mut Windows) {
        Window::new(format!("{SLIDERS_HORIZONTAL} Configuration settings"))
            .id(ui.auto_id_with(ID_SOURCE).with("Settings"))
            .open(&mut windows.open_settings)
            .show(ui.ctx(), |ui| {
                let mut settings = Settings::load(ui.ctx(), self.hash());
                settings.show(ui);
                settings.store(ui.ctx(), self.hash());
            });
    }

    // fn settings(&mut self, ui: &mut Ui) {
    //     Window::new(format!("{GEAR} Configuration settings"))
    //         .id(ui.auto_id_with(ID_SOURCE))
    //         .default_pos(ui.next_widget_position())
    //         .open(&mut self.state.open_settings_window)
    //         .show(ui.ctx(), |ui| self.settings.show(ui));
    // }
}

impl PaneDelegate for Pane {
    fn header(&mut self, ui: &mut Ui) -> Response {
        let mut settings = Settings::load(ui.ctx(), self.hash());
        settings.filter_columns.update(vec![
            "Index",
            "Label",
            "Fatty acid",
            "SN-1,2,3",
            "SN-1,2(2,3)",
            "SN-2",
        ]);
        let response = self.header_content(ui, &mut settings);
        settings.store(ui.ctx(), self.hash());
        response
    }

    fn body(&mut self, ui: &mut Ui) {
        let mut settings = Settings::load(ui.ctx(), self.hash());
        if settings.edit_table {
            self.body_content_meta(ui, self.parameters.index);
        }
        self.body_content_data(ui, self.parameters.index, &mut settings);
        settings.store(ui.ctx(), self.hash());
        self.windows(ui);
    }
}

pub(crate) mod parameters;

mod state;
mod table;
