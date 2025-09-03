use super::{
    ID_SOURCE,
    parameters::{From, Parameters},
    state::Settings,
};
use crate::{
    app::{
        computers::{
            DisplayFactorComputed, DisplayFactorKey, DisplayValueComputed, DisplayValueKey, Factor,
        },
        panes::MARGIN,
    },
    utils::Hashed,
};
use egui::{Context, Frame, Id, Margin, Response, TextStyle, TextWrapMode, Ui};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use itertools::Itertools;
use lipid::prelude::*;
use polars::prelude::*;
use std::ops::Range;
use tracing::instrument;

const IDENTIFIER: Range<usize> = 0..3;
const STEREOSPECIFIC_NUMBERS: Range<usize> = IDENTIFIER.end..IDENTIFIER.end + 4;
const FACTORS: Range<usize> = STEREOSPECIFIC_NUMBERS.end..STEREOSPECIFIC_NUMBERS.end + 2;
const LEN: usize = FACTORS.end;

const TOP: &[Range<usize>] = &[IDENTIFIER, STEREOSPECIFIC_NUMBERS, FACTORS];

/// Calculation table
pub(crate) struct TableView<'a> {
    data_frame: &'a Hashed<DataFrame>,
    parameters: &'a Parameters,
    settings: Settings,
}

