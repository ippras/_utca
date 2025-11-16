use super::ID_SOURCE;
use crate::{
    app::{
        computers::configuration::display::{
            Computed as ConfigurationDisplayComputed, Key as ConfigurationDisplayKey,
        },
        panes::MARGIN,
        states::configuration::State,
        widgets::{FattyAcidWidget, FloatWidget, Inner, LabelWidget},
    },
    utils::HashedDataFrame,
};
use anyhow::Result;
use egui::{Context, Event, Frame, Id, Margin, TextStyle, TextWrapMode, Ui};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::{ARROW_FAT_UP, HASH, MINUS, PLUS};
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use lipid::prelude::*;
use polars::{chunked_array::builder::AnonymousOwnedListBuilder, prelude::*};
use polars_ext::prelude::DataFrameExt as _;
use std::ops::Range;
use tracing::instrument;

const INDEX: Range<usize> = 0..1;
const LABEL: Range<usize> = INDEX.end..INDEX.end + 1;
const FA: Range<usize> = LABEL.end..LABEL.end + 1;
const SN123: Range<usize> = FA.end..FA.end + 1;
const SN2_OR_SN1223: Range<usize> = SN123.end..SN123.end + 1;
const LEN: usize = SN2_OR_SN1223.end;

/// Table view
pub(super) struct TableView<'a> {
    data: &'a mut HashedDataFrame,
    state: &'a mut State,
}

impl<'a> TableView<'a> {
    pub(super) fn new(data_frame: &'a mut HashedDataFrame, state: &'a mut State) -> Self {
        Self {
            data: data_frame,
            state,
        }
    }
}

