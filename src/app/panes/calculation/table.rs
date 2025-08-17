use super::{
    ID_SOURCE, State,
    settings::{From, Settings},
};
use crate::app::{panes::MARGIN, widgets::FloatWidget};
use egui::{Frame, Id, Margin, Response, TextStyle, TextWrapMode, Ui};
use egui_l20n::UiExt as _;
use egui_phosphor::regular::HASH;
use egui_table::{CellInfo, Column, HeaderCellInfo, HeaderRow, Table, TableDelegate, TableState};
use lipid::prelude::*;
use polars::prelude::*;
use std::ops::Range;
use tracing::instrument;

const ID: Range<usize> = 0..3;
const EXPERIMENTAL: Range<usize> = ID.end..ID.end + 3;
const THEORETICAL: Range<usize> = EXPERIMENTAL.end..EXPERIMENTAL.end + 5;
const FACTORS: Range<usize> = THEORETICAL.end..THEORETICAL.end + 2;
const LEN: usize = FACTORS.end;

const TOP: &[Range<usize>] = &[ID, EXPERIMENTAL, THEORETICAL, FACTORS];
const MIDDLE: &[Range<usize>] = &[
    id::INDEX,
    id::LABEL,
    id::FA,
    experimental::TAG,
    experimental::DAG1223,
    experimental::MAG2,
    theoretical::TAG,
    theoretical::DAG1223,
    theoretical::MAG2,
    theoretical::DAG13,
    factors::EF,
    factors::SF,
];

