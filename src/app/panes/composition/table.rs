use super::ID_SOURCE;
use crate::{
    app::{
        panes::MARGIN, states::composition::State,
        widgets::mean_and_standard_deviation::NewMeanAndStandardDeviation,
    },
    r#const::{KEY, SPECIES, VALUE},
    text::Text,
    utils::HashedDataFrame,
};
use egui::{
    Context, Frame, Grid, Id, Label, Margin, PopupCloseBehavior, ScrollArea, TextStyle, Ui, Widget,
    containers::menu::{MenuButton, MenuConfig},
};
use egui_ext::InnerResponseExt as _;
use egui_l20n::prelude::*;
use egui_phosphor::regular::{HASH, LIST};
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use lipid::prelude::*;
use polars::prelude::*;
use std::ops::Range;
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
        let num_rows = self.data_frame.height() as _;
        let num_columns = self.data_frame.width();
        let top = vec![0..1, 1..num_columns - 1, num_columns - 1..num_columns];
        let mut middle = vec![0..1];
        const STEP: usize = 2;
        for index in (1..num_columns - 1).step_by(STEP) {
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
            (0, column) if column.start + 1 < self.data_frame.width() => {
                ui.heading(ui.localize("Composition?PluralCategory=other"));
            }
            (0, _last) => {
                ui.heading(ui.localize("Species"));
            }
            (1, column) if column.start != 0 => {
                let index = column.start / 2;
                let composition = self.state.settings.compositions[index];
                ui.heading(ui.localize(composition.text()))
                    .on_hover_text(ui.localize(composition.hover_text()));
            }
            (2, column) if column.start + 1 < self.data_frame.width() => {
                if !column.start.is_multiple_of(2) {
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
        self.body_cell_content_ui(ui, row, column)
        // if row != self.data_frame.height() {
        //     self.body_cell_content_ui(ui, row, column)
        // } else {
        //     self.footer_cell_content_ui(ui, column)
        // }
    }

    fn body_cell_content_ui(
        &mut self,
        ui: &mut Ui,
        row: usize,
        column: Range<usize>,
    ) -> PolarsResult<()> {
        match (row, column) {
            (row, top::INDEX) => {
                if let Some(text) = self.data_frame[INDEX].idx()?.get(row) {
                    ui.label(text.to_string());
                }
            }
            (row, column) if column.start + 1 < self.data_frame.width() => {
                // let index = (column.start + 1) / 2 - 1;
                if !column.start.is_multiple_of(2) {
                    let key = self.data_frame[column.start].str()?.get(row);
                    if let Some(text) = key {
                        Label::new(text).truncate().ui(ui);
                    }
                } else {
                    let value = self.data_frame[column.start].as_materialized_series();
                    NewMeanAndStandardDeviation::new(&value, row)
                        .with_standard_deviation(self.state.settings.standard_deviation)
                        .with_sample(true)
                        .show(ui)?;
                }
            }
            (row, _last) => {
                self.list_button(ui, row)?;
            }
        }
        Ok(())
    }

    fn list_button(&self, ui: &mut Ui, row: usize) -> PolarsResult<()> {
        let species_series = self.data_frame[SPECIES].as_materialized_series();
        if let Some(species) = species_series.list()?.get_as_series(row) {
            let (_, inner_response) = MenuButton::new(LIST)
                .config(MenuConfig::new().close_behavior(PopupCloseBehavior::CloseOnClickOutside))
                .ui(ui, |ui| {
                    let len = species.len();
                    ui.heading(SPECIES).on_hover_text(len.to_string());
                    ui.separator();
                    ScrollArea::vertical()
                        .max_height(ui.spacing().combo_height)
                        .show(ui, |ui| self.list_button_content(ui, &species))
                        .inner
                });
            inner_response.transpose()?;
        }
        Ok(())
    }

    fn list_button_content(&self, ui: &mut Ui, species: &Series) -> PolarsResult<()> {
        Grid::new(ui.next_auto_id())
            .show(ui, |ui| {
                for index in 0..species.len() {
                    ui.label(index.to_string());
                    ui.label(species.struct_()?.field_by_name(LABEL)?.str_value(index)?);
                    NewMeanAndStandardDeviation::new(
                        &species.struct_()?.field_by_name(VALUE)?,
                        index,
                    )
                    .with_standard_deviation(self.state.settings.standard_deviation)
                    .with_sample(true)
                    .show(ui)?;
                    ui.end_row();
                }
                Ok(())
            })
            .inner
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
