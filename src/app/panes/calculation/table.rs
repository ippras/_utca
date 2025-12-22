// use super::ID_SOURCE;
#[cfg(feature = "markdown")]
use crate::r#const::markdown::{ENRICHMENT_FACTOR, SELECTIVITY_FACTOR};
use crate::{
    app::{
        computers::calculation::table::{Computed as TableComputed, Key as TableKey},
        panes::MARGIN,
        states::calculation::{ID_SOURCE, State},
        widgets::MeanAndStandardDeviation,
    },
    r#const::*,
    utils::{HashedDataFrame, egui::ResponseExt},
};
use egui::{Frame, Grid, Id, Label, Margin, TextStyle, TextWrapMode, Ui, Widget};
#[cfg(feature = "markdown")]
use egui_ext::Markdown as _;
use egui_l20n::prelude::*;
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use lipid::prelude::*;
use polars::prelude::{array::ArrayNameSpace, *};
use std::ops::Range;
use tracing::instrument;

const LEN: usize = top::FACTORS.end;
const TOP: &[Range<usize>] = &[top::IDENTIFIER, top::STEREOSPECIFIC_NUMBERS, top::FACTORS];

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
                .cache::<TableComputed>()
                .get(TableKey::new(self.data_frame, &self.state.settings))
        })
    }
}