impl TableView<'_> {
    pub(super) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.reset_table {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.reset_table = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data.height() as u64 + 1;
        let num_columns = LEN;
        Table::new()
            .id_salt(id_salt)
            .num_rows(num_rows)
            .columns(vec![
                Column::default()
                    .resizable(self.state.settings.resize_table);
                num_columns
            ])
            .num_sticky_cols(self.state.settings.sticky_columns)
            .headers([HeaderRow::new(height)])
            .show(ui, self);
        let _ = self.change();
    }

    #[instrument(skip(self), err)]
    fn change(&mut self) -> PolarsResult<()> {
        if self.state.add_row {
            self.data.add_row()?;
            self.data.rehash()?;
            self.state.add_row = false;
        }
        if let Some(index) = self.state.delete_row {
            self.data.delete_row(index)?;
            self.data.rehash()?;
            self.state.delete_row = None;
        }
        if let Some(index) = self.state.row_up {
            let indices = (0..index - 1)
                .chain(Some(index))
                .chain(Some(index - 1))
                .chain(index + 1..self.data.height())
                .map(|index| index as u32);
            self.data.data_frame = self
                .data
                .data_frame
                .clone()
                .lazy()
                .with_columns([lit(Series::from_iter(indices)).alias("Index")])
                .sort_by_exprs([col("Index")], Default::default())
                .drop(cols(["Index"]))
                .collect()?;
            self.data.rehash()?;
            self.state.row_up = None;
        }
        Ok(())
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.state.settings.truncate_headers {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            (0, INDEX) => {
                ui.heading(HASH).on_hover_ui(|ui| {
                    ui.label(ui.localize("Index"));
                });
            }
            (0, LABEL) => {
                let response = ui.heading(ui.localize("Label"));
                if self.state.settings.edit_table && response.hovered() {
                    ui.ctx().input(|input| {
                        for event in &input.raw.events {
                            if let Event::Paste(text) = event {
                                let _ = self.paste_labels("Label", text);
                            }
                        }
                    });
                };
            }
            (0, FA) => {
                ui.heading(ui.localize("FattyAcid.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("FattyAcid"));
                    });
            }
            (0, SN2_OR_SN1223) => {
                let name = self.data.get_columns()[3].name();
                if name == STEREOSPECIFIC_NUMBERS2 {
                    let response = ui
                        .heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("StereospecificNumber?number=2"));
                        });
                    if self.state.settings.edit_table && response.hovered() {
                        ui.ctx().input(|input| {
                            for event in &input.raw.events {
                                if let Event::Paste(text) = event {
                                    let _ = self.paste_floats(STEREOSPECIFIC_NUMBERS2, text);
                                }
                            }
                        });
                    };
                } else if name == STEREOSPECIFIC_NUMBERS12_23 {
                    let response = ui
                        .heading(ui.localize("StereospecificNumber.abbreviation?number=1223"))
                        .on_hover_ui(|ui| {
                            ui.label(ui.localize("StereospecificNumber?number=1223"));
                        });
                    if self.state.settings.edit_table && response.hovered() {
                        ui.ctx().input(|input| {
                            for event in &input.raw.events {
                                if let Event::Paste(text) = event {
                                    let _ = self.paste_floats(STEREOSPECIFIC_NUMBERS12_23, text);
                                }
                            }
                        });
                    };
                };
            }
            (0, SN123) => {
                let response = ui
                    .heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=123"));
                    });
                if self.state.settings.edit_table && response.hovered() {
                    ui.ctx().input(|input| {
                        for event in &input.raw.events {
                            if let Event::Paste(text) = event {
                                let _ = self.paste_floats(STEREOSPECIFIC_NUMBERS123, text);
                            }
                        }
                    });
                };
            }
            _ => {}
        };
    }

    #[instrument(skip(self), err)]
    fn paste_labels(&mut self, column: &'static str, text: &str) -> Result<()> {
        let mut builder =
            StringChunkedBuilder::new(PlSmallStr::from_static(column), self.data.height());
        for row in text.split('\n') {
            builder.append_value(row.trim());
        }
        self.data.replace(column, builder.finish())?;
        self.data.rehash()?;
        Ok(())
    }

    #[instrument(skip(self), err)]
    fn paste_floats(&mut self, column: &'static str, text: &str) -> Result<()> {
        let mut builder = PrimitiveChunkedBuilder::<Float64Type>::new(
            PlSmallStr::from_static(column),
            self.data.height(),
        );
        for row in text.split('\n') {
            builder.append_value(row.trim().parse()?);
        }
        self.data.replace(column, builder.finish())?;
        self.data.rehash()?;
        Ok(())
    }

    #[instrument(skip(self, ui), err)]
    fn cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        if row != self.data.height() {
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
                if self.state.settings.edit_table {
                    if ui.button(MINUS).clicked() {
                        self.state.delete_row = Some(row);
                    }
                    if ui.button(ARROW_FAT_UP).clicked() {
                        self.state.row_up = Some(row);
                    }
                }
                ui.label(row.to_string());
            }
            (row, LABEL) => {
                let fatty_acid = self.data.try_fatty_acid()?;
                let label = self.data["Label"].str()?;
                let inner_response = LabelWidget::new(label, fatty_acid, row)
                    .editable(self.state.settings.edit_table)
                    .hover_names(self.state.settings.hover_names)
                    .show(ui);
                if inner_response.response.changed() {
                    match inner_response.inner? {
                        Some(Inner::Cell(new)) => {
                            self.data.try_apply("Label", change_label(row, &new))?;
                            self.data.rehash()?;
                        }
                        Some(Inner::Column(new_col)) => {
                            self.data.replace("Label", new_col)?;
                            self.data.rehash()?;
                        }
                        None => todo!(),
                    }
                }
            }
            (row, FA) => {
                let fatty_acid = self.data.try_fatty_acid()?.get(row)?;
                let inner_response = FattyAcidWidget::new(fatty_acid.as_ref())
                    .editable(self.state.settings.edit_table)
                    .hover(true)
                    .show(ui);
                if inner_response.response.changed() {
                    self.data
                        .try_apply("FattyAcid", change_fatty_acid(row, inner_response.inner))?;
                    self.data.rehash()?;
                }
            }
            (row, SN123) => {
                let data_frame = ui.memory_mut(|memory| -> PolarsResult<_> {
                    Ok(memory
                        .caches
                        .cache::<ConfigurationDisplayComputed>()
                        .get(ConfigurationDisplayKey { frame: &self.data }))
                })?;
                let value = data_frame[STEREOSPECIFIC_NUMBERS123].f64()?.get(row);
                let inner_response = FloatWidget::new(value)
                    .editable(self.state.settings.edit_table)
                    .precision(Some(self.state.settings.float_precision))
                    .hover(true)
                    .show(ui);
                if let Some(value) = inner_response.inner {
                    self.data
                        .try_apply(STEREOSPECIFIC_NUMBERS123, change_f64(row, value))?;
                    self.data.rehash()?;
                }
            }
            (row, SN2_OR_SN1223) => {
                let data_frame = ui.memory_mut(|memory| -> PolarsResult<_> {
                    Ok(memory
                        .caches
                        .cache::<ConfigurationDisplayComputed>()
                        .get(ConfigurationDisplayKey { frame: &self.data }))
                })?;
                let name = data_frame.get_columns()[3].name().as_str();
                let value = data_frame[name].f64()?.get(row);
                let inner_response = FloatWidget::new(value)
                    .editable(self.state.settings.edit_table)
                    .precision(Some(self.state.settings.float_precision))
                    .hover(true)
                    .show(ui);
                if let Some(value) = inner_response.inner {
                    self.data.try_apply(name, change_f64(row, value))?;
                    self.data.rehash()?;
                }
            }
            _ => {}
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        match column {
            INDEX => {
                if self.state.settings.edit_table {
                    if ui.button(PLUS).clicked() {
                        self.state.add_row = true;
                    }
                }
            }
            // TAG => {
            //     FloatWidget::new(self.data_frame["Triacylglycerol"].f64()?.sum())
            //         .precision(Some(self.state.settings.precision))
            //         .hover(true)
            //         .show(ui)
            //         .response
            //         .on_hover_text("∑ TAG");
            // }
            // DAG1223 => {
            //     FloatWidget::new(self.data_frame["Diacylglycerol1223"].f64()?.sum())
            //         .precision(Some(self.state.settings.precision))
            //         .hover(true)
            //         .show(ui)
            //         .response
            //         .on_hover_text("∑ DAG1223");
            // }
            // MAG2 => {
            //     FloatWidget::new(self.data_frame["Monoacylglycerol2"].f64()?.sum())
            //         .precision(Some(self.state.settings.precision))
            //         .hover(true)
            //         .show(ui)
            //         .response
            //         .on_hover_text("∑ MAG");
            // }
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

    fn row_top_offset(&self, ctx: &Context, _table_id: Id, row_nr: u64) -> f32 {
        row_nr as f32 * (ctx.style().spacing.interact_size.y + 2.0 * MARGIN.y)
    }
}

