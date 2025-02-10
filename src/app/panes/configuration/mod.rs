use self::{settings::Settings, state::State, table::TableView};
use super::PaneDelegate;
use crate::{app::ContextExt, utils::save};
use anyhow::Result;
use egui::{CursorIcon, Id, Response, RichText, Ui, Window, util::hash};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{
    ARROWS_CLOCKWISE, ARROWS_HORIZONTAL, CALCULATOR, ERASER, FLOPPY_DISK, GEAR, LIST, NOTE_PENCIL,
    PENCIL, TAG, TRASH,
};
use metadata::MetaDataFrame;
use polars::prelude::*;
use serde::{Deserialize, Serialize};
use std::sync::LazyLock;

const ID_SOURCE: &str = "Configuration";

pub(crate) static SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
    Schema::from_iter([
        Field::new("Index".into(), DataType::UInt32),
        Field::new("Label".into(), DataType::String),
        Field::new(
            "FattyAcid".into(),
            DataType::Struct(vec![
                Field::new("Carbons".into(), DataType::UInt8),
                Field::new(
                    "Unsaturated".into(),
                    DataType::List(Box::new(DataType::Struct(vec![
                        Field::new("Index".into(), DataType::UInt8),
                        Field::new("Isomerism".into(), DataType::Int8),
                        Field::new("Unsaturation".into(), DataType::UInt8),
                    ]))),
                ),
            ]),
        ),
        Field::new("Triacylglycerol".into(), DataType::Float64),
        Field::new("Diacylglycerol1223".into(), DataType::Float64),
        Field::new("Monoacylglycerol2".into(), DataType::Float64),
    ])
});

/// Configuration pane
#[derive(Default, Deserialize, Serialize)]
pub(crate) struct Pane {
    pub(crate) frames: Vec<MetaDataFrame>,
    pub(crate) settings: Settings,
    state: State,
}

impl Pane {
    pub(crate) const fn new(frames: Vec<MetaDataFrame>) -> Self {
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
        self.frames[self.settings.index].meta.title()
    }

    fn header_content(&mut self, ui: &mut Ui) -> Response {
        let mut response = ui
            .heading(Self::icon())
            .on_hover_text(ui.localize("configuration"));
        response |= ui.heading(self.title());
        response = response
            .on_hover_text(format!("{:x}", self.hash()))
            .on_hover_cursor(CursorIcon::Grab);
        ui.separator();
        // List
        ui.menu_button(RichText::new(LIST).heading(), |ui| {
            for index in 0..self.frames.len() {
                if ui
                    .selectable_value(
                        &mut self.settings.index,
                        index,
                        self.frames[index].meta.title(),
                    )
                    .clicked()
                {
                    ui.close_menu();
                }
            }
        })
        .response
        .on_hover_ui(|ui| {
            ui.label(ui.localize("list"));
        });
        ui.separator();
        // Reset
        if ui
            .button(RichText::new(ARROWS_CLOCKWISE).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("reset_table"));
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
            ui.label(ui.localize("resize_table"));
        });
        // Edit
        ui.toggle_value(&mut self.settings.editable, RichText::new(PENCIL).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("edit"));
            });
        // Clear
        ui.add_enabled_ui(
            self.settings.editable && self.frames[self.settings.index].data.height() > 0,
            |ui| {
                if ui
                    .button(RichText::new(ERASER).heading())
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("clear_table"));
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
                    ui.label(ui.localize("delete_table"));
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
            ui.label(ui.localize("settings"));
        });
        ui.separator();
        // Save
        if ui
            .button(RichText::new(FLOPPY_DISK).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("save"));
            })
            .on_hover_text(format!("{}.utca.ipc", self.title()))
            .clicked()
        {
            if let Err(error) = self.save() {
                ui.ctx().error(error);
            }
        }
        ui.separator();
        // Calculation
        if ui
            .button(RichText::new(CALCULATOR).heading())
            .on_hover_ui(|ui| {
                ui.label(ui.localize("calculation"));
            })
            .clicked()
        {
            ui.data_mut(|data| {
                data.insert_temp(
                    Id::new("Calculate"),
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
            self.frames[index].meta.show(ui);
        });
    }

    fn body_content_data(&mut self, ui: &mut Ui, index: usize) {
        let data_frame = &mut self.frames[index].data;
        TableView::new(data_frame, &self.settings, &mut self.state).show(ui);
    }

    fn _add_row(&mut self) -> PolarsResult<()> {
        let data_frame = &mut self.frames[self.settings.index].data;
        *data_frame = concat(
            [
                data_frame.clone().lazy(),
                df! {
                    data_frame[0].name().clone() => [data_frame.height() as u32],
                    "Label" => [""],
                    "FattyAcid" => df! {
                        "Carbons" => [0u8],
                        "Unsaturated" => [
                            df! {
                                "Index" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8),
                                "Isomerism" => Series::new_empty(PlSmallStr::EMPTY, &DataType::Int8),
                                "Unsaturation" => Series::new_empty(PlSmallStr::EMPTY, &DataType::UInt8),
                            }?.into_struct(PlSmallStr::EMPTY).into_series(),
                        ],
                    }?.into_struct(PlSmallStr::EMPTY),
                    "Triacylglycerol" => [0f64],
                    "Diacylglycerol1223" => [0f64],
                    "Monoacylglycerol2" => [0f64],
                }?
                .lazy(),
            ],
            UnionArgs {
                rechunk: true,
                diagonal: true,
                ..Default::default()
            },
        )?
        .collect()?;
        Ok(())
    }

    fn _delete_row(&mut self, row: usize) -> PolarsResult<()> {
        let data_frame = &mut self.frames[self.settings.index].data;
        let mut lazy_frame = data_frame.clone().lazy();
        lazy_frame = lazy_frame
            .filter(nth(0).neq(lit(row as u32)))
            .with_column(nth(0).cum_count(false) - lit(1));
        *data_frame = lazy_frame.collect()?;
        Ok(())
    }

    pub(crate) fn windows(&mut self, ui: &mut Ui) {
        Window::new(format!("{GEAR} Configuration settings"))
            .id(ui.auto_id_with(ID_SOURCE))
            .default_pos(ui.next_widget_position())
            .open(&mut self.state.open_settings_window)
            .show(ui.ctx(), |ui| self.settings.show(ui));
    }

    fn hash(&self) -> u64 {
        hash(&self.frames)
    }

    fn save(&mut self) -> Result<()> {
        let name = format!("{}.utca.ipc", self.title());
        save(&name, &mut self.frames[self.settings.index])?;
        Ok(())
    }
}

impl PaneDelegate for Pane {
    fn header(&mut self, ui: &mut Ui) -> Response {
        self.header_content(ui)
    }

    fn body(&mut self, ui: &mut Ui) {
        self.windows(ui);
        if self.settings.editable {
            self.body_content_meta(ui, self.settings.index);
        }
        self.body_content_data(ui, self.settings.index);
    }
}

pub(crate) mod settings;

mod state;
mod table;