impl TableView<'_> {
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.event.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.event.reset_table_state = false;
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
            (0, top::IDENTIFIER) => {
                ui.heading(ui.localize("Identifier.abbreviation"))
                    .on_hover_localized("Identifier");
            }
            (0, top::STEREOSPECIFIC_NUMBERS) => {
                ui.heading(ui.localize("StereospecificNumber?number=many"));
            }
            (0, top::FACTORS) => {
                ui.heading(ui.localize("Factors"));
            }
            // Bottom
            (1, bottom::INDEX) => {
                ui.heading(HASH).on_hover_localized("Index");
            }
            (1, bottom::LABEL) => {
                ui.heading(ui.localize("Label"));
            }
            (1, bottom::FATTY_ACID) => {
                ui.heading(ui.localize("FattyAcid.abbreviation"))
                    .on_hover_localized("FattyAcid");
            }
            (1, bottom::STEREOSPECIFIC_NUMBERS123) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                    .on_hover_localized("StereospecificNumber?number=123");
            }
            (1, bottom::STEREOSPECIFIC_NUMBERS2) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                    .on_hover_localized("StereospecificNumber?number=2");
            }
            (1, bottom::STEREOSPECIFIC_NUMBERS13) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                    .on_hover_localized("StereospecificNumber?number=13");
            }
            (1, bottom::ENRICHMENT_FACTOR) => {
                #[allow(unused_variables)]
                let response = ui.heading(ui.localize("EnrichmentFactor.abbreviation"));
                #[cfg(feature = "markdown")]
                response.on_hover_ui(|ui| {
                    ui.markdown(ENRICHMENT_FACTOR);
                });
            }
            (1, bottom::SELECTIVITY_FACTOR) => {
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
            self.body_cell_content_ui(ui, row, column)
        } else {
            self.footer_cell_content_ui(ui, row, column)
        }
    }

    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        let data_frame = self.data_frame(ui);
        if let Some(standard) = data_frame[STANDARD]
            .struct_()?
            .field_by_name(MASK)?
            .bool()?
            .get(row)
            && standard
        {
            ui.visuals_mut().override_text_color = Some(ui.visuals().strong_text_color());
        } else if let Some(threshold) = data_frame[THRESHOLD].bool()?.get(row)
            && !threshold
        {
            ui.multiply_opacity(ui.visuals().disabled_alpha());
        }
        match (row, column) {
            (row, bottom::INDEX) => {
                ui.label(row.to_string());
            }
            (row, bottom::LABEL) => {
                if let Some(text) = data_frame[LABEL].str()?.get(row) {
                    Label::new(text).truncate().ui(ui).try_on_hover_ui(
                        |ui| -> PolarsResult<()> {
                            ui.heading(ui.localize(PROPERTIES));
                            let properties = &data_frame[PROPERTIES];
                            Grid::new(ui.next_auto_id())
                                .show(ui, |ui| {
                                    ui.label(ui.localize(IODINE_VALUE));
                                    ui.label(
                                        properties
                                            .struct_()?
                                            .field_by_name(IODINE_VALUE)?
                                            .get(row)?
                                            .str_value(),
                                    );
                                    ui.end_row();

                                    ui.label(ui.localize(RELATIVE_ATOMIC_MASS));
                                    ui.label(
                                        properties
                                            .struct_()?
                                            .field_by_name(RELATIVE_ATOMIC_MASS)?
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
            (row, bottom::FATTY_ACID) => {
                if let Some(fatty_acid) = data_frame.try_fatty_acid()?.delta()?.get(row) {
                    // let mut text = RichText::new(fatty_acid);
                    // Strong standard and weak filtered
                    // text = match data_frame[THRESHOLD].bool()?.get(row) {
                    //     None => text.strong(),
                    //     Some(false) => text.weak(),
                    //     Some(true) => text,
                    // };
                    Label::new(fatty_acid).truncate().ui(ui);
                }
            }
            (row, bottom::STEREOSPECIFIC_NUMBERS123) => {
                MeanAndStandardDeviation::new(&data_frame, [STEREOSPECIFIC_NUMBERS123], row)
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .show(ui)?
                    .try_on_hover_ui(|ui| -> PolarsResult<()> {
                        ui.heading(ui.localize(STANDARD));
                        let factors = &data_frame[STANDARD]
                            .struct_()?
                            .field_by_name(STEREOSPECIFIC_NUMBERS123)?;
                        let mean = factors
                            .struct_()?
                            .field_by_name(MEAN)?
                            .f64()?
                            .get(row)
                            .unwrap_or_default();
                        let standard_deviation = factors
                            .struct_()?
                            .field_by_name(STANDARD_DEVIATION)?
                            .f64()?
                            .get(row)
                            .unwrap_or_default();
                        let sample_series = factors.struct_()?.field_by_name(SAMPLE)?;
                        let sample = sample_series.get(row)?.str_value();
                        Grid::new(ui.next_auto_id())
                            .show(ui, |ui| {
                                ui.label(ui.localize(FACTORS));
                                ui.label(format!(
                                    "{mean}{NO_BREAK_SPACE}±{standard_deviation} {sample}"
                                ));
                                ui.end_row();
                                Ok(())
                            })
                            .inner
                    })?;
            }
            (row, bottom::STEREOSPECIFIC_NUMBERS2) => {
                MeanAndStandardDeviation::new(&data_frame, [STEREOSPECIFIC_NUMBERS2], row)
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .show(ui)?;
            }
            (row, bottom::STEREOSPECIFIC_NUMBERS13) => {
                MeanAndStandardDeviation::new(&data_frame, [STEREOSPECIFIC_NUMBERS13], row)
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .with_calculation(true)
                    .show(ui)?;
            }
            (row, bottom::ENRICHMENT_FACTOR) => {
                MeanAndStandardDeviation::new(&data_frame, [FACTORS, ENRICHMENT], row)
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .with_calculation(true)
                    .show(ui)?;
            }
            (row, bottom::SELECTIVITY_FACTOR) => {
                MeanAndStandardDeviation::new(&data_frame, [FACTORS, SELECTIVITY], row)
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .with_calculation(true)
                    .show(ui)?;
            }
            _ => {}
        }
        Ok(())
    }

    fn footer_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        let data_frame = self.data_frame(ui);
        match (row, column) {
            (row, bottom::STEREOSPECIFIC_NUMBERS123) => {
                MeanAndStandardDeviation::new(&data_frame, [STEREOSPECIFIC_NUMBERS123], row)
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .show(ui)?;
            }
            (row, bottom::STEREOSPECIFIC_NUMBERS2) => {
                MeanAndStandardDeviation::new(&data_frame, [STEREOSPECIFIC_NUMBERS2], row)
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .show(ui)?;
            }
            (row, bottom::STEREOSPECIFIC_NUMBERS13) => {
                MeanAndStandardDeviation::new(&data_frame, [STEREOSPECIFIC_NUMBERS13], row)
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .with_calculation(true)
                    .show(ui)?;
            }
            _ => {}
        }
        Ok(())
    }

    // fn mean_and_standard_deviation(
    //     &self,
    //     ui: &mut Ui,
    //     column: &'static str,
    //     row: usize,
    // ) -> PolarsResult<Response> {
    //     let data_frame = self.data_frame(ui);
    //     // MeanAndStandardDeviation::new(&data_frame, &format!("{column}.Mean"), row).show(ui)?;
    //     // let data_frame = self.data_frame(ui);
    //     // let mean = data_frame[&*format!("{column}.Mean")].f64()?.get(row);
    //     // let standard_deviation = data_frame[&*format!("{column}.StandardDeviation")]
    //     //     .f64()?
    //     //     .get(row);
    //     // let text = match mean {
    //     //     Some(mean)
    //     //         if self.state.settings.display_standard_deviation
    //     //             && let Some(standard_deviation) = standard_deviation =>
    //     //     {
    //     //         WidgetText::from(format!("{mean} {standard_deviation}"))
    //     //     }
    //     //     Some(mean) => WidgetText::from(mean.to_string()),
    //     //     None => WidgetText::from(""),
    //     // };
    //     // let mut response = ui.label(text);
    //     // if response.hovered() {
    //     //     // Standard deviation
    //     //     if let Some(text) = standard_deviation {
    //     //         response = response.on_hover_ui(|ui| {
    //     //             ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
    //     //             ui.heading(ui.localize("StandardDeviation"));
    //     //             ui.label(text);
    //     //         });
    //     //     }
    //     // }
    //     Ok(response)
    // }

    // fn with_array(&self, ui: &mut Ui, column: &'static str, row: usize) -> PolarsResult<Response> {
    //     let mut response = self.mean_and_standard_deviation(ui, column, row)?;
    //     if response.hovered() {
    //         let data_frame = self.data_frame(ui);
    //         // Array
    //         if let Some(text) = data_frame[&*format!("{column}.Array")].str()?.get(row) {
    //             response = response.on_hover_ui(|ui| {
    //                 ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
    //                 ui.heading(ui.localize("Array"));
    //                 ui.label(text);
    //             });
    //         }
    //     }
    //     Ok(response)
    // }

    // fn with_calculation(
    //     &self,
    //     ui: &mut Ui,
    //     column: &'static str,
    //     row: usize,
    // ) -> PolarsResult<Response> {
    //     let mut response = self.with_array(ui, column, row)?;
    //     if response.hovered() {
    //         let data_frame = self.data_frame(ui);
    //         // Calculation
    //         if let Some(text) = data_frame[&*format!("{column}.Calculation")]
    //             .str()?
    //             .get(row)
    //         {
    //             response = response.on_hover_ui(|ui| {
    //                 ui.style_mut().wrap_mode = Some(TextWrapMode::Extend);
    //                 ui.heading(ui.localize("Calculation"));
    //                 ui.label(text);
    //             });
    //         }
    //     }
    //     Ok(response)
    // }
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
                _ = self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }
}

mod top {
    use super::*;

    pub(super) const IDENTIFIER: Range<usize> = 0..3;
    pub(super) const STEREOSPECIFIC_NUMBERS: Range<usize> = IDENTIFIER.end..IDENTIFIER.end + 3;
    pub(super) const FACTORS: Range<usize> =
        STEREOSPECIFIC_NUMBERS.end..STEREOSPECIFIC_NUMBERS.end + 2;
}

mod bottom {
    use super::*;

    pub(super) const INDEX: Range<usize> = top::IDENTIFIER.start..top::IDENTIFIER.start + 1;
    pub(super) const LABEL: Range<usize> = INDEX.end..INDEX.end + 1;
    pub(super) const FATTY_ACID: Range<usize> = LABEL.end..LABEL.end + 1;
    pub(super) const STEREOSPECIFIC_NUMBERS123: Range<usize> =
        top::STEREOSPECIFIC_NUMBERS.start..top::STEREOSPECIFIC_NUMBERS.start + 1;
    pub(super) const STEREOSPECIFIC_NUMBERS2: Range<usize> =
        STEREOSPECIFIC_NUMBERS123.end..STEREOSPECIFIC_NUMBERS123.end + 1;
    pub(super) const STEREOSPECIFIC_NUMBERS13: Range<usize> =
        STEREOSPECIFIC_NUMBERS2.end..STEREOSPECIFIC_NUMBERS2.end + 1;
    pub(super) const ENRICHMENT_FACTOR: Range<usize> = top::FACTORS.start..top::FACTORS.start + 1;
    pub(super) const SELECTIVITY_FACTOR: Range<usize> =
        ENRICHMENT_FACTOR.end..ENRICHMENT_FACTOR.end + 1;
}