fn change_fatty_acid(
    row: usize,
    value: Option<FattyAcid>,
) -> impl FnMut(&Series) -> PolarsResult<Series> {
    move |series| {
        // Ok(series.clone())
        let fatty_acid = series.try_fatty_acid()?;
        let carbon = fatty_acid.carbon()?;
        let mut carbon_builder =
            PrimitiveChunkedBuilder::<UInt8Type>::new(carbon.name().clone(), carbon.len());
        let indices = fatty_acid.indices()?;
        let mut indices_builder = AnonymousOwnedListBuilder::new(
            indices.name().clone(),
            indices.len(),
            indices.dtype().inner_dtype().cloned(),
        );
        for index in 0..series.len() {
            let mut fatty_acid = fatty_acid.get(index)?;
            if index == row {
                fatty_acid = value.clone();
            }
            let fatty_acid = fatty_acid.as_ref();
            // Carbons
            carbon_builder.append_option(fatty_acid.map(|fatty_acid| fatty_acid.carbon));
            // Unsaturated
            if let Some(fatty_acid) = fatty_acid {
                let mut index_builder = PrimitiveChunkedBuilder::<UInt8Type>::new(
                    "Index".into(),
                    fatty_acid.unsaturated.len(),
                );
                let mut triple_builder =
                    BooleanChunkedBuilder::new("Triple".into(), fatty_acid.unsaturated.len());
                let mut parity_builder =
                    BooleanChunkedBuilder::new("Parity".into(), fatty_acid.unsaturated.len());
                for unsaturated in &fatty_acid.unsaturated {
                    index_builder.append_option(unsaturated.index);
                    triple_builder.append_option(unsaturated.triple);
                    parity_builder.append_option(unsaturated.parity);
                }
                indices_builder.append_series(
                    &StructChunked::from_series(
                        PlSmallStr::EMPTY,
                        fatty_acid.unsaturated.len(),
                        [
                            index_builder.finish().into_series(),
                            triple_builder.finish().into_series(),
                            parity_builder.finish().into_series(),
                        ]
                        .iter(),
                    )?
                    .into_series(),
                )?;
            } else {
                indices_builder.append_opt_series(None)?;
            }
        }
        Ok(StructChunked::from_series(
            series.name().clone(),
            series.len(),
            [
                carbon_builder.finish().into_series(),
                indices_builder.finish().into_series(),
            ]
            .iter(),
        )?
        .into_series())
    }
}

fn change_f64(row: usize, value: Option<f64>) -> impl FnMut(&Series) -> PolarsResult<Series> {
    move |series| {
        Ok(series
            .f64()?
            .iter()
            .enumerate()
            .map(|(index, current)| Ok(if index == row { value } else { current }))
            .collect::<PolarsResult<Float64Chunked>>()?
            .into_series())
    }
}

