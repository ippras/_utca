use super::{
    ID_SOURCE, Settings, State,
    settings::{MMC, MSC, NMC, NSC, SMC, SPC, SSC, TMC, TPC, TSC, UMC, USC},
};
use crate::{
    app::{panes::MARGIN, widgets::FloatWidget},
    text::Text,
};
use egui::{Frame, Grid, Id, Margin, ScrollArea, TextStyle, Ui};
use egui_l20n::{ResponseExt as _, UiExt as _};
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use lipid::prelude::*;
use polars::prelude::*;
use polars_utils::format_list;
use std::{
    fmt::from_fn,
    ops::{Add, Range},
};
use tracing::instrument;

const INDEX: Range<usize> = 0..1;

/// Composition table
#[derive(Debug)]
pub(super) struct TableView<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
    state: &'a mut State,
    // is_row_expanded: BTreeMap<u64, bool>,
    // prefetched: Vec<PrefetchInfo>,
}

impl<'a> TableView<'a> {
    pub(crate) fn new(
        data_frame: &'a DataFrame,
        settings: &'a Settings,
        state: &'a mut State,
    ) -> Self {
        Self {
            data_frame,
            settings,
            state,
        }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.reset_table_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading);
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = self.settings.special.selections.len() * 2 + 1;
        let top = vec![0..1, 1..num_columns];
        let mut middle = vec![0..1];
        const STEP: usize = 2;
        for index in (1..num_columns).step_by(STEP) {
            middle.push(index..index + STEP);
        }
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default().resizable(self.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.settings.sticky_columns)
            .headers([
                HeaderRow {
                    height,
                    groups: top,
                },
                HeaderRow {
                    height,
                    groups: middle,
                },
                HeaderRow::new(height),
            ])
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        match (row, column) {
            (0, INDEX) => {
                ui.heading(HASH).on_hover_localized("index");
            }
            (0, _) => {
                ui.heading("Compositions");
            }
            (1, column) => {
                if column.start % 2 == 1 {
                    let index = column.start / 2;
                    let composition = self.settings.special.selections[index].composition;
                    ui.heading(ui.localize(composition.text()))
                        .on_hover_text(ui.localize(composition.hover_text()));
                } else if column.start != 0 {
                    ui.heading("Value");
                }
            }
            (2, column) => {
                if column.start % 2 == 1 {
                    ui.heading("Key");
                } else if column.start != 0 {
                    ui.heading("Value");
                }
            }
            _ => {}
        }
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
            (row, INDEX) => {
                ui.label(row.to_string());
            }
            (row, column) => {
                let index = (column.start + 1) / 2 - 1;
                if column.start % 2 == 1 {
                    let keys = self.data_frame["Keys"].struct_()?;
                    let key = &keys.fields_as_series()[index];
                    let response = match self.settings.special.selections[index].composition {
                        MMC | NMC | UMC => {
                            let text = Mono(key.str_value(row)?).to_string();
                            ui.label(text)
                        }
                        MSC | NSC | USC => {
                            let text = key
                                .try_triacylglycerol()?
                                .get_any_value(row)?
                                .stereo()
                                .to_string();
                            ui.label(text)
                        }
                        // TMC => {
                        //     match key.u32()?.get(row) {
                        //         Some(0) => ui.label("S3"),
                        //         Some(1) => ui.label("S2U"),
                        //         Some(2) => ui.label("SU2"),
                        //         Some(3) => ui.label("U3"),
                        //         _ => ui.label("None"),
                        //     };
                        // }
                        SMC | TMC => {
                            let text = key
                                .try_triacylglycerol()?
                                .get_any_value(row)?
                                .map(|any_value| any_value.str_value())
                                .mono()
                                .to_string();
                            ui.label(text)
                        }
                        SPC | TPC => {
                            let text = key
                                .try_triacylglycerol()?
                                .get_any_value(row)?
                                .map(|any_value| any_value.str_value())
                                .positional()
                                .to_string();
                            ui.label(text)
                        }
                        SSC | TSC => {
                            let text = key
                                .try_triacylglycerol()?
                                .get_any_value(row)?
                                .map(|any_value| any_value.str_value())
                                .stereo()
                                .to_string();
                            ui.label(text)
                        }
                    };
                    response.on_hover_ui(|ui| {
                        let species = self.data_frame["Species"].as_materialized_series();
                        let _ = self.species(species, row, ui);
                    });
                } else {
                    let values = self.data_frame["Values"].as_materialized_series();
                    self.value(ui, values, Some(row), index)?;
                }
            }
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        // Last column
        if column.start == self.settings.special.selections.len() * 2 {
            self.value(
                ui,
                self.data_frame["Values"].as_materialized_series(),
                None,
                self.settings.special.selections.len() - 1,
            )?;
        }
        Ok(())
    }

    fn value(
        &self,
        ui: &mut Ui,
        series: &Series,
        row: Option<usize>,
        index: usize,
    ) -> PolarsResult<()> {
        Ok(match series.dtype() {
            DataType::Array(inner, _) if inner.is_float() => {
                let value = if let Some(row) = row {
                    array_value(series, row, |array| Ok(array.f64()?.get(index)))?
                } else {
                    array_sum(series, |array| Ok(array.f64()?.get(index)))?
                };
                FloatWidget::new(value)
                    .percent(self.settings.percent)
                    .precision(Some(self.settings.precision))
                    .hover(true)
                    .show(ui);
            }
            DataType::Array(inner, _) if inner.is_struct() => {
                let value = if let Some(row) = row {
                    array_value(series, row, |array| {
                        Ok(array.struct_()?.field_by_name("Mean")?.f64()?.get(index))
                    })?
                } else {
                    array_sum(series, |array| {
                        Ok(array.struct_()?.field_by_name("Mean")?.f64()?.get(index))
                    })?
                };
                let response = FloatWidget::new(value)
                    .percent(self.settings.percent)
                    .precision(Some(self.settings.precision))
                    .hover(true)
                    .show(ui)
                    .response
                    .on_hover_ui(|ui| {
                        let _ = self.standard_deviation(series, row, index, ui);
                    });
                if let Some(row) = row {
                    response.on_hover_ui(|ui| {
                        let _ = self.repetitions(series, row, index, ui);
                    });
                }
            }
            data_type => panic!("value not implemented for {data_type:?}"),
        })
    }

    #[instrument(skip(self, series, ui), err)]
    fn species(&self, series: &Series, row: usize, ui: &mut Ui) -> PolarsResult<()> {
        let Some(species) = series.list()?.get_as_series(row) else {
            polars_bail!(NoData: r#"no "Species" list in row: {row}"#);
        };
        ui.heading("Species")
            .on_hover_text(species.len().to_string());
        ui.separator();
        ScrollArea::vertical()
            .auto_shrink(false)
            .max_height(ui.spacing().combo_height)
            .show(ui, |ui| {
                Grid::new(ui.next_auto_id())
                    .show(ui, |ui| -> PolarsResult<()> {
                        for (index, stereospecific_numbers) in species
                            .struct_()?
                            .field_by_name(LABEL)?
                            .try_triacylglycerol()?
                            .fields(|series| Ok(series.str()?.clone()))?
                            .iter()
                            .zip(species.struct_()?.field_by_name("Value")?.f64()?)
                            .enumerate()
                        {
                            ui.label(index.to_string());
                            let text = Triacylglycerol([
                                stereospecific_numbers.0.0,
                                stereospecific_numbers.0.1,
                                stereospecific_numbers.0.2,
                            ])
                            .map(|label| match label {
                                Some(label) => label,
                                None => "None",
                            })
                            .stereo()
                            .to_string();
                            ui.label(text);
                            let text = from_fn(|f| match stereospecific_numbers.1 {
                                Some(mut value) => {
                                    if self.settings.percent {
                                        value *= 100.0;
                                    }
                                    f.write_fmt(format_args!("{}", AnyValue::Float64(value)))
                                }
                                None => f.write_str("None"),
                            })
                            .to_string();
                            ui.label(text);
                            ui.end_row();
                        }
                        Ok(())
                    })
                    .inner
            })
            .inner
    }

    #[instrument(skip(self, series, ui), err)]
    fn standard_deviation(
        &self,
        series: &Series,
        row: Option<usize>,
        index: usize,
        ui: &mut Ui,
    ) -> PolarsResult<()> {
        let value = if let Some(row) = row {
            array_value(series, row, |array| {
                Ok(array
                    .struct_()?
                    .field_by_name("StandardDeviation")?
                    .f64()?
                    .get(index))
            })?
        } else {
            array_sum(series, |array| {
                Ok(array
                    .struct_()?
                    .field_by_name("StandardDeviation")?
                    .f64()?
                    .get(index))
            })?
        };
        ui.horizontal(|ui| {
            ui.label("±");
            FloatWidget::new(value)
                .percent(self.settings.percent)
                .show(ui);
        });
        Ok(())
    }

    #[instrument(skip(self, series, ui), err)]
    fn repetitions(
        &self,
        series: &Series,
        row: usize,
        index: usize,
        ui: &mut Ui,
    ) -> PolarsResult<()> {
        let Some(values) = series.array()?.get_as_series(row) else {
            polars_bail!(NoData: r#"no "Values" in row: {row}"#);
        };
        let Some(repetitions) = values
            .struct_()?
            .field_by_name("Repetitions")?
            .array()?
            .get_as_series(index)
        else {
            polars_bail!(NoData: r#"no "Repetitions" in index: {index}"#);
        };
        let text = format_list!(repetitions.f64()?.iter().map(|item| {
            from_fn(move |f| match item {
                Some(mut item) => {
                    if self.settings.percent {
                        item *= 100.0;
                    }
                    write!(f, "{}", AnyValue::Float64(item))
                }
                None => f.write_str("None"),
            })
        }));
        ui.label(text);
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

fn array_value<T>(
    series: &Series,
    row: usize,
    f: impl Fn(Series) -> PolarsResult<Option<T>>,
) -> PolarsResult<Option<T>> {
    let Some(values) = series.array()?.get_as_series(row) else {
        return Ok(None);
    };
    Ok(f(values)?)
}

fn array_sum<T: Add<Output = T>>(
    series: &Series,
    f: impl Fn(Series) -> PolarsResult<Option<T>>,
) -> PolarsResult<Option<T>> {
    let mut sum = None;
    let array = series.array()?;
    for row in 0..array.len() {
        if let Some(values) = array.get_as_series(row) {
            sum = match (sum, f(values)?) {
                (Some(sum), Some(value)) => Some(sum + value),
                (sum, value) => sum.or(value),
            };
        }
    }
    Ok(sum)
}
