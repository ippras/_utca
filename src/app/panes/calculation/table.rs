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
    utils::{HashedDataFrame, egui::ResponseExt},
};
use egui::{
    Frame, Grid, Id, Label, Margin, Response, RichText, TextStyle, TextWrapMode, Ui, Widget,
    WidgetText,
};
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

    fn data_frame(&self, ui: &Ui) -> DataFrame {
        ui.memory_mut(|memory| {
            memory
                .caches
                .cache::<CalculationDisplayComputed>()
                .get(CalculationDisplayKey::new(
                    self.data_frame,
                    &self.state.settings,
                ))
        })
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
        let data_frame = self.data_frame(ui);
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = data_frame.height() as u64;
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
        let data_frame = self.data_frame(ui);
        match (row, column) {
            (row, bottom::INDEX) if row + 1 < data_frame.height() => {
                ui.label(row.to_string());
            }
            (row, bottom::LABEL) => {
                if let Some(text) = data_frame[LABEL].str()?.get(row) {
                    Label::new(text).truncate().ui(ui).try_on_hover_ui(
                        |ui| -> PolarsResult<()> {
                            ui.heading(ui.localize("Properties"));
                            Grid::new(ui.next_auto_id())
                                .show(ui, |ui| {
                                    ui.label(ui.localize("IodineValue"));
                                    ui.label(
                                        data_frame["Properties.IodineValue"].get(row)?.str_value(),
                                    );
                                    ui.end_row();

                                    ui.label(ui.localize("RelativeAtomicMass"));
                                    ui.label(
                                        data_frame["Properties.RelativeAtomicMass"]
                                            .get(row)?
                                            .str_value(),
                                    );
                                    ui.end_row();
                                    Ok(())
                                })
                                .inner
                        },
                    )?;
                }
            }
            (row, bottom::FA) => {
                if let Some(fatty_acid) = data_frame.try_fatty_acid()?.delta()?.get(row) {
                    let mut text = RichText::new(fatty_acid);
                    // Strong standard and weak filtered
                    text = match data_frame["Filter"].bool()?.get(row) {
                        None => text.strong(),
                        Some(false) => text.weak(),
                        Some(true) => text,
                    };
                    Label::new(text).truncate().ui(ui);
                }
            }
            (row, bottom::SN123) => {
                self.with_array(ui, STEREOSPECIFIC_NUMBERS123, row)?;
            }
            (row, bottom::SN2) => {
                self.with_array(ui, STEREOSPECIFIC_NUMBERS2, row)?;
            }
            (row, bottom::SN13) => {
                self.with_calculation(ui, STEREOSPECIFIC_NUMBERS13, row)?;
            }
            (row, bottom::EF) => {
                self.with_calculation(ui, "Factors.Enrichment", row)?;
            }
            (row, bottom::SF) => {
                self.with_calculation(ui, "Factors.Selectivity", row)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn mean_and_standard_deviation(
        &self,
        ui: &mut Ui,
        column: &'static str,
        row: usize,
    ) -> PolarsResult<Response> {
        let data_frame = self.data_frame(ui);
        let mean = data_frame[&*format!("{column}.Mean")].str()?.get(row);
        let standard_deviation = data_frame[&*format!("{column}.StandardDeviation")]
            .str()?
            .get(row);
        let text = match mean {
            Some(mean)
                if self.state.settings.display_standard_deviation
                    && let Some(standard_deviation) = standard_deviation =>
            {
                WidgetText::from(format!("{mean} {standard_deviation}"))
            }
            Some(mean) => WidgetText::from(mean.to_string()),
            None => WidgetText::from(""),
        };
        let mut response = ui.label(text);
        if response.hovered() {
            // Standard deviation
            if let Some(text) = standard_deviation {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize("StandardDeviation"));
                    ui.label(text);
                });
            }
        }
        Ok(response)
    }

    fn with_array(&self, ui: &mut Ui, column: &'static str, row: usize) -> PolarsResult<Response> {
        let mut response = self.mean_and_standard_deviation(ui, column, row)?;
        if response.hovered() {
            let data_frame = self.data_frame(ui);
            // Array
            if let Some(text) = data_frame[&*format!("{column}.Array")].str()?.get(row) {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize("Array"));
                    ui.label(text);
                });
            }
        }
        Ok(response)
    }

    fn with_calculation(
        &self,
        ui: &mut Ui,
        column: &'static str,
        row: usize,
    ) -> PolarsResult<Response> {
        let mut response = self.with_array(ui, column, row)?;
        if response.hovered() {
            let data_frame = self.data_frame(ui);
            // Calculation
            if let Some(text) = data_frame[&*format!("{column}.Calculation")]
                .str()?
                .get(row)
            {
                response = response.on_hover_ui(|ui| {
                    ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
                    ui.heading(ui.localize("Calculation"));
                    ui.label(text);
                });
            }
        }
        Ok(response)
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

mod top {
    use super::*;

    pub(super) const ID: Range<usize> = 0..3;
    pub(super) const SN: Range<usize> = ID.end..ID.end + 3;
    pub(super) const FS: Range<usize> = SN.end..SN.end + 2;
}

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
