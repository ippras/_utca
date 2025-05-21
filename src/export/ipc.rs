use anyhow::Result;
use metadata::MetaDataFrame;
use tracing::instrument;

#[cfg(not(target_arch = "wasm32"))]
pub use self::native::{save, save_data};
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use polars::prelude::*;
    use std::fs::File;

    #[instrument(err)]
    pub fn save(frame: &mut MetaDataFrame, name: &str) -> Result<()> {
        let file = File::create(name)?;
        MetaDataFrame::new(frame.meta.clone(), &mut frame.data).write(file)?;
        Ok(())
    }

    #[instrument(err)]
    pub fn save_data(data_frame: &mut DataFrame, name: &str) -> Result<()> {
        let file = File::create(name)?;
        let mut writer = IpcWriter::new(file);
        // writer.metadata(self.meta);
        writer.finish(data_frame)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use anyhow::bail;
    use egui_ext::download::{NONE, download};

    #[instrument(err)]
    pub fn save(frame: &mut MetaDataFrame, name: &str) -> Result<()> {
        let mut bytes = Vec::new();
        MetaDataFrame::new(frame.meta.clone(), &mut frame.data).write(&mut bytes)?;
        if let Err(error) = download(&bytes, NONE, name) {
            bail!("save: {error:?}");
        }
        Ok(())
    }
}
