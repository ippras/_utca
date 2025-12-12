// let window = window().expect("Window not found");
// let document = window.document().expect("Document not found");
// let body = document.body().expect("Document not found");
use thiserror::Error;

/// Result
pub type Result<T, E = Error> = core::result::Result<T, E>;

/// Error
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum Error {
    #[error("window not found")]
    WindowNotFound,
    #[error("document not found")]
    DocumentNotFound,
    #[error("body not found")]
    BodyNotFound,
    // #[error(transparent)]
    // Molecule(#[from] molecule::Error),
    // #[error("tag not found")]
    // TagNotFound,
    // #[error("taxonomy not found")]
    // TaxonomyNotFound,
    // #[error(transparent)]
    // Toml(#[from] toml_edit::TomlError),
}
