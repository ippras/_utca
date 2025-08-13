use crate::text::Text;
use constcat::concat;
use egui::Vec2b;
use egui_phosphor::regular::{CHART_BAR, TABLE};
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize)]
pub(crate) struct State {
    pub(crate) open_settings_window: bool,
    pub(crate) open_indices_window: bool,
    pub(crate) view: View,
    // Table
    pub(crate) reset_table_state: bool,
    // Plot
    pub(crate) allow_drag: Vec2b,
    pub(crate) allow_scroll: Vec2b,
    pub(crate) show_legend: bool,
}

impl State {
    pub(crate) fn new() -> Self {
        Self {
            open_settings_window: false,
            open_indices_window: false,
            view: View::Table,

            reset_table_state: false,

            allow_drag: Vec2b { x: false, y: false },
            allow_scroll: Vec2b { x: false, y: false },
            show_legend: true,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Serialize)]
pub(crate) enum View {
    Plot,
    #[default]
    Table,
}

impl View {
    pub(crate) const fn icon(&self) -> &'static str {
        match self {
            Self::Plot => CHART_BAR,
            Self::Table => TABLE,
        }
    }

    pub(crate) const fn title(&self) -> &'static str {
        match self {
            Self::Plot => "Plot",
            Self::Table => "Table",
        }
    }
}

impl Text for View {
    fn text(&self) -> &'static str {
        match self {
            Self::Plot => concat!(CHART_BAR, " Plot"),
            Self::Table => concat!(TABLE, " Table"),
        }
    }

    fn hover_text(&self) -> &'static str {
        self.title()
    }
}