impl<'a> TableView<'a> {
    pub(crate) fn new(
        ctx: &Context,
        data_frame: &'a Hashed<DataFrame>,
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
            (0, IDENTIFIER) => {
                ui.heading(ui.localize("Identifier.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("Identifier"));
                    });
            }
            (0, STEREOSPECIFIC_NUMBERS) => {
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
            (1, stereospecific_numbers::TAG) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=123"));
                    });
            }
            (1, stereospecific_numbers::DAG1223) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=1223"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=1223"));
                    });
            }
            (1, stereospecific_numbers::MAG2) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=2"));
                    });
            }
            (1, stereospecific_numbers::MAG1_3) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=13"));
                    });
            }
            (1, factors::EF) => {
                ui.heading(ui.localize("EnrichmentFactor.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("EnrichmentFactor"));
                    });
            }
            (1, factors::SF) => {
                ui.heading(ui.localize("SelectivityFactor.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("SelectivityFactor"));
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
                ui.label(label);
            }
            (row, identifier::FA) => {
                if let Some(fatty_acid) = self.data_frame.try_fatty_acid()?.delta()?.get(row) {
                    ui.label(fatty_acid);
                }
            }
            (row, stereospecific_numbers::TAG) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayValueComputed>()
                        .get(DisplayValueKey {
                            data_frame: self.data_frame,
                            expr: &col(STEREOSPECIFIC_NUMBERS123)
                                .struct_()
                                .field_by_name("Experimental"),
                            percent: self.settings.percent,
                        })
                });
                if let Some(value) = data_frame["Mean"].f64()?.get(row) {
                    let response = ui
                        .label(format!("{value:.0$}", self.settings.precision))
                        .on_hover_text(value.to_string());
                    if response.hovered() {
                        response
                            .standard_deviation(&data_frame, row)?
                            .repetitions(&data_frame, row)?;
                    }
                }
            }
            (row, stereospecific_numbers::DAG1223) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayValueComputed>()
                        .get(DisplayValueKey {
                            data_frame: self.data_frame,
                            expr: &col(STEREOSPECIFIC_NUMBERS12_23)
                                .struct_()
                                .field_by_name("Experimental"),
                            percent: self.settings.percent,
                        })
                });
                if let Some(value) = data_frame["Mean"].f64()?.get(row) {
                    let response = ui
                        .label(format!("{value:.0$}", self.settings.precision))
                        .on_hover_text(value.to_string());
                    if response.hovered() {
                        response
                            .standard_deviation(&data_frame, row)?
                            .repetitions(&data_frame, row)?;
                    }
                }
                // let experimental = self.data_frame[STEREOSPECIFIC_NUMBERS12_23]
                //     .struct_()?
                //     .field_by_name("Experimental")?;
                // let value = value_or_mean(&experimental, row)?;
                // FloatWidget::new(value)
                //     .percent(self.settings.percent)
                //     .precision(Some(self.settings.precision))
                //     .disable(self.parameters.from != From::Sn12_23)
                //     .hover(true)
                //     .show(ui)
                //     .response
                //     .try_on_hover_ui(|ui| -> PolarsResult<()> {
                //         let theoretical = self.data_frame[STEREOSPECIFIC_NUMBERS12_23]
                //             .struct_()?
                //             .field_by_name("Theoretical")?;
                //         let value = value_or_mean(&theoretical, row)?;
                //         ui.horizontal(|ui| {
                //             ui.label(ui.localize("Theoretical"));
                //             FloatWidget::new(value)
                //                 .percent(self.settings.percent)
                //                 .precision(Some(self.settings.precision))
                //                 .show(ui);
                //         });
                //         Ok(())
                //     })?;
            }
            (row, stereospecific_numbers::MAG2) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayValueComputed>()
                        .get(DisplayValueKey {
                            data_frame: self.data_frame,
                            expr: &col(STEREOSPECIFIC_NUMBERS2)
                                .struct_()
                                .field_by_name("Experimental"),
                            percent: self.settings.percent,
                        })
                });
                if let Some(value) = data_frame["Mean"].f64()?.get(row) {
                    let response = ui
                        .label(format!("{value:.0$}", self.settings.precision))
                        .on_hover_text(value.to_string());
                    if response.hovered() {
                        response
                            .standard_deviation(&data_frame, row)?
                            .repetitions(&data_frame, row)?;
                    }
                }
                // let experimental = self.data_frame[STEREOSPECIFIC_NUMBERS2]
                //     .struct_()?
                //     .field_by_name("Experimental")?;
                // let value = value_or_mean(&experimental, row)?;
                // FloatWidget::new(value)
                //     .percent(self.settings.percent)
                //     .precision(Some(self.settings.precision))
                //     .disable(self.parameters.from != From::Sn2)
                //     .hover(true)
                //     .show(ui)
                //     .response
                //     .try_on_hover_ui(|ui| -> PolarsResult<()> {
                //         let theoretical = self.data_frame[STEREOSPECIFIC_NUMBERS2]
                //             .struct_()?
                //             .field_by_name("Theoretical")?;
                //         let value = value_or_mean(&theoretical, row)?;
                //         ui.horizontal(|ui| {
                //             ui.label(ui.localize("Theoretical"));
                //             FloatWidget::new(value)
                //                 .percent(self.settings.percent)
                //                 .precision(Some(self.settings.precision))
                //                 .show(ui);
                //         });
                //         Ok(())
                //     })?;
            }
            (row, stereospecific_numbers::MAG1_3) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayValueComputed>()
                        .get(DisplayValueKey {
                            data_frame: self.data_frame,
                            expr: &match self.parameters.from {
                                From::Sn12_23 => col(STEREOSPECIFIC_NUMBERS13)
                                    .struct_()
                                    .field_by_name(STEREOSPECIFIC_NUMBERS12_23),
                                From::Sn2 => col(STEREOSPECIFIC_NUMBERS13)
                                    .struct_()
                                    .field_by_name(STEREOSPECIFIC_NUMBERS2),
                            },
                            percent: self.settings.percent,
                        })
                });
                if let Some(value) = data_frame["Mean"].f64()?.get(row) {
                    let response = ui
                        .label(format!("{value:.0$}", self.settings.precision))
                        .on_hover_text(value.to_string());
                    if response.hovered() {
                        response
                            .standard_deviation(&data_frame, row)?
                            .repetitions(&data_frame, row)?;
                    }
                }
                // let value = self.data_frame[STEREOSPECIFIC_NUMBERS13]
                //     .struct_()?
                //     .fields_as_series()[0]
                //     .f64()?
                //     .get(row);
                // FloatWidget::new(value)
                //     .percent(self.settings.percent)
                //     .precision(Some(self.settings.precision))
                //     .hover(true)
                //     .show(ui)
                //     .response
                //     .try_on_enabled_hover_ui(|ui| -> PolarsResult<()> {
                //         let value = self.data_frame[STEREOSPECIFIC_NUMBERS13]
                //             .struct_()?
                //             .fields_as_series()[1]
                //             .f64()?
                //             .get(row);
                //         ui.horizontal(|ui| {
                //             let text = match self.parameters.from {
                //                 From::Sn12_23 => "FromSn2",
                //                 From::Sn2 => "FromSn12_23",
                //             };
                //             ui.label(text);
                //             FloatWidget::new(value)
                //                 .percent(self.settings.percent)
                //                 .precision(Some(self.settings.precision))
                //                 .show(ui);
                //         });
                //         Ok(())
                //     })?;
            }
            (row, factors::EF) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayFactorComputed>()
                        .get(DisplayFactorKey {
                            data_frame: self.data_frame,
                            factor: Factor::Enrichment,
                            percent: self.settings.percent,
                        })
                });
                if let Some(value) = data_frame["Mean"].f64()?.get(row) {
                    let response = ui
                        .label(format!("{value:.0$}", self.settings.precision))
                        .on_hover_text(value.to_string());
                    if response.hovered() {
                        response
                            .standard_deviation(&data_frame, row)?
                            .repetitions(&data_frame, row)?
                            .calculation(&data_frame, row)?;
                    }
                }
            }
            (row, factors::SF) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayFactorComputed>()
                        .get(DisplayFactorKey {
                            data_frame: self.data_frame,
                            factor: Factor::Selectivity,
                            percent: self.settings.percent,
                        })
                });
                if let Some(value) = data_frame["Mean"].f64()?.get(row) {
                    let response = ui
                        .label(format!("{value:.0$}", self.settings.precision))
                        .on_hover_text(value.to_string());
                    if response.hovered() {
                        response
                            .standard_deviation(&data_frame, row)?
                            .repetitions(&data_frame, row)?
                            .calculation(&data_frame, row)?;
                    }
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        match column {
            stereospecific_numbers::TAG => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<DisplayValueComputed>()
                        .get(DisplayValueKey {
                            data_frame: self.data_frame,
                            expr: &col(STEREOSPECIFIC_NUMBERS123)
                                .struct_()
                                .field_by_name("Experimental"),
                            percent: self.settings.percent,
                        })
                });
                if let Some(mean) = data_frame["Mean"].f64()?.sum() {
                    let response = ui
                        .label(format!("{mean:.0$}", self.settings.precision))
                        .on_hover_text(mean.to_string());
                    if response.hovered() {
                        // response
                        //     .try_on_enabled_hover_ui(|ui| -> PolarsResult<()> {
                        //         // Theoretical
                        //         let theoretical = self.data_frame[STEREOSPECIFIC_NUMBERS123]
                        //             .struct_()?
                        //             .field_by_name("Theoretical")?
                        //             .struct_()?
                        //             .field_by_name("Mean")?
                        //             .f64()?
                        //             .sum();
                        //         ui.horizontal(|ui| {
                        //             ui.label("Theoretical");
                        //             FloatWidget::new(theoretical)
                        //                 .percent(self.settings.percent)
                        //                 .precision(Some(self.settings.precision))
                        //                 .show(ui);
                        //         });
                        //         Ok(())
                        //     })?
                        //     .on_hover_text("∑ TAG");
                    }
                }
                // FloatWidget::new(experimental)
                //     .percent(self.settings.percent)
                //     .precision(Some(self.settings.precision))
                //     .hover(true)
                //     .show(ui)
                //     .response
                //     .try_on_enabled_hover_ui(|ui| -> PolarsResult<()> {
                //         let theoretical = self.data_frame[STEREOSPECIFIC_NUMBERS123]
                //             .struct_()?
                //             .field_by_name("Theoretical")?
                //             .f64()?
                //             .sum();
                //         ui.horizontal(|ui| {
                //             ui.label("Theoretical");
                //             FloatWidget::new(theoretical)
                //                 .percent(self.settings.percent)
                //                 .precision(Some(self.settings.precision))
                //                 .show(ui);
                //         });
                //         Ok(())
                //     })?
                //     .on_hover_text("∑ TAG");
            }
            // stereospecific_numbers::DAG1223 => {
            //     let experimental = self.data_frame[STEREOSPECIFIC_NUMBERS12_23]
            //         .struct_()?
            //         .field_by_name("Experimental")?
            //         .f64()?
            //         .sum();
            //     FloatWidget::new(experimental)
            //         .percent(self.settings.percent)
            //         .precision(Some(self.settings.precision))
            //         .disable(self.parameters.from != From::Sn12_23)
            //         .hover(true)
            //         .show(ui)
            //         .response
            //         .try_on_hover_ui(|ui| -> PolarsResult<()> {
            //             let theoretical = self.data_frame[STEREOSPECIFIC_NUMBERS12_23]
            //                 .struct_()?
            //                 .field_by_name("Theoretical")?
            //                 .f64()?
            //                 .sum();
            //             ui.horizontal(|ui| {
            //                 ui.label("Theoretical");
            //                 FloatWidget::new(theoretical)
            //                     .percent(self.settings.percent)
            //                     .precision(Some(self.settings.precision))
            //                     .show(ui);
            //             });
            //             Ok(())
            //         })?
            //         .on_hover_text("∑ DAG1223");
            // }
            // stereospecific_numbers::MAG2 => {
            //     let experimental = self.data_frame[STEREOSPECIFIC_NUMBERS2]
            //         .struct_()?
            //         .field_by_name("Experimental")?
            //         .f64()?
            //         .sum();
            //     FloatWidget::new(experimental)
            //         .percent(self.settings.percent)
            //         .precision(Some(self.settings.precision))
            //         .disable(self.parameters.from != From::Sn2)
            //         .hover(true)
            //         .show(ui)
            //         .response
            //         .try_on_hover_ui(|ui| -> PolarsResult<()> {
            //             let theoretical = self.data_frame[STEREOSPECIFIC_NUMBERS2]
            //                 .struct_()?
            //                 .field_by_name("Theoretical")?
            //                 .f64()?
            //                 .sum();
            //             ui.horizontal(|ui| {
            //                 ui.label("Theoretical");
            //                 FloatWidget::new(theoretical)
            //                     .percent(self.settings.percent)
            //                     .precision(Some(self.settings.precision))
            //                     .show(ui);
            //             });
            //             Ok(())
            //         })?
            //         .on_hover_text("∑ MAG2");
            // }
            // stereospecific_numbers::MAG1_3 => {
            //     let value = self.data_frame[STEREOSPECIFIC_NUMBERS13]
            //         .struct_()?
            //         .fields_as_series()[0]
            //         .f64()?
            //         .sum();
            //     FloatWidget::new(value)
            //         .percent(self.settings.percent)
            //         .precision(Some(self.settings.precision))
            //         .hover(true)
            //         .show(ui)
            //         .response
            //         .try_on_enabled_hover_ui(|ui| -> PolarsResult<()> {
            //             let value = self.data_frame[STEREOSPECIFIC_NUMBERS13]
            //                 .struct_()?
            //                 .fields_as_series()[1]
            //                 .f64()?
            //                 .sum();
            //             ui.horizontal(|ui| {
            //                 let text = match self.parameters.from {
            //                     From::Sn12_23 => "FromSn2",
            //                     From::Sn2 => "FromSn12_23",
            //                 };
            //                 ui.label(text);
            //                 FloatWidget::new(value)
            //                     .percent(self.settings.percent)
            //                     .precision(Some(self.settings.precision))
            //                     .show(ui);
            //             });
            //             Ok(())
            //         })?
            //         .on_hover_text("∑ MAG1(3)");
            // }
            _ => {}
        }
        Ok(())
    }

    // fn value(
    //     &self,
    //     ui: &mut Ui,
    //     series: &Series,
    //     row: Option<usize>,
    //     percent: bool,
    //     disable: bool,
    // ) -> PolarsResult<Response> {
    //     let experimental = series.struct_()?.field_by_name("Experimental")?;
    //     Ok(if let Some(r#struct) = experimental.try_struct() {
    //         let mean = if let Some(row) = row {
    //             r#struct.field_by_name("Mean")?.f64()?.get(row)
    //         } else {
    //             r#struct.field_by_name("Mean")?.f64()?.sum()
    //         };
    //         let response = FloatWidget::new(mean)
    //             .percent(percent)
    //             .precision(Some(self.settings.precision))
    //             .disable(disable)
    //             .hover(true)
    //             .show(ui)
    //             .response;
    //         if let Some(row) = row {
    //             response.on_hover_text(r#struct.field_by_name("Values")?.str_value(row)?);
    //         }
    //         ui.label("±");
    //         let standard_deviation = if let Some(row) = row {
    //             r#struct.field_by_name("StandardDeviation")?.f64()?.get(row)
    //         } else {
    //             r#struct.field_by_name("StandardDeviation")?.f64()?.sum()
    //         };
    //         FloatWidget::new(standard_deviation)
    //             .percent(percent)
    //             .precision(Some(self.settings.precision))
    //             .disable(disable)
    //             .hover(true)
    //             .show(ui)
    //             .response
    //     } else {
    //         let values = experimental.f64()?;
    //         let value = if let Some(row) = row {
    //             values.get(row)
    //         } else {
    //             values.sum()
    //         };
    //         FloatWidget::new(value)
    //             .percent(percent)
    //             .precision(Some(self.settings.precision))
    //             .disable(disable)
    //             .hover(true)
    //             .show(ui)
    //             .response
    //     })
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
                let _ = self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }
}

