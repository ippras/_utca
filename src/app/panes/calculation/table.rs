use super::{ID_SOURCE, parameters::Parameters, state::Settings};
#[cfg(feature = "markdown")]
use crate::asset;
use crate::{
    app::{
        computers::{
            CalculationDisplayComputed, CalculationDisplayKey, CalculationDisplaySettings,
        },
        panes::MARGIN,
    },
    utils::HashedDataFrame,
};
use egui::{Context, Frame, Id, Label, Margin, Response, TextStyle, TextWrapMode, Ui, Widget};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_l20n::UiExt;
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use lipid::prelude::*;
use polars::prelude::*;
use std::ops::Range;
use tracing::instrument;

const ID: Range<usize> = 0..3;
const SN: Range<usize> = ID.end..ID.end + 3;
const FACTORS: Range<usize> = SN.end..SN.end + 2;
const LEN: usize = FACTORS.end;

const TOP: &[Range<usize>] = &[ID, SN, FACTORS];

/// Calculation table
pub(crate) struct TableView<'a> {
    data_frame: &'a HashedDataFrame,
    parameters: &'a Parameters,
    settings: Settings,
}

impl<'a> TableView<'a> {
    pub(crate) fn new(
        ctx: &Context,
        data_frame: &'a HashedDataFrame,
        parameters: &'a Parameters,
    ) -> Self {
        Self {
            data_frame,
            parameters,
            settings: Settings::load(ctx),
        }
    }
}

impl TableView<'_> {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.settings.table.reset_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.settings.table.reset_state = false;
            self.settings.clone().store(ui.ctx());
        }
        // let settings = Settings::load(ui.ctx());
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default()
                    .resizable(self.settings.table.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.table.sticky_columns)
            .headers([
                HeaderRow {
                    height,
                    groups: TOP.to_vec(),
                },
                HeaderRow::new(height),
            ])
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.settings.table.truncate_headers {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, ID) => {
                ui.heading(ui.localize("Identifier.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("Identifier"));
                    });
            }
            (0, SN) => {
                ui.heading(ui.localize("StereospecificNumber?number=many"));
            }
            (0, FACTORS) => {
                ui.heading(ui.localize("Factors"));
            }
            // Bottom
            (1, identifier::INDEX) => {
                ui.heading(HASH).on_hover_ui(|ui| {
                    ui.label(ui.localize("Index"));
                });
            }
            (1, identifier::LABEL) => {
                ui.heading(ui.localize("Label"));
            }
            (1, identifier::FA) => {
                ui.heading(ui.localize("FattyAcid.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("FattyAcid"));
                    });
            }
            (1, stereospecific_numbers::SN123) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=123"));
                    });
            }
            (1, stereospecific_numbers::SN2) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=2"));
                    });
            }
            (1, stereospecific_numbers::SN13) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=13"));
                    });
            }
            (1, factors::EF) => {
                let response = ui.heading(ui.localize("EnrichmentFactor.abbreviation"));
                #[cfg(feature = "markdown")]
                response.on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/en/Factors/EnrichmentFactor.md"));
                })
            }
            (1, factors::SF) => {
                let response = ui.heading(ui.localize("SelectivityFactor.abbreviation"));
                #[cfg(feature = "markdown")]
                response.on_hover_ui(|ui| {
                    ui.markdown(asset!("/doc/en/Factors/SelectivityFactor.md"));
                });
            }
            _ => {}
        };
    }

    #[instrument(skip(self, ui), err)]
    fn cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        if row != self.data_frame.height() {
            self.body_cell_content_ui(ui, row, column)?;
        } else {
            self.footer_cell_content_ui(ui, column)?;
        }
        Ok(())
    }

    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        match (row, column) {
            (row, identifier::INDEX) => {
                ui.label(row.to_string());
            }
            (row, identifier::LABEL) => {
                let labels = self.data_frame[LABEL].str()?;
                let label = labels.get(row).unwrap();
                Label::new(label).truncate().ui(ui);
            }
            (row, identifier::FA) => {
                if let Some(fatty_acid) = self.data_frame.try_fatty_acid()?.delta()?.get(row) {
                    Label::new(fatty_acid).truncate().ui(ui);
                }
            }
            (row, stereospecific_numbers::SN123) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationDisplayComputed>()
                        .get(CalculationDisplayKey {
                            frame: self.data_frame,
                            settings: CalculationDisplaySettings::stereospecific_numbers123(
                                &self.settings,
                            ),
                        })
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?;
                }
                if self.settings.standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            (row, stereospecific_numbers::SN2) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationDisplayComputed>()
                        .get(CalculationDisplayKey {
                            frame: self.data_frame,
                            settings: CalculationDisplaySettings::stereospecific_numbers2(
                                &self.settings,
                            ),
                        })
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?;
                }
                if self.settings.standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            (row, stereospecific_numbers::SN13) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationDisplayComputed>()
                        .get(CalculationDisplayKey {
                            frame: self.data_frame,
                            settings: CalculationDisplaySettings::stereospecific_numbers13(
                                &self.settings,
                            ),
                        })
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?
                        .calculation(&data_frame, row)?;
                }
                if self.settings.standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            (row, factors::EF) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationDisplayComputed>()
                        .get(CalculationDisplayKey {
                            frame: self.data_frame,
                            settings: CalculationDisplaySettings::enrichment_factor(&self.settings),
                        })
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?
                        .calculation(&data_frame, row)?;
                }
                if self.settings.standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            (row, factors::SF) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationDisplayComputed>()
                        .get(CalculationDisplayKey {
                            frame: self.data_frame,
                            settings: CalculationDisplaySettings::selectivity_factor(
                                &self.settings,
                            ),
                        })
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?
                        .calculation(&data_frame, row)?;
                }
                if self.settings.standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        match column {
            stereospecific_numbers::SN123 => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationDisplayComputed>()
                        .get(CalculationDisplayKey {
                            frame: self.data_frame,
                            settings: CalculationDisplaySettings::stereospecific_numbers123(
                                &self.settings,
                            ),
                        })
                });
                let row = data_frame.height() - 1;
                if let Some(mean) = data_frame["Mean"].str()?.get(row) {
                    let response = ui.label(mean);
                    if response.hovered() {
                        response
                            .on_hover_text(format!(
                                "∑ {}",
                                ui.localize("StereospecificNumber.abbreviation?number=123")
                            ))
                            .standard_deviation(&data_frame, row)?
                            .array(&data_frame, row)?;
                    }
                }
            }
            stereospecific_numbers::SN2 => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationDisplayComputed>()
                        .get(CalculationDisplayKey {
                            frame: self.data_frame,
                            settings: CalculationDisplaySettings::stereospecific_numbers2(
                                &self.settings,
                            ),
                        })
                });
                let row = data_frame.height() - 1;
                if let Some(mean) = data_frame["Mean"].str()?.get(row) {
                    let response = ui.label(mean);
                    if response.hovered() {
                        response
                            .on_hover_text(format!(
                                "∑ {}",
                                ui.localize("StereospecificNumber.abbreviation?number=2")
                            ))
                            .standard_deviation(&data_frame, row)?
                            .array(&data_frame, row)?;
                    }
                }
            }
            stereospecific_numbers::SN13 => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<CalculationDisplayComputed>()
                        .get(CalculationDisplayKey {
                            frame: self.data_frame,
                            settings: CalculationDisplaySettings::stereospecific_numbers13(
                                &self.settings,
                            ),
                        })
                });
                let row = data_frame.height() - 1;
                if let Some(mean) = data_frame["Mean"].str()?.get(row) {
                    let response = ui.label(mean);
                    if response.hovered() {
                        response
                            .on_hover_text(format!(
                                "∑ {}",
                                ui.localize("StereospecificNumber.abbreviation?number=13")
                            ))
                            .standard_deviation(&data_frame, row)?
                            .array(&data_frame, row)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }
}

