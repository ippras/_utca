use super::ID_SOURCE;
use crate::{
    app::{
        computers::composition::table::{Computed as TableComputed, Key as TableKey},
        panes::MARGIN,
        states::composition::State,
        widgets::FloatWidget,
    },
    text::Text,
    utils::{HashedDataFrame, egui::ResponseExt as _},
};
use egui::{Context, Frame, Grid, Id, Label, Margin, Response, ScrollArea, TextStyle, Ui, Widget};
use egui_l20n::{ResponseExt as _, UiExt as _};
use egui_phosphor::regular::{HASH, LIST};
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use itertools::Itertools as _;
use lipid::prelude::*;
use polars::prelude::*;
use polars_utils::format_list;
use std::{
    fmt::from_fn,
    ops::{Add, Range},
};
use tracing::instrument;

/// Composition table
#[derive(Debug)]
pub(super) struct TableView<'a> {
    data_frame: &'a HashedDataFrame,
    state: &'a mut State,
    // is_row_expanded: BTreeMap<u64, bool>,
    // prefetched: Vec<PrefetchInfo>,
}

impl<'a> TableView<'a> {
    pub(crate) fn new(data_frame: &'a HashedDataFrame, state: &'a mut State) -> Self {
        Self { data_frame, state }
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
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = self.state.settings.selections.len() * 2 + 1;
        // let top = vec![0..1, 1..num_columns, num_columns..num_columns + 1];
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
                Column::default()
                    .resizable(self.state.settings.resizable);
                num_columns
            ])
            .num_sticky_cols(self.state.settings.sticky_columns)
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
            (0, top::INDEX) => {
                ui.heading(HASH).on_hover_localized("Index");
            }
            (0, _) => {
                ui.heading("Compositions");
            }
            (1, column) => {
                if column.start % 2 == 1 {
                    let index = column.start / 2;
                    let composition = self.state.settings.selections[index].composition;
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
            self.body_cell_content_ui(ui, row, column)
        } else {
            self.footer_cell_content_ui(ui, column)
        }
    }

    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        match (row, column) {
            (row, top::INDEX) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<TableComputed>()
                        .get(TableKey::new(self.data_frame, &self.state.settings))
                });
                ui.horizontal(|ui| -> PolarsResult<()> {
                    ui.menu_button(LIST, |ui| self.list_button_content(ui, &data_frame, row))
                        .inner
                        .transpose()?;
                    ui.label(row.to_string());
                    Ok(())
                })
                .inner?;
            }
            (row, column) => {
                let data_frame = ui.memory_mut(|memory| {
                    memory
                        .caches
                        .cache::<TableComputed>()
                        .get(TableKey::new(self.data_frame, &self.state.settings))
                });
                let index = (column.start + 1) / 2 - 1;
                let name = &*index.to_string();
                if column.start % 2 == 1 {
                    let key = data_frame[name].struct_()?.field_by_name("Key")?;
                    let text = key.str_value(row)?;
                    Label::new(text).truncate().ui(ui);
                    // ui.label(text);
                    // let keys = self.data_frame["Keys"].struct_()?;
                    // let key = &keys.fields_as_series()[index];
                    // let response = match self.state.settings.selections[index].composition {
                    //     ECN_MONO | MASS_MONO | UNSATURATION_MONO => {
                    //         let text = Mono(key.str_value(row)?).to_string();
                    //         ui.label(text)
                    //     }
                    //     ECN_STEREO | MASS_STEREO | UNSATURATION_STEREO => {
                    //         let text = key
                    //             .try_triacylglycerol()?
                    //             .get_any_value(row)?
                    //             .stereo()
                    //             .to_string();
                    //         ui.label(text)
                    //     }
                    //     // TMC => {
                    //     //     match key.u32()?.get(row) {
                    //     //         Some(0) => ui.label("S3"),
                    //     //         Some(1) => ui.label("S2U"),
                    //     //         Some(2) => ui.label("SU2"),
                    //     //         Some(3) => ui.label("U3"),
                    //     //         _ => ui.label("None"),
                    //     //     };
                    //     // }
                    //     SPECIES_MONO | TYPE_MONO => {
                    //         let text = key
                    //             .try_triacylglycerol()?
                    //             .get_any_value(row)?
                    //             .map(|any_value| any_value.str_value())
                    //             .mono()
                    //             .to_string();
                    //         ui.label(text)
                    //     }
                    //     SPECIES_POSITIONAL | TYPE_POSITIONAL => {
                    //         let text = key
                    //             .try_triacylglycerol()?
                    //             .get_any_value(row)?
                    //             .map(|any_value| any_value.str_value())
                    //             .positional()
                    //             .to_string();
                    //         ui.label(text)
                    //     }
                    //     SPECIES_STEREO | TYPE_STEREO => {
                    //         let text = key
                    //             .try_triacylglycerol()?
                    //             .get_any_value(row)?
                    //             .map(|any_value| any_value.str_value())
                    //             .stereo()
                    //             .to_string();
                    //         ui.label(text)
                    //     }
                    // };
                    // response.on_hover_ui(|ui| {
                    //     let species = self.data_frame["Species"].as_materialized_series();
                    //     _ = self.species(species, row, ui);
                    // });
                } else {
                    let value = data_frame[name].struct_()?.field_by_name("Value")?;
                    let mean = value.struct_()?.field_by_name("Mean")?;
                    if let Some(mean) = mean.f64()?.get(row) {
                        let response = ui
                            .label(format!("{mean:.0$}", self.state.settings.precision))
                            .on_hover_text(mean.to_string());
                        if response.hovered() {
                            response
                                .standard_deviation(&value, row)?
                                .array(&value, row)?;
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn list_button_content(
        &self,
        ui: &mut Ui,
        data_frame: &DataFrame,
        row: usize,
    ) -> PolarsResult<()> {
        let species_series = data_frame["Species"].as_materialized_series();
        if let Some(species) = species_series.list()?.get_as_series(row) {
            ui.heading("Species")
                .on_hover_text(species.len().to_string());
            ui.separator();
            ScrollArea::vertical()
                .auto_shrink([false, true])
                .max_height(ui.spacing().combo_height)
                .show(ui, |ui| {
                    Grid::new(ui.next_auto_id())
                        .show(ui, |ui| -> PolarsResult<()> {
                            for (index, (label, value)) in species
                                .struct_()?
                                .field_by_name(LABEL)?
                                .str()?
                                .iter()
                                .zip(species.struct_()?.field_by_name("Value")?.f64()?)
                                .enumerate()
                            {
                                ui.label(index.to_string());
                                if let Some(label) = label {
                                    ui.label(label);
                                }
                                if let Some(value) = value {
                                    ui.label(value.to_string());
                                }
                                ui.end_row();
                            }
                            Ok(())
                        })
                        .inner
                })
                .inner?;
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        // Last column
        if column.start == self.state.settings.selections.len() * 2 {
            self.value(
                ui,
                self.data_frame["Values"].as_materialized_series(),
                None,
                self.state.settings.selections.len() - 1,
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
            .percent(self.state.settings.percent)
            .precision(Some(self.state.settings.precision))
            .hover(true)
            .show(ui)
            .response
            .on_hover_ui(|ui| {
                _ = self.standard_deviation(series, row, index, ui);
            });
        if let Some(row) = row {
            response.on_hover_ui(|ui| {
                _ = self.array(series, row, index, ui);
            });
        }
        Ok(())
        // Ok(match series.dtype() {
        //     DataType::Array(inner, _) if inner.is_float() => {
        //         let value = if let Some(row) = row {
        //             array_value(series, row, |array| Ok(array.f64()?.get(index)))?
        //         } else {
        //             array_sum(series, |array| Ok(array.f64()?.get(index)))?
        //         };
        //         FloatWidget::new(value)
        //             .percent(self.state.settings.percent)
        //             .precision(Some(self.state.settings.precision))
        //             .hover(true)
        //             .show(ui);
        //     }
        //     DataType::Array(inner, _) if inner.is_struct() => {
        //         let value = if let Some(row) = row {
        //             array_value(series, row, |array| {
        //                 Ok(array.struct_()?.field_by_name("Mean")?.f64()?.get(index))
        //             })?
        //         } else {
        //             array_sum(series, |array| {
        //                 Ok(array.struct_()?.field_by_name("Mean")?.f64()?.get(index))
        //             })?
        //         };
        //         let response = FloatWidget::new(value)
        //             .percent(self.state.settings.percent)
        //             .precision(Some(self.state.settings.precision))
        //             .hover(true)
        //             .show(ui)
        //             .response
        //             .on_hover_ui(|ui| {
        //                 _ = self.standard_deviation(series, row, index, ui);
        //             });
        //         if let Some(row) = row {
        //             response.on_hover_ui(|ui| {
        //                 _ = self.array(series, row, index, ui);
        //             });
        //         }
        //     }
        //     data_type => panic!("value not implemented for {data_type:?}"),
        // })
    }

    // #[instrument(skip(self, series, ui), err)]
    // fn species(&self, series: &Series, row: usize, ui: &mut Ui) -> PolarsResult<()> {
    //     let Some(species) = series.list()?.get_as_series(row) else {
    //         polars_bail!(NoData: r#"no "Species" list in row: {row}"#);
    //     };
    //     ui.heading("Species")
    //         .on_hover_text(species.len().to_string());
    //     ui.separator();
    //     ScrollArea::vertical()
    //         .auto_shrink(false)
    //         .max_height(ui.spacing().combo_height)
    //         .show(ui, |ui| {
    //             Grid::new(ui.next_auto_id())
    //                 .show(ui, |ui| -> PolarsResult<()> {
    //                     for (index, stereospecific_numbers) in species
    //                         .struct_()?
    //                         .field_by_name(LABEL)?
    //                         .try_triacylglycerol()?
    //                         .fields(|series| Ok(series.str()?.clone()))?
    //                         .iter()
    //                         .zip(species.struct_()?.field_by_name("Value")?.f64()?)
    //                         .enumerate()
    //                     {
    //                         ui.label(index.to_string());
    //                         let text = Triacylglycerol([
    //                             stereospecific_numbers.0.0,
    //                             stereospecific_numbers.0.1,
    //                             stereospecific_numbers.0.2,
    //                         ])
    //                         .map(|label| match label {
    //                             Some(label) => label,
    //                             None => "None",
    //                         })
    //                         .stereo()
    //                         .to_string();
    //                         ui.label(text);
    //                         let text = from_fn(|f| match stereospecific_numbers.1 {
    //                             Some(mut value) => {
    //                                 if self.state.settings.percent {
    //                                     value *= 100.0;
    //                                 }
    //                                 f.write_fmt(format_args!("{}", AnyValue::Float64(value)))
    //                             }
    //                             None => f.write_str("None"),
    //                         })
    //                         .to_string();
    //                         ui.label(text);
    //                         ui.end_row();
    //                     }
    //                     Ok(())
    //                 })
    //                 .inner
    //         })
    //         .inner
    // }

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
                .percent(self.state.settings.percent)
                .show(ui);
        });
        Ok(())
    }

    #[instrument(skip(self, series, ui), err)]
    fn array(&self, series: &Series, row: usize, index: usize, ui: &mut Ui) -> PolarsResult<()> {
        let Some(values) = series.array()?.get_as_series(row) else {
            polars_bail!(NoData: r#"no "Values" in row: {row}"#);
        };
        let Some(array) = values
            .struct_()?
            .field_by_name("Array")?
            .array()?
            .get_as_series(index)
        else {
            polars_bail!(NoData: r#"no "Array" in index: {index}"#);
        };
        let text = format_list!(array.f64()?.iter().map(|item| {
            from_fn(move |f| match item {
                Some(mut item) => {
                    if self.state.settings.percent {
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
                _ = self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }

    fn row_top_offset(&self, ctx: &Context, _table_id: Id, row_nr: u64) -> f32 {
        row_nr as f32 * (ctx.style().spacing.interact_size.y + 2.0 * MARGIN.y)
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

/// Extension methods for [`Response`]
trait ResponseExt: Sized {
    fn species(self, value: &Series, row: usize) -> PolarsResult<Self>;

    fn standard_deviation(self, value: &Series, row: usize) -> PolarsResult<Self>;

    fn array(self, value: &Series, row: usize) -> PolarsResult<Self>;
}

impl ResponseExt for Response {
    fn species(mut self, species: &Series, row: usize) -> PolarsResult<Self> {
        if let Some(species) = species.list()?.get_as_series(row) {
            self = self.try_on_enabled_hover_ui(|ui| -> PolarsResult<()> {
                ui.heading("Species")
                    .on_hover_text(species.len().to_string());
                ui.separator();
                ScrollArea::vertical()
                    .auto_shrink([false, true])
                    .max_height(ui.spacing().combo_height)
                    .show(ui, |ui| {
                        Grid::new(ui.next_auto_id())
                            .show(ui, |ui| -> PolarsResult<()> {
                                for (index, (label, value)) in species
                                    .struct_()?
                                    .field_by_name(LABEL)?
                                    .str()?
                                    .iter()
                                    .zip(species.struct_()?.field_by_name("Value")?.f64()?)
                                    .enumerate()
                                {
                                    ui.label(index.to_string());
                                    if let Some(label) = label {
                                        ui.label(label);
                                    }
                                    if let Some(value) = value {
                                        ui.label(value.to_string());
                                    }
                                    ui.end_row();
                                }
                                Ok(())
                            })
                            .inner
                    })
                    .inner?;
                Ok(())
            })?;
        }
        Ok(self)
    }

    fn standard_deviation(mut self, value: &Series, row: usize) -> PolarsResult<Self> {
        if let Some(standard_deviation) = value
            .struct_()?
            .field_by_name("StandardDeviation")?
            .f64()?
            .get(row)
        {
            self = self.on_hover_text(format!("± {standard_deviation}"));
        }
        Ok(self)
    }

    fn array(mut self, value: &Series, row: usize) -> PolarsResult<Self> {
        if let Some(array) = value
            .struct_()?
            .field_by_name("Array")?
            .array()?
            .get_as_series(row)
            && array.len() > 1
        {
            let formated = array.f64()?.iter().format_with(", ", |value, f| {
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

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
}
