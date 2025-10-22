#[cfg(not(target_arch = "wasm32"))]
pub use self::native::save;
#[cfg(target_arch = "wasm32")]
pub use self::web::save;

use anyhow::Result;
use metadata::{MetaDataFrame, Metadata};
use ron::ser::{PrettyConfig, to_string_pretty};
use tracing::instrument;

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use ron::extensions::Extensions;

    use super::*;
    use std::{fs::File, io::Write};

    #[instrument(err)]
    pub fn save(frame: &MetaDataFrame, name: &str) -> Result<()> {
        let mut file = File::create(name)?;
        let serialized = to_string_pretty(
            &frame,
            PrettyConfig::default().extensions(Extensions::UNWRAP_NEWTYPES),
        )?;
        file.write_all(serialized.as_bytes())?;
        Ok(())
    }
}

#[cfg(target_arch = "wasm32")]
mod web {
    use super::*;
    use anyhow::bail;
    use egui_ext::download::{NONE, download};

    #[instrument(err)]
    pub fn save(frame: &MetaDataFrame, name: &str) -> Result<()> {
        let serialized = to_string_pretty(&frame, PrettyConfig::default())?;
        if let Err(error) = download(serialized.as_bytes(), NONE, name) {
            bail!("save: {error:?}");
        }
        Ok(())
    }
}
