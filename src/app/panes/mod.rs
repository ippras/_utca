use crate::utils::HashedMetaDataFrame;
use egui::{Ui, Vec2, WidgetText, vec2};
use egui_tiles::{TileId, UiResponse};
use serde::{Deserialize, Serialize};
use std::fmt::Display;

const MARGIN: Vec2 = vec2(4.0, 2.0);

/// Central pane
#[derive(Deserialize, Serialize)]
pub(crate) enum Pane {
    Configuration(configuration::Pane),
    Calculation(calculation::Pane),
    Composition(composition::Pane),
}

impl Pane {
    pub(crate) fn configuration(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self::Configuration(configuration::Pane::new(frames))
    }

    pub(crate) fn calculation(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self::Calculation(calculation::Pane::new(frames))
    }

    pub(crate) fn composition(frames: Vec<HashedMetaDataFrame>) -> Self {
        Self::Composition(composition::Pane::new(frames))
    }

    pub(crate) const fn kind(&self) -> Kind {
        match self {
            Self::Configuration(_) => Kind::Configuration,
            Self::Calculation(_) => Kind::Calculation,
            Self::Composition(_) => Kind::Composition,
        }
    }

    pub(crate) fn title(&self) -> impl Display {
        // match self {
        //     Self::Configuration(pane) => pane.title(),
        //     Self::Calculation(pane) => pane.title(),
        //     Self::Composition(pane) => pane.title(),
        // }
        "pane.title()"
    }
}

impl From<&Pane> for Kind {
    fn from(value: &Pane) -> Self {
        value.kind()
    }
}

impl PartialEq for Pane {
    fn eq(&self, other: &Self) -> bool {
        self.kind() == other.kind()
    }
}

/// Behavior
#[derive(Debug)]
pub(crate) struct Behavior {
    pub(crate) close: Option<TileId>,
}

impl egui_tiles::Behavior<Pane> for Behavior {
    fn tab_title_for_pane(&mut self, pane: &Pane) -> WidgetText {
        pane.title().to_string().into()
    }

    fn pane_ui(&mut self, ui: &mut Ui, tile_id: TileId, pane: &mut Pane) -> UiResponse {
        match pane {
            Pane::Configuration(pane) => pane.ui(ui, self, tile_id),
            Pane::Calculation(pane) => pane.ui(ui, self, tile_id),
            Pane::Composition(pane) => pane.ui(ui, self, tile_id),
        }
    }
}

/// Central pane kind
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Configuration,
    Calculation,
    Composition,
}

pub(crate) mod calculation;
pub(crate) mod composition;
pub(crate) mod configuration;
