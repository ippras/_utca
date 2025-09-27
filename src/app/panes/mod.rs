use crate::utils::HashedMetaDataFrame;
use egui::{Response, Ui, Vec2, vec2};
use serde::{Deserialize, Serialize};

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

    pub(crate) fn calculation(frames: Vec<HashedMetaDataFrame>, index: usize) -> Self {
        Self::Calculation(calculation::Pane::new(frames, index))
    }

    pub(crate) fn composition(frames: Vec<HashedMetaDataFrame>, index: Option<usize>) -> Self {
        Self::Composition(composition::Pane::new(frames, index))
    }

    pub(crate) const fn kind(&self) -> Kind {
        match self {
            Self::Configuration(_) => Kind::Configuration,
            Self::Calculation(_) => Kind::Calculation,
            Self::Composition(_) => Kind::Composition,
        }
    }

    pub(crate) fn title(&self) -> String {
        match self {
            Self::Configuration(pane) => pane.title(),
            Self::Calculation(pane) => pane.title(),
            Self::Composition(pane) => pane.title(),
        }
    }

    fn header(&mut self, ui: &mut Ui) -> Response {
        match self {
            Self::Configuration(pane) => pane.header(ui),
            Self::Calculation(pane) => pane.header(ui),
            Self::Composition(pane) => pane.header(ui),
        }
    }

    fn body(&mut self, ui: &mut Ui) {
        match self {
            Self::Configuration(pane) => pane.body(ui),
            Self::Calculation(pane) => pane.body(ui),
            Self::Composition(pane) => pane.body(ui),
        }
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

/// Pane delegate
pub(crate) trait PaneDelegate {
    fn header(&mut self, ui: &mut Ui) -> Response;

    fn body(&mut self, ui: &mut Ui);
}

/// Central pane kind
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) enum Kind {
    Configuration,
    Calculation,
    Composition,
}

pub(crate) mod behavior;
pub(crate) mod calculation;
pub(crate) mod composition;
pub(crate) mod configuration;
