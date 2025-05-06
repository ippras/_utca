use anyhow::Result;
use metadata::MetaDataFrame;
use std::fs::File;

#[cfg(not(target_arch = "wasm32"))]
pub fn save(frame: &mut MetaDataFrame, name: &str) -> Result<()> {
    let file = File::create(name)?;
    MetaDataFrame::new(frame.meta.clone(), &mut frame.data).write(file)?;
    Ok(())
}

#[cfg(target_arch = "wasm32")]
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
