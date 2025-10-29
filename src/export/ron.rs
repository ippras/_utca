#[cfg(not(target_arch = "wasm32"))]
pub use self::native::save;
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

use anyhow::Result;
use metadata::polars::MetaDataFrame;
use ron::{
    extensions::Extensions,
    ser::{PrettyConfig, to_string_pretty},
};
use serde::Serialize;
use std::sync::LazyLock;
use tracing::instrument;

const CONFIG: LazyLock<PrettyConfig> =
    LazyLock::new(|| PrettyConfig::new().extensions(Extensions::UNWRAP_NEWTYPES));

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use super::*;
    use std::{fs::File, io::Write};

    #[instrument(skip(frame), err)]
    pub fn save(frame: &MetaDataFrame<impl Serialize, impl Serialize>, name: &str) -> Result<()> {
        let mut file = File::create(name)?;
        let serialized = to_string_pretty(&frame, CONFIG.clone())?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use anyhow::bail;
    use egui_ext::download::{NONE, download};

    #[instrument(skip(frame), err)]
    pub fn save(frame: &MetaDataFrame<impl Serialize, impl Serialize>, name: &str) -> Result<()> {
        let serialized = to_string_pretty(&frame, CONFIG.clone())?;
        if let Err(error) = download(serialized.as_bytes(), NONE, name) {
            bail!("save: {error:?}");
        }
        Ok(())
    }
}
