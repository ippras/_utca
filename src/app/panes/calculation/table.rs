use super::ID_SOURCE;
#[cfg(feature = "markdown")]
use crate::r#const::markdown::{ENRICHMENT_FACTOR, SELECTIVITY_FACTOR};
use crate::{
    app::{
        computers::calculation::display::{
            Computed as CalculationDisplayComputed, Key as CalculationDisplayKey,
        },
        panes::MARGIN,
        states::calculation::State,
    },
    utils::HashedDataFrame,
};
use egui::{Frame, Id, Label, Margin, Response, TextStyle, TextWrapMode, Ui, Widget};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_l20n::UiExt;
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use lipid::prelude::*;
use polars::prelude::*;
use std::ops::Range;
use tracing::instrument;

const LEN: usize = top::FS.end;
const TOP: &[Range<usize>] = &[top::ID, top::SN, top::FS];

/// Calculation table
pub(crate) struct TableView<'a> {
    data_frame: &'a HashedDataFrame,
    state: &'a mut State,
}

impl<'a> TableView<'a> {
    pub(crate) fn new(data_frame: &'a HashedDataFrame, state: &'a mut State) -> Self {
        Self { data_frame, state }
    }
}

impl TableView<'_> {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.settings.table.reset_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.settings.table.reset_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default()
                    .resizable(self.state.settings.table.resizable);
                num_columns
            ])
            .num_sticky_cols(self.state.settings.table.sticky_columns)
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
        if self.state.settings.table.truncate_headers {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, top::ID) => {
                ui.heading(ui.localize("Identifier.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("Identifier"));
                    });
            }
            (0, top::SN) => {
                ui.heading(ui.localize("StereospecificNumber?number=many"));
            }
            (0, top::FS) => {
                ui.heading(ui.localize("Factors"));
            }
            // Bottom
            (1, bottom::INDEX) => {
                ui.heading(HASH).on_hover_ui(|ui| {
                    ui.label(ui.localize("Index"));
                });
            }
            (1, bottom::LABEL) => {
                ui.heading(ui.localize("Label"));
            }
            (1, bottom::FA) => {
                ui.heading(ui.localize("FattyAcid.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("FattyAcid"));
                    });
            }
            (1, bottom::SN123) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=123"));
                    });
            }
            (1, bottom::SN2) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=2"));
                    });
            }
            (1, bottom::SN13) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=13"));
                    });
            }
            (1, bottom::EF) => {
                #[allow(unused_variables)]
                let response = ui.heading(ui.localize("EnrichmentFactor.abbreviation"));
                #[cfg(feature = "markdown")]
                response.on_hover_ui(|ui| {
                    ui.markdown(ENRICHMENT_FACTOR);
                });
            }
            (1, bottom::SF) => {
                #[allow(unused_variables)]
                let response = ui.heading(ui.localize("SelectivityFactor.abbreviation"));
                #[cfg(feature = "markdown")]
                response.on_hover_ui(|ui| {
                    ui.markdown(SELECTIVITY_FACTOR);
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
            (row, bottom::INDEX) => {
                ui.label(row.to_string());
            }
            (row, bottom::LABEL) => {
                let labels = self.data_frame[LABEL].str()?;
                let label = labels.get(row).unwrap();
                Label::new(label).truncate().ui(ui);
            }
            (row, bottom::FA) => {
                if let Some(fatty_acid) = self.data_frame.try_fatty_acid()?.delta()?.get(row) {
                    Label::new(fatty_acid).truncate().ui(ui);
                }
            }
            (row, bottom::SN123) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<CalculationDisplayComputed>().get(
                        CalculationDisplayKey::stereospecific_numbers123(
                            self.data_frame,
                            &self.state.settings,
                        ),
                    )
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?;
                }
                if self.state.settings.display_standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            (row, bottom::SN2) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<CalculationDisplayComputed>().get(
                        CalculationDisplayKey::stereospecific_numbers2(
                            self.data_frame,
                            &self.state.settings,
                        ),
                    )
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?;
                }
                if self.state.settings.display_standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            (row, bottom::SN13) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<CalculationDisplayComputed>().get(
                        CalculationDisplayKey::stereospecific_numbers13(
                            self.data_frame,
                            &self.state.settings,
                        ),
                    )
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?
                        .calculation(&data_frame, row)?;
                }
                if self.state.settings.display_standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            (row, bottom::EF) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<CalculationDisplayComputed>().get(
                        CalculationDisplayKey::enrichment_factor(
                            self.data_frame,
                            &self.state.settings,
                        ),
                    )
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?
                        .calculation(&data_frame, row)?;
                }
                if self.state.settings.display_standard_deviation {
                    if let Some(standard_deviation) =
                        data_frame["StandardDeviation"].str()?.get(row)
                    {
                        ui.label(standard_deviation);
                    }
                }
            }
            (row, bottom::SF) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<CalculationDisplayComputed>().get(
                        CalculationDisplayKey::selectivity_factor(
                            self.data_frame,
                            &self.state.settings,
                        ),
                    )
                });
                let mean = data_frame["Mean"].get(row)?.str_value();
                let response = ui.label(mean);
                if response.hovered() {
                    response
                        .standard_deviation(&data_frame, row)?
                        .array(&data_frame, row)?
                        .calculation(&data_frame, row)?;
                }
                if self.state.settings.display_standard_deviation {
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
            bottom::SN123 => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<CalculationDisplayComputed>().get(
                        CalculationDisplayKey::stereospecific_numbers123(
                            self.data_frame,
                            &self.state.settings,
                        ),
                    )
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
            bottom::SN2 => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<CalculationDisplayComputed>().get(
                        CalculationDisplayKey::stereospecific_numbers2(
                            self.data_frame,
                            &self.state.settings,
                        ),
                    )
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
            bottom::SN13 => {
                let data_frame = ui.memory_mut(|memory| {
                    memory.caches.cache::<CalculationDisplayComputed>().get(
                        CalculationDisplayKey::stereospecific_numbers13(
                            self.data_frame,
                            &self.state.settings,
                        ),
                    )
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

mod top {
    use super::*;

    pub(super) const ID: Range<usize> = 0..3;
    pub(super) const SN: Range<usize> = ID.end..ID.end + 3;
    pub(super) const FS: Range<usize> = SN.end..SN.end + 2;
}

// ENRICHMENT_FACTOR, SELECTIVITY_FACTOR
mod bottom {
    use super::*;

    pub(super) const INDEX: Range<usize> = top::ID.start..top::ID.start + 1;
    pub(super) const LABEL: Range<usize> = INDEX.end..INDEX.end + 1;
    pub(super) const FA: Range<usize> = LABEL.end..LABEL.end + 1;
    pub(super) const SN123: Range<usize> = top::SN.start..top::SN.start + 1;
    pub(super) const SN2: Range<usize> = SN123.end..SN123.end + 1;
    pub(super) const SN13: Range<usize> = SN2.end..SN2.end + 1;
    pub(super) const EF: Range<usize> = top::FS.start..top::FS.start + 1;
    pub(super) const SF: Range<usize> = EF.end..EF.end + 1;
}