/// Extension methods for [`Response`]
trait ResponseExt: Sized {
    fn calculation(self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self>;

    fn standard_deviation(self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self>;

    fn repetitions(self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self>;
}

impl ResponseExt for Response {
    fn calculation(mut self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self> {
        if let Some(calculation) = data_frame["Calculation"].str()?.get(row) {
            self = self.on_hover_text(calculation);
        }
        Ok(self)
    }

    fn standard_deviation(mut self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self> {
        if let Some(standard_deviation) = data_frame["StandardDeviation"].f64()?.get(row) {
            self = self.on_hover_text(format!("± {standard_deviation}"));
        }
        Ok(self)
    }

    fn repetitions(mut self, data_frame: &DataFrame, row: usize) -> PolarsResult<Self> {
        if let Some(repetitions) = data_frame["Repetitions"].array()?.get_as_series(row)
            && repetitions.len() > 1
        {
            let formated = repetitions.f64()?.iter().format_with(", ", |value, f| {
                if let Some(value) = value {
                    f(&value)?;
                }
                Ok(())
            });
            self = self.on_hover_text(format!("[{formated}]"));
        }
        Ok(self)
    }
}

mod identifier {
    use super::*;

    pub(super) const INDEX: Range<usize> = IDENTIFIER.start..IDENTIFIER.start + 1;
    pub(super) const LABEL: Range<usize> = INDEX.end..INDEX.end + 1;
    pub(super) const FA: Range<usize> = LABEL.end..LABEL.end + 1;
}

mod stereospecific_numbers {
    use super::*;

    pub(super) const TAG: Range<usize> =
        STEREOSPECIFIC_NUMBERS.start..STEREOSPECIFIC_NUMBERS.start + 1;
    pub(super) const DAG1223: Range<usize> = TAG.end..TAG.end + 1;
    pub(super) const MAG2: Range<usize> = DAG1223.end..DAG1223.end + 1;
    pub(super) const MAG1_3: Range<usize> = MAG2.end..MAG2.end + 1;
}

mod factors {
    use super::*;

    pub(super) const EF: Range<usize> = FACTORS.start..FACTORS.start + 1;
    pub(super) const SF: Range<usize> = EF.end..EF.end + 1;
}
