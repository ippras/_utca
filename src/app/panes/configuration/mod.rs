use self::{
    state::{Settings, Windows},
    table::TableView,
};
use super::PaneDelegate;
use crate::{
    app::identifiers::CALCULATE,
    export,
    utils::{HashedDataFrame, HashedMetaDataFrame, egui::UiExt as _},
};
use anyhow::Result;
use egui::{CursorIcon, Id, Response, RichText, TextWrapMode, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    CALCULATOR, ERASER, FLOPPY_DISK, LIST, NOTE_PENCIL, SLIDERS_HORIZONTAL, TAG, TRASH,
};
use lipid::prelude::*;
use metadata::egui::MetadataWidget;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;
use tracing::instrument;

const ID_SOURCE: &str = "Configuration";
const COLUMNS: [&str; 6] = [
    "Index",
    "Label",
    "Fatty acid",
    "SN-1,2,3",
    "SN-1,2(2,3)",
    "SN-2",
];

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
    frames: Vec<HashedMetaDataFrame>,
    index: usize,
}

impl Pane {
    pub(crate) fn new(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self { frames, index: 0 }
    }

    pub(crate) const fn icon() -> &'static str {
        NOTE_PENCIL
    }

    pub(crate) fn title(&self) -> String {
        self.title_with_separator(" ")
    }

    fn title_with_separator(&self, separator: &str) -> String {
        self.frames[self.index].meta.format(separator).to_string()
    }

    fn hash(&self) -> u64 {
        hash(&self.frames)
    }
}

impl Pane {
    fn top_content(&mut self, ui: &mut Ui, settings: &mut Settings) -> Response {
        let mut windows = Windows::load(ui.ctx());
        let mut response = ui.heading(Self::icon()).on_hover_ui(|ui| {
            ui.label(ui.localize("Configuration"));
        });
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.hash()))
            .on_hover_ui(|ui| MetadataWidget::new(&self.frames[self.index].meta).show(ui))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
            // ui.set_max_width(ui.spacing().tooltip_width);
            for index in 0..self.frames.len() {
                if ui
                    .selectable_value(
                        &mut self.index,
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
        ui.reset(&mut settings.reset_state);
        // Resize
        ui.resize(&mut settings.resize_table);
        // Edit
        ui.edit(&mut settings.edit_table);
        // Clear
        ui.add_enabled_ui(
            settings.edit_table && self.frames[self.index].data.height() > 0,
            |ui| {
                if ui
                    .button(RichText::new(ERASER).heading())
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("ClearTable"));
                    })
                    .clicked()
                {
                    let data_frame = &mut self.frames[self.index].data;
                    *data_frame = HashedDataFrame::EMPTY;
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
                self.frames.remove(self.index);
                self.index = 0;
            }
        });
        ui.separator();
        // Settings
        ui.settings(&mut windows.open_settings);
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
                data.insert_temp(Id::new(CALCULATE), (self.frames.clone(), self.index));
            });
        }
        ui.separator();
        windows.store(ui.ctx());
        response
    }

    fn central_content_meta(&mut self, ui: &mut Ui, index: usize) {
        ui.style_mut().visuals.collapsing_header_frame = true;
        ui.collapsing(RichText::new(format!("{TAG} Metadata")).heading(), |ui| {
            MetadataWidget::new(&mut self.frames[index].meta)
                .with_writable(true)
                .show(ui);
        });
    }

    fn central_content_data(&mut self, ui: &mut Ui, index: usize, settings: &mut Settings) {
        let data_frame = &mut self.frames[index].data;
        TableView::new(data_frame, settings).show(ui);
    }

    #[instrument(skip(self), err)]
    fn save(&mut self) -> Result<()> {
        let name = format!("{}.utca.ron", self.title_with_separator("."));
        export::ron::save(&mut self.frames[self.index], &name)?;
        Ok(())
    }
}

impl Pane {
    fn windows(&mut self, ui: &mut Ui) {
        let windows = &mut Windows::load(ui.ctx());
        self.settings(ui, windows);
        windows.store(ui.ctx());
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
    fn top(&mut self, ui: &mut Ui) -> Response {
        let mut settings = Settings::load(ui.ctx(), self.hash());
        settings.filter_columns.update(&COLUMNS);
        let response = self.top_content(ui, &mut settings);
        settings.store(ui.ctx(), self.hash());
        response
    }

    fn central(&mut self, ui: &mut Ui) {
        let mut settings = Settings::load(ui.ctx(), self.hash());
        if settings.edit_table {
            self.central_content_meta(ui, self.index);
        }
        self.central_content_data(ui, self.index, &mut settings);
        settings.store(ui.ctx(), self.hash());
        self.windows(ui);
    }
}

mod state;
mod table;