/// Calculation table
pub(crate) struct TableView<'a> {
    data_frame: &'a DataFrame,
    settings: &'a Settings,
    state: &'a mut State,
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
    pub(crate) fn show(&mut self, ui: &mut Ui) {
        let id_salt = Id::new(ID_SOURCE).with("Table");
        if self.state.reset_table_state {
            let id = TableState::id(ui, Id::new(id_salt));
            TableState::reset(ui.ctx(), id);
            self.state.reset_table_state = false;
        }
        let height = ui.text_style_height(&TextStyle::Heading) + 2.0 * MARGIN.y;
        let num_rows = self.data_frame.height() as u64 + 1;
        let num_columns = LEN;
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
                    groups: TOP.to_vec(),
                },
                HeaderRow {
                    height,
                    groups: MIDDLE.to_vec(),
                },
                HeaderRow::new(height),
            ])
            .show(ui, self);
    }

    fn header_cell_content_ui(&mut self, ui: &mut Ui, row: usize, column: Range<usize>) {
        if self.settings.truncate_headers {
            ui.style_mut().wrap_mode = Some(TextWrapMode::Truncate);
        }
        match (row, column) {
            // Top
            (0, ID) => {
                ui.heading(ui.localize("Identifier.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("Identifier"));
                    });
            }
            (0, EXPERIMENTAL) => {
                ui.heading(ui.localize("Experimental"));
            }
            (0, THEORETICAL) => {
                ui.heading(ui.localize("Theoretical"));
            }
            (0, FACTORS) if self.settings.factors => {
                ui.heading(ui.localize("Factors"));
            }
            // Middle
            (1, id::INDEX) => {
                ui.heading(HASH).on_hover_ui(|ui| {
                    ui.label(ui.localize("Index"));
                });
            }
            (1, id::LABEL) => {
                ui.heading(ui.localize("Label"));
            }
            (1, id::FA) => {
                ui.heading(ui.localize("FattyAcid.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("FattyAcid"));
                    });
            }
            (1, experimental::TAG) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=123"));
                    });
            }
            (1, experimental::DAG1223) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=1223"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber.abbreviation?number=1223"));
                    });
            }
            (1, experimental::MAG2) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=2"));
                    });
            }
            (1, theoretical::TAG) if self.settings.theoretical => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=123"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=2"));
                    });
            }
            (1, theoretical::DAG1223) if self.settings.theoretical => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=1223"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=1223"));
                    });
            }
            (1, theoretical::MAG2) if self.settings.theoretical => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=2"));
                    });
            }
            (1, theoretical::DAG13) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=13"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=13"));
                    });
            }
            (1, factors::EF) if self.settings.factors => {
                ui.heading(ui.localize("EnrichmentFactor.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("EnrichmentFactor"));
                    });
            }
            (1, factors::SF) if self.settings.factors => {
                ui.heading(ui.localize("SelectivityFactor.abbreviation"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("SelectivityFactor"));
                    });
            }
            // Bottom
            (2, theoretical::dag13::DAG1223) => {
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=1223"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=1223"));
                    });
            }
            (2, theoretical::dag13::MAG2) => {
                // "Calculated from sn-2 {}",
                ui.heading(ui.localize("StereospecificNumber.abbreviation?number=2"))
                    .on_hover_ui(|ui| {
                        ui.label(ui.localize("StereospecificNumber?number=2"));
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
            (row, id::INDEX) => {
                ui.label(row.to_string());
            }
            (row, id::LABEL) => {
                let labels = self.data_frame["Label"].str()?;
                let label = labels.get(row).unwrap();
                ui.label(label);
            }
            (row, id::FA) => {
                if let Some(fatty_acid) = self.data_frame.try_fatty_acid()?.delta()?.get(row) {
                    ui.label(fatty_acid);
                }
                // FattyAcidWidget::new(fatty_acid).hover(true).show(ui);
            }
            (row, experimental::TAG) => {
                self.value(
                    ui,
                    self.data_frame["Experimental"]
                        .struct_()?
                        .field_by_name("Triacylglycerol")?,
                    Some(row),
                    self.settings.percent,
                    false,
                )?;
            }
            (row, experimental::DAG1223) => {
                self.value(
                    ui,
                    self.data_frame["Experimental"]
                        .struct_()?
                        .field_by_name("Diacylglycerol1223")?,
                    Some(row),
                    self.settings.percent,
                    self.settings.from != From::Dag1223,
                )?;
            }
            (row, experimental::MAG2) => {
                self.value(
                    ui,
                    self.data_frame["Experimental"]
                        .struct_()?
                        .field_by_name("Monoacylglycerol2")?,
                    Some(row),
                    self.settings.percent,
                    self.settings.from != From::Mag2,
                )?;
            }
            (row, theoretical::TAG) if self.settings.theoretical => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Triacylglycerol")?,
                    Some(row),
                    self.settings.percent,
                    true,
                )?;
            }
            (row, theoretical::DAG1223) if self.settings.theoretical => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Diacylglycerol1223")?,
                    Some(row),
                    self.settings.percent,
                    true,
                )?;
            }
            (row, theoretical::MAG2) if self.settings.theoretical => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Monoacylglycerol2")?,
                    Some(row),
                    self.settings.percent,
                    true,
                )?;
            }
            (row, theoretical::dag13::DAG1223) => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Diacylglycerol13")?
                        .struct_()?
                        .field_by_name("Diacylglycerol1223")?,
                    Some(row),
                    self.settings.percent,
                    self.settings.from != From::Dag1223,
                )?;
            }
            (row, theoretical::dag13::MAG2) => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Diacylglycerol13")?
                        .struct_()?
                        .field_by_name("Monoacylglycerol2")?,
                    Some(row),
                    self.settings.percent,
                    self.settings.from != From::Mag2,
                )?;
            }
            (row, factors::ef::MAG2) if self.settings.factors => {
                self.value(
                    ui,
                    self.data_frame["Factors"]
                        .struct_()?
                        .field_by_name("Enrichment")?,
                    Some(row),
                    false,
                    false,
                )?;
            }
            (row, factors::sf::MAG2) if self.settings.factors => {
                self.value(
                    ui,
                    self.data_frame["Factors"]
                        .struct_()?
                        .field_by_name("Selectivity")?,
                    Some(row),
                    false,
                    false,
                )?;
            }
            _ => {}
        }
        Ok(())
    }

    fn footer_cell_content_ui(&mut self, ui: &mut Ui, column: Range<usize>) -> PolarsResult<()> {
        match column {
            experimental::TAG => {
                self.value(
                    ui,
                    self.data_frame["Experimental"]
                        .struct_()?
                        .field_by_name("Triacylglycerol")?,
                    None,
                    self.settings.percent,
                    false,
                )?
                .on_hover_text("∑ TAG");
            }
            experimental::DAG1223 => {
                self.value(
                    ui,
                    self.data_frame["Experimental"]
                        .struct_()?
                        .field_by_name("Diacylglycerol1223")?,
                    None,
                    self.settings.percent,
                    self.settings.from != From::Dag1223,
                )?
                .on_hover_text("∑ DAG1223");
            }
            experimental::MAG2 => {
                self.value(
                    ui,
                    self.data_frame["Experimental"]
                        .struct_()?
                        .field_by_name("Monoacylglycerol2")?,
                    None,
                    self.settings.percent,
                    self.settings.from != From::Mag2,
                )?
                .on_hover_text("∑ MAG2");
            }
            theoretical::TAG if self.settings.theoretical => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Triacylglycerol")?,
                    None,
                    self.settings.percent,
                    true,
                )?
                .on_hover_text("∑ TAG");
            }
            theoretical::DAG1223 if self.settings.theoretical => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Diacylglycerol1223")?,
                    None,
                    self.settings.percent,
                    true,
                )?
                .on_hover_text("∑ DAG1223");
            }
            theoretical::MAG2 if self.settings.theoretical => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Monoacylglycerol2")?,
                    None,
                    self.settings.percent,
                    true,
                )?
                .on_hover_text("∑ MAG2");
            }
            theoretical::dag13::DAG1223 => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Diacylglycerol13")?
                        .struct_()?
                        .field_by_name("Diacylglycerol1223")?,
                    None,
                    self.settings.percent,
                    self.settings.from != From::Dag1223,
                )?
                .on_hover_text("∑ DAG13");
            }
            theoretical::dag13::MAG2 => {
                self.value(
                    ui,
                    self.data_frame["Theoretical"]
                        .struct_()?
                        .field_by_name("Diacylglycerol13")?
                        .struct_()?
                        .field_by_name("Monoacylglycerol2")?,
                    None,
                    self.settings.percent,
                    self.settings.from != From::Mag2,
                )?
                .on_hover_text("∑ DAG13");
            }
            _ => {}
        }
        Ok(())
    }

    fn value(
        &self,
        ui: &mut Ui,
        series: Series,
        row: Option<usize>,
        percent: bool,
        disable: bool,
    ) -> PolarsResult<Response> {
        Ok(if let Some(r#struct) = series.try_struct() {
            let mean = if let Some(row) = row {
                r#struct.field_by_name("Mean")?.f64()?.get(row)
            } else {
                r#struct.field_by_name("Mean")?.f64()?.sum()
            };
            let response = FloatWidget::new(mean)
                .percent(percent)
                .precision(Some(self.settings.precision))
                .disable(disable)
                .hover(true)
                .show(ui)
                .response;
            if let Some(row) = row {
                response.on_hover_text(r#struct.field_by_name("Values")?.str_value(row)?);
            }
            ui.label("±");
            let standard_deviation = if let Some(row) = row {
                r#struct.field_by_name("StandardDeviation")?.f64()?.get(row)
            } else {
                r#struct.field_by_name("StandardDeviation")?.f64()?.sum()
            };
            FloatWidget::new(standard_deviation)
                .percent(percent)
                .precision(Some(self.settings.precision))
                .disable(disable)
                .hover(true)
                .show(ui)
                .response
        } else {
            let values = series.f64()?;
            let value = if let Some(row) = row {
                values.get(row)
            } else {
                values.sum()
            };
            FloatWidget::new(value)
                .percent(percent)
                .precision(Some(self.settings.precision))
                .disable(disable)
                .hover(true)
                .show(ui)
                .response
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
                let _ = self.cell_content_ui(ui, cell.row_nr as _, cell.col_nr..cell.col_nr + 1);
            });
    }
}

