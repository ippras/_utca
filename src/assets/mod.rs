use crate::utils::{HashedDataFrame, HashedMetaDataFrame};
use anyhow::Result;
use metadata::polars::MetaDataFrame;
use std::sync::LazyLock;

macro ron($name:literal) {
    LazyLock::new(|| parse(include_bytes!($name)).expect(concat!("preset ", $name)))
}

fn parse(bytes: &[u8]) -> Result<HashedMetaDataFrame> {
    let frame = ron::de::from_bytes::<MetaDataFrame>(bytes)?;
    Ok(MetaDataFrame {
        meta: frame.meta,
        data: HashedDataFrame::new(frame.data).unwrap(),
    })
}

pub(crate) static CHRISTIE: LazyLock<HashedMetaDataFrame> = ron!("Christie.ron");