fn change_label(row: usize, new: &str) -> impl FnMut(&Series) -> PolarsResult<Series> {
    move |series| {
        Ok(series
            .str()?
            .iter()
            .enumerate()
            .map(|(index, mut value)| {
                if index == row {
                    value = Some(new);
                }
                Ok(value)
            })
            .collect::<PolarsResult<StringChunked>>()?
            .into_series())
    }
}

// fn change_fatty_acid(
//     row: usize,
//     new: &FattyAcid,
// ) -> impl FnMut(&Series) -> PolarsResult<Series> + '_ {
//     move |series| {
//         let fatty_acid_series = series.fa();
//         let mut carbons = PrimitiveChunkedBuilder::<UInt8Type>::new(
//             fatty_acid_series.carbons.name().clone(),
//             fatty_acid_series.len(),
//         );
//         let mut unsaturated = AnonymousOwnedListBuilder::new(
//             fatty_acid_series.unsaturated.name().clone(),
//             fatty_acid_series.len(),
//             fatty_acid_series.unsaturated.dtype().inner_dtype().cloned(),
//         );
//         for index in 0..fatty_acid_series.len() {
//             let mut fatty_acid = fatty_acid_series.get(index)?;
//             if index == row {
//                 fatty_acid = Some(new.clone());
//             }
//             let fatty_acid = fatty_acid.as_ref();
//             // Carbons
//             carbons.append_option(fatty_acid.map(|fatty_acid| fatty_acid.carbons));
//             // Unsaturated
//             if let Some(fatty_acid) = fatty_acid {
//                 // let mut fields = Vec::with_capacity(fatty_acid.unsaturated.len());
//                 // if let Some(unsaturated_series) = fatty_acid_series.unsaturated(index)? {
//                 //     fields.push(unsaturated_series.index);
//                 //     fields.push(unsaturated_series.isomerism);
//                 //     fields.push(unsaturated_series.unsaturation);
//                 // }
//                 // unsaturated.append_series(
//                 //     &StructChunked::from_series(
//                 //         PlSmallStr::EMPTY,
//                 //         fatty_acid.unsaturated.len(),
//                 //         fields.iter(),
//                 //     )?
//                 //     .into_series(),
//                 // )?;
//                 let mut index = PrimitiveChunkedBuilder::<UInt8Type>::new(
//                     "Index".into(),
//                     fatty_acid.unsaturated.len(),
//                 );
//                 let mut isomerism = PrimitiveChunkedBuilder::<Int8Type>::new(
//                     "Isomerism".into(),
//                     fatty_acid.unsaturated.len(),
//                 );
//                 let mut unsaturation = PrimitiveChunkedBuilder::<UInt8Type>::new(
//                     "Unsaturation".into(),
//                     fatty_acid.unsaturated.len(),
//                 );
//                 for unsaturated in &fatty_acid.unsaturated {
//                     index.append_option(unsaturated.index);
//                     isomerism.append_option(unsaturated.isomerism.map(|isomerism| isomerism as _));
//                     unsaturation.append_option(
//                         unsaturated
//                             .unsaturation
//                             .map(|unsaturation| unsaturation as _),
//                     );
//                 }
//                 unsaturated.append_series(
//                     &StructChunked::from_series(
//                         PlSmallStr::EMPTY,
//                         fatty_acid.unsaturated.len(),
//                         [
//                             index.finish().into_series(),
//                             isomerism.finish().into_series(),
//                             unsaturation.finish().into_series(),
//                         ]
//                         .iter(),
//                     )?
//                     .into_series(),
//                 )?;
//             } else {
//                 println!("HERE1");
//                 unsaturated.append_opt_series(None)?;
//             }
//         }
//         Ok(StructChunked::from_series(
//             series.name().clone(),
//             fatty_acid_series.len(),
//             [
//                 carbons.finish().into_series(),
//                 unsaturated.finish().into_series(),
//             ]
//             .iter(),
//         )?
//         .into_series())
//     }
// }

// fn change_experimental(row: usize, new: f64) -> impl FnMut(&Series) -> PolarsResult<Series> {
//     move |series| {
//         Ok(series
//             .f64()?
//             .iter()
//             .enumerate()
//             .map(|(index, mut value)| {
//                 if index == row {
//                     value = Some(new);
//                 }
//                 Ok(value)
//             })
//             .collect::<PolarsResult<Float64Chunked>>()?
//             .into_series())
//     }
// }
