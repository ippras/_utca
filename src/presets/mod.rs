pub(crate) use self::r#const::*;

use metadata::MetaDataFrame;
use std::{io::Cursor, sync::LazyLock};

macro_rules! preset {
    ($name:literal) => {
        LazyLock::new(|| {
            let bytes = include_bytes!($name);
            MetaDataFrame::read_parquet(Cursor::new(bytes)).expect(concat!("deserialize ", $name))
        })
    };
}

#[rustfmt::skip]
mod r#const {
    use super::*;

    pub(crate) static ACER_GINNALA_2025_07_08_1: LazyLock<MetaDataFrame> = preset!("Acer/Acer Ginnala.2025-07-08.0.0.1.utca.parquet");
    pub(crate) static ACER_GINNALA_2025_07_08_2: LazyLock<MetaDataFrame> = preset!("Acer/Acer Ginnala.2025-07-08.0.0.2.utca.parquet");
    pub(crate) static ACER_GINNALA_2025_07_08_3: LazyLock<MetaDataFrame> = preset!("Acer/Acer Ginnala.2025-07-08.0.0.3.utca.parquet");
    pub(crate) static ACER_PENSYLVANICUM_2025_07_08_1: LazyLock<MetaDataFrame> = preset!("Acer/Acer Pensylvanicum.2025-07-08.0.0.1.utca.parquet");
    pub(crate) static ACER_PENSYLVANICUM_2025_07_08_2: LazyLock<MetaDataFrame> = preset!("Acer/Acer Pensylvanicum.2025-07-08.0.0.2.utca.parquet");
    pub(crate) static ACER_PENSYLVANICUM_2025_07_08_3: LazyLock<MetaDataFrame> = preset!("Acer/Acer Pensylvanicum.2025-07-08.0.0.3.utca.parquet");
    pub(crate) static ACER_RUBRUM_2025_07_09_1: LazyLock<MetaDataFrame> = preset!("Acer/Acer Rubrum.2025-07-09.0.0.1.utca.parquet");
    pub(crate) static ACER_RUBRUM_2025_07_09_2: LazyLock<MetaDataFrame> = preset!("Acer/Acer Rubrum.2025-07-09.0.0.2.utca.parquet");
    pub(crate) static ACER_RUBRUM_2025_07_09_3: LazyLock<MetaDataFrame> = preset!("Acer/Acer Rubrum.2025-07-09.0.0.3.utca.parquet");
    pub(crate) static ACER_SPICATUM_2025_07_09_1: LazyLock<MetaDataFrame> = preset!("Acer/Acer Spicatum.2025-07-09.0.0.1.utca.parquet");
    pub(crate) static ACER_SPICATUM_2025_07_09_2: LazyLock<MetaDataFrame> = preset!("Acer/Acer Spicatum.2025-07-09.0.0.2.utca.parquet");
    pub(crate) static ACER_SPICATUM_2025_07_09_3: LazyLock<MetaDataFrame> = preset!("Acer/Acer Spicatum.2025-07-09.0.0.3.utca.parquet");
    pub(crate) static ACER_UKURUNDUENSE_2025_07_08_1: LazyLock<MetaDataFrame> = preset!("Acer/Acer Ukurunduense.2025-07-08.0.0.1.utca.parquet");
    pub(crate) static ACER_UKURUNDUENSE_2025_07_08_2: LazyLock<MetaDataFrame> = preset!("Acer/Acer Ukurunduense.2025-07-08.0.0.2.utca.parquet");
    pub(crate) static ACER_UKURUNDUENSE_2025_07_08_3: LazyLock<MetaDataFrame> = preset!("Acer/Acer Ukurunduense.2025-07-08.0.0.3.utca.parquet");

    pub(crate) static CEDRUS_2023_05_19: LazyLock<MetaDataFrame> = preset!("Cedrus/Cedrus.2023-05-19.utca.parquet");

    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_1: LazyLock<MetaDataFrame> = preset!("Lunaria/Lunaria Rediviva.2024-01-24.0.1.1.utca.parquet");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_2: LazyLock<MetaDataFrame> = preset!("Lunaria/Lunaria Rediviva.2024-01-24.0.1.2.utca.parquet");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_3: LazyLock<MetaDataFrame> = preset!("Lunaria/Lunaria Rediviva.2024-01-24.0.1.3.utca.parquet");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_2_1: LazyLock<MetaDataFrame> = preset!("Lunaria/Lunaria Rediviva.2024-01-24.0.2.1.utca.parquet");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_2_2: LazyLock<MetaDataFrame> = preset!("Lunaria/Lunaria Rediviva.2024-01-24.0.2.2.utca.parquet");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_1: LazyLock<MetaDataFrame> = preset!("Lunaria/Lunaria Rediviva.2024-01-24.0.3.1.utca.parquet");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_2: LazyLock<MetaDataFrame> = preset!("Lunaria/Lunaria Rediviva.2024-01-24.0.3.2.utca.parquet");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_3: LazyLock<MetaDataFrame> = preset!("Lunaria/Lunaria Rediviva.2024-01-24.0.3.3.utca.parquet");

    pub(crate) static C108_2025_04_23_1: LazyLock<MetaDataFrame> = preset!("Microalgae/C108-N.2025-04-23.0.0.1.utca.parquet");
    pub(crate) static C108_2025_04_23_2: LazyLock<MetaDataFrame> = preset!("Microalgae/C108-N.2025-04-23.0.0.2.utca.parquet");
    pub(crate) static C108_2025_04_23_3: LazyLock<MetaDataFrame> = preset!("Microalgae/C108-N.2025-04-23.0.0.3.utca.parquet");
    pub(crate) static C519_2025_04_23_1: LazyLock<MetaDataFrame> = preset!("Microalgae/519-N.2025-04-23.0.0.1.utca.parquet");
    pub(crate) static C519_2025_04_23_2: LazyLock<MetaDataFrame> = preset!("Microalgae/519-N.2025-04-23.0.0.2.utca.parquet");
    pub(crate) static C1210_2025_04_23_1: LazyLock<MetaDataFrame> = preset!("Microalgae/C1210-N.2025-04-24.0.0.1.utca.parquet");
    pub(crate) static C1210_2025_04_23_2: LazyLock<MetaDataFrame> = preset!("Microalgae/C1210-N.2025-04-24.0.0.2.utca.parquet");
    pub(crate) static C1210_2025_04_23_3: LazyLock<MetaDataFrame> = preset!("Microalgae/C1210-N.2025-04-24.0.0.3.utca.parquet");
    pub(crate) static H626_2025_04_24: LazyLock<MetaDataFrame> = preset!("Microalgae/H626-N.2025-04-24.utca.parquet");
    pub(crate) static LOBOSPHERA_2025_04_24_1: LazyLock<MetaDataFrame> = preset!("Microalgae/Lobosphera-N.2025-04-24.0.0.1.utca.parquet");
    pub(crate) static LOBOSPHERA_2025_04_24_2: LazyLock<MetaDataFrame> = preset!("Microalgae/Lobosphera-N.2025-04-24.0.0.2.utca.parquet");
    pub(crate) static LOBOSPHERA_2025_04_24_3: LazyLock<MetaDataFrame> = preset!("Microalgae/Lobosphera-N.2025-04-24.0.0.3.utca.parquet");
}
