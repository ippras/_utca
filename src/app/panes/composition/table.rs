use super::{ID_SOURCE, Settings, State};
use crate::{
    app::{ResultExt, panes::MARGIN, text::Text, widgets::FloatWidget},
    special::composition::{MNC, MSC, NNC, NSC, SNC, SPC, SSC, TNC, TPC, TSC, UNC, USC},
    utils::polars::{tag_map, r#type},
};
use egui::{Frame, Id, Margin, TextStyle, Ui};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::HASH;
use egui_table::{
    AutoSizeMode, CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState,
};
use polars::prelude::*;
use polars_ext::functions::round;
use std::ops::{Add, Range};

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
        let num_columns = self.settings.confirmed.selections.len() * 2 + 1;
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
            .auto_size_mode(AutoSizeMode::OnParentResize)
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        match (row, column) {
            (0, INDEX) => {
                ui.heading(HASH).on_hover_ui(|ui| {
                    ui.label(ui.localize("index"));
                });
            }
            (0, _) => {
                ui.heading("Compositions");
            }
            (1, column) => {
                if column.start % 2 == 1 {
                    let index = column.start / 2;
                    let composition = self.settings.confirmed.selections[index].composition;
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
                let indices = self.data_frame["Index"].u32()?;
                let index = indices.get(row).unwrap();
                ui.label(index.to_string());
            }
            (row, column) => {
                let index = (column.start + 1) / 2 - 1;
                if column.start % 2 == 1 {
                    let keys = self.data_frame["Keys"].struct_()?;
                    let key = &keys.fields_as_series()[index];
                    match self.settings.confirmed.selections[index].composition {
                        MNC => {
                            FloatWidget::new(|| Ok(key.f64()?.get(row)))
                                .precision(Some(self.settings.precision))
                                .hover()
                                .show(ui);
                        }
                        MSC => {
                            let key = tag_map(round(self.settings.precision as _))(key)?;
                            ui.label(key.str_value(row)?);
                        }
                        NNC | NSC => {
                            ui.label(key.str_value(row)?);
                        }
                        SNC | SPC | SSC => {
                            let sn1 = key
                                .struct_()?
                                .field_by_name("StereospecificNumber1")?
                                .str_value(row)?
                                .to_string();
                            let sn2 = key
                                .struct_()?
                                .field_by_name("StereospecificNumber2")?
                                .str_value(row)?
                                .to_string();
                            let sn3 = key
                                .struct_()?
                                .field_by_name("StereospecificNumber3")?
                                .str_value(row)?
                                .to_string();
                            ui.label(format!("{{{sn1},{sn2},{sn3}}}"));
                        }
                        TNC | TPC | TSC => {
                            let r#type = tag_map(r#type)(key)?;
                            let sn1 = r#type
                                .struct_()?
                                .field_by_name("StereospecificNumber1")?
                                .str_value(row)?
                                .to_string();
                            let sn2 = r#type
                                .struct_()?
                                .field_by_name("StereospecificNumber2")?
                                .str_value(row)?
                                .to_string();
                            let sn3 = r#type
                                .struct_()?
                                .field_by_name("StereospecificNumber3")?
                                .str_value(row)?
                                .to_string();
                            ui.label(format!("{{{sn1},{sn2},{sn3}}}"));
                        }
                        UNC | USC => {
                            ui.label(key.str_value(row)?);
                        }
                    }
                } else {
                    self.value(
                        ui,
                        self.data_frame["Values"].as_materialized_series(),
                        Some(row),
                        index,
                        self.settings.percent,
                    )?;
                }
            }
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        // Last column
        if column.start == self.settings.confirmed.selections.len() * 2 {
            self.value(
                ui,
                self.data_frame["Values"].as_materialized_series(),
                None,
                self.settings.confirmed.selections.len() - 1,
                self.settings.percent,
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
        percent: bool,
    ) -> PolarsResult<()> {
        Ok(match series.dtype() {
            DataType::Array(inner, _) if inner.is_float() => {
                FloatWidget::new(|| {
                    Ok(if let Some(row) = row {
                        array_value(series, row, |list| Ok(list.f64()?.get(index)))?
                    } else {
                        array_sum(series, |list| Ok(list.f64()?.get(index)))?
                    })
                })
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover()
                .show(ui);
            }
            DataType::Array(inner, _) if inner.is_struct() => {
                FloatWidget::new(|| {
                    Ok(if let Some(row) = row {
                        array_value(series, row, |list| {
                            Ok(list.struct_()?.field_by_name("Mean")?.f64()?.get(index))
                        })?
                    } else {
                        array_sum(series, |list| {
                            Ok(list.struct_()?.field_by_name("Mean")?.f64()?.get(index))
                        })?
                    })
                })
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover()
                .show(ui);
                ui.label("±");
                FloatWidget::new(|| {
                    Ok(if let Some(row) = row {
                        array_value(series, row, |list| {
                            Ok(list
                                .struct_()?
                                .field_by_name("StandardDeviation")?
                                .f64()?
                                .get(index))
                        })?
                    } else {
                        array_sum(series, |list| {
                            Ok(list
                                .struct_()?
                                .field_by_name("StandardDeviation")?
                                .f64()?
                                .get(index))
                        })?
                    })
                })
                .percent(percent)
                .precision(Some(self.settings.precision))
                .hover()
                .show(ui);
            }
            data_type => panic!("value not implemented for {data_type:?}"),
        })
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
                self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1)
                    .context(ui.ctx())
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