impl TableDelegate for TableView<'_> {
    fn header_cell_ui(&mut self, ui: &mut Ui, cell: &HeaderCellInfo) {
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                self.header_cell_content_ui(ui, cell.row_nr, cell.col_range.clone())
            });
    }

    fn cell_ui(&mut self, ui: &mut Ui, cell: &CellInfo) {
        if cell.row_nr % 2 == 0 {
            ui.painter()
                .rect_filled(ui.max_rect(), 0.0, ui.visuals().faint_bg_color);
        }
        Frame::new()
            .inner_margin(Margin::from(MARGIN))
            .show(ui, |ui| {
                let _ = self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }
}

/// Extension methods for [`Response`]
trait ResponseExt: Sized {
    fn calculation(self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self>;

    fn standard_deviation(self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self>;

    fn array(self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self>;
}

impl ResponseExt for Response {
    fn calculation(mut self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self> {
        if let Some(calculation) = data_frame["Calculation"].str()?.get(row) {
            self = self.on_hover_ui(|ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                ui.heading(ui.localize("Calculation"));
                ui.label(calculation);
            });
        }
        Ok(self)
    }

    fn standard_deviation(mut self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self> {
        if let Some(text) = data_frame["StandardDeviation"].str()?.get(row) {
            self = self.on_hover_ui(|ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                ui.heading(ui.localize("StandardDeviation"));
                ui.label(text);
            });
        }
        Ok(self)
    }

    fn array(mut self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self> {
        if let Some(text) = data_frame["Array"].str()?.get(row) {
            self = self.on_hover_ui(|ui| {
                ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                ui.heading(ui.localize("Array"));
                ui.label(text);
            });
        }
        Ok(self)
    }
}

// mod columns {
//     use super::*;

//     const ID: Range<usize> = 0..3;
//     const SN: Range<usize> = ID.end..ID.end + 3;
//     pub(super) const SN123: Range<usize> = SN.start..SN.start + 1;
//     pub(super) const SN2: Range<usize> = SN123.end..SN123.end + 1;
//     pub(super) const SN13: Range<usize> = SN2.end..SN2.end + 1;

//     const FACTORS: Range<usize> = SN.end..SN.end + 2;
//     const LEN: usize = FACTORS.end;
// }

mod identifier {
    use super::*;

    pub(super) const INDEX: Range<usize> = ID.start..ID.start + 1;
    pub(super) const LABEL: Range<usize> = INDEX.end..INDEX.end + 1;
    pub(super) const FA: Range<usize> = LABEL.end..LABEL.end + 1;
}

mod stereospecific_numbers {
    use super::*;

    pub(super) const SN123: Range<usize> = SN.start..SN.start + 1;
    pub(super) const SN2: Range<usize> = SN123.end..SN123.end + 1;
    pub(super) const SN13: Range<usize> = SN2.end..SN2.end + 1;
}

mod factors {
    use super::*;

    pub(super) const EF: Range<usize> = FACTORS.start..FACTORS.start + 1;
    pub(super) const SF: Range<usize> = EF.end..EF.end + 1;
}