mod id {
    use super::*;

    pub(super) const INDEX: Range<usize> = ID.start..ID.start + 1;
    pub(super) const LABEL: Range<usize> = INDEX.end..INDEX.end + 1;
    pub(super) const FA: Range<usize> = LABEL.end..LABEL.end + 1;
}

mod experimental {
    use super::*;

    pub(super) const TAG: Range<usize> = EXPERIMENTAL.start..EXPERIMENTAL.start + 1;
    pub(super) const DAG1223: Range<usize> = TAG.end..TAG.end + 1;
    pub(super) const MAG2: Range<usize> = DAG1223.end..DAG1223.end + 1;
}

mod theoretical {
    use super::*;

    pub(super) const TAG: Range<usize> = THEORETICAL.start..THEORETICAL.start + 1;
    pub(super) const DAG1223: Range<usize> = TAG.end..TAG.end + 1;
    pub(super) const MAG2: Range<usize> = DAG1223.end..DAG1223.end + 1;
    pub(super) const DAG13: Range<usize> = MAG2.end..MAG2.end + 2;

    pub(super) mod dag13 {
        use super::*;

        pub(in super::super) const DAG1223: Range<usize> = DAG13.start..DAG13.start + 1;
        pub(in super::super) const MAG2: Range<usize> = DAG1223.end..DAG1223.end + 1;
    }
}

mod factors {
    use super::*;

    pub(super) const EF: Range<usize> = FACTORS.start..FACTORS.start + 1;
    pub(super) const SF: Range<usize> = EF.end..EF.end + 1;

    pub(super) mod ef {
        use super::*;

        pub(in super::super) const MAG2: Range<usize> = EF.start..EF.start + 1;
    }

    pub(super) mod sf {
        use super::*;

        pub(in super::super) const MAG2: Range<usize> = SF.start..SF.start + 1;
    }
}
