pub use self::{
    hash::{HashedDataFrame, HashedMetaDataFrame, hash_data_frame, hash_expr},
    polars::SchemaExt,
    spawn::spawn,
    trie::unique_prefixes,
};

pub mod egui;
pub mod metadata;
pub mod polars;
pub mod ui;

mod hash;
mod spawn;
mod trie;
