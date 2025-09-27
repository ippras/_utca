#[cfg(not(target_arch = "wasm32"))]
pub use self::native::{save, save_data};
#[cfg(target_arch = "wasm32")]
pub use self::web::{save, save_data};

use anyhow::Result;
use metadata::{MetaDataFrame, Metadata};
use polars::prelude::*;
use std::{borrow::BorrowMut, fmt::Debug};
use tracing::instrument;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::fs::File;

    #[instrument(err)]
    pub fn save(
        frame: &mut MetaDataFrame<Metadata, impl BorrowMut<DataFrame> + Debug>,
        name: &str,
    ) -> Result<()> {
        let file = File::create(name)?;
        frame.write_parquet(file)?;
        Ok(())
    }

    #[instrument(err)]
    pub fn save_data(data_frame: &mut DataFrame, name: &str) -> Result<()> {
        let file = File::create(name)?;
        let writer = ParquetWriter::new(file);
        writer.finish(data_frame)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use anyhow::bail;
    use egui_ext::download::{NONE, download};
    use metadata::Metadata;

    #[instrument(err)]
    pub fn save(frame: &mut MetaDataFrame, name: &str) -> Result<()> {
        let mut bytes = Vec::new();
        let mut writer = IpcWriter::new(&mut bytes);
        writer.metadata(frame.meta.clone());
        writer.finish(&mut frame.data)?;
        if let Err(error) = download(&bytes, NONE, name) {
            bail!("save: {error:?}");
        }
        Ok(())
    }

    #[instrument(err)]
    pub fn save_data(data_frame: &mut DataFrame, name: &str) -> Result<()> {
        let mut bytes = Vec::new();
        let mut writer = IpcWriter::new(&mut bytes);
        writer.finish(data_frame)?;
        if let Err(error) = download(&bytes, NONE, name) {
            bail!("save: {error:?}");
        }
        Ok(())
    }
}
