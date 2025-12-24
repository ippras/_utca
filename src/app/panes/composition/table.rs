use super::ID_SOURCE;
use crate::{
    app::{
        panes::MARGIN,
        states::composition::State,
        widgets::{
            FloatWidget,
            mean_and_standard_deviation::{MeanAndStandardDeviation, NewMeanAndStandardDeviation},
        },
    },
    r#const::{KEY, MEAN, SAMPLE, SPECIES, VALUE, VALUES},
    text::Text,
    utils::{HashedDataFrame, egui::ResponseExt as _},
};
use egui::{
    Context, Frame, Grid, Id, Label, Margin, Response, RichText, ScrollArea, TextStyle, Ui, Widget,
};
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
                    ui.heading(VALUE);
                }
            }
            (2, column) => {
                if column.start % 2 == 1 {
                    ui.heading(KEY);
                } else if column.start != 0 {
                    ui.heading(VALUE);
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

    // ┌─────────────────────────────┬───────────────┬──────────────────────┬─────────────────────────────┐
    // │ 0                           ┆ 1.Key         ┆ 1.Value              ┆ Species                     │
    // │ ---                         ┆ ---           ┆ ---                  ┆ ---                         │
    // │ struct[2]                   ┆ str           ┆ struct[3]            ┆ list[struct[3]]             │
    // ╞═════════════════════════════╪═══════════════╪══════════════════════╪═════════════════════════════╡
    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        match (row, column) {
            (row, top::INDEX) => {
                ui.horizontal(|ui| -> PolarsResult<()> {
                    ui.menu_button(LIST, |ui| self.list_button_content(ui, row))
                        .inner
                        .transpose()?;
                    ui.label(row.to_string());
                    Ok(())
                })
                .inner?;
            }
            (row, column) => {
                let index = (column.start + 1) / 2 - 1;
                let name = &*index.to_string();
                if !column.start.is_multiple_of(2) {
                    let key = self.data_frame[name].struct_()?.field_by_name(KEY)?;
                    let text = key.str_value(row)?;
                    Label::new(text).truncate().ui(ui);
                } else {
                    NewMeanAndStandardDeviation::new(
                        &self.data_frame[name].struct_()?.field_by_name(VALUE)?,
                        row,
                    )
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .show(ui)?;
                }
            }
        }
        Ok(())
    }

    fn list_button_content(&self, ui: &mut Ui, row: usize) -> PolarsResult<()> {
        let species_series = self.data_frame[SPECIES].as_materialized_series();
        if let Some(species) = species_series.list()?.get_as_series(row) {
            ui.heading(SPECIES).on_hover_text(species.len().to_string());
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
                                .zip(species.struct_()?.field_by_name(VALUE)?.f64()?)
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
            let last = &self.data_frame[self.data_frame.width() - 2];
            if let Some(text) = last
                .struct_()?
                .field_by_name(VALUE)?
                .struct_()?
                .field_by_name(MEAN)?
                .f64()?
                .sum()
            {
                ui.label(RichText::new(text.to_string()).strong());
            }
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
        if cell.row_nr.is_multiple_of(2) {
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

mod top {
    use super::*;

    pub(super) const INDEX: Range<usize> = 0..1;
}
