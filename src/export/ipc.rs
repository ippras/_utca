#[cfg(not(target_arch = "wasm32"))]
pub use self::native::save;
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use anyhow::Result;
    use metadata::MetaDataFrame;
    use std::fs::File;
    use tracing::instrument;

    #[instrument(err)]
    pub fn save(frame: &mut MetaDataFrame, name: &str) -> Result<()> {
        let file = File::create(name)?;
        MetaDataFrame::new(frame.meta.clone(), &mut frame.data).write(file)?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use anyhow::{Result, bail};
    use egui_ext::download::{XLSX, download};
    use metadata::MetaDataFrame;
    use tracing::instrument;

    #[instrument(err)]
    pub fn save(frame: &mut MetaDataFrame, name: &str) -> Result<()> {
        use anyhow::bail;
        use egui_ext::download::{NONE, download};

        let mut bytes = Vec::new();
        MetaDataFrame::new(frame.meta.clone(), &mut frame.data).write(&mut bytes)?;
        if let Err(error) = download(&bytes, NONE, name) {
            bail!("save: {error:?}");
        }
        Ok(())
    }
}
