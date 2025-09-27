use metadata::MetaDataFrame;
use std::{io::Cursor, sync::LazyLock};

pub(crate) static CHRISTIE: LazyLock<MetaDataFrame> = LazyLock::new(|| {
    let bytes = include_bytes!("Christie.ipc");
    MetaDataFrame::read_parquet(Cursor::new(bytes)).expect("read metadata Christie.ipc")
});
