use metadata::MetaDataFrame;
use std::{io::Cursor, sync::LazyLock};

macro_rules! preset {
    ($name:literal) => {
        LazyLock::new(|| {
            let bytes = include_bytes!($name);
            MetaDataFrame::read(Cursor::new(bytes)).expect(concat!("deserialize ", $name))
        })
    };
}

pub(crate) static C519_2025_04_23_1: LazyLock<MetaDataFrame> =
    preset!("C519/519-N.2025-04-23.0.0.1.utca.ipc");
pub(crate) static C519_2025_04_23_2: LazyLock<MetaDataFrame> =
    preset!("C519/519-N.2025-04-23.0.0.2.utca.ipc");

pub(crate) static C108_2025_04_23_1: LazyLock<MetaDataFrame> =
    preset!("C108/C108-N.2025-04-23.0.0.1.utca.ipc");
pub(crate) static C108_2025_04_23_2: LazyLock<MetaDataFrame> =
    preset!("C108/C108-N.2025-04-23.0.0.2.utca.ipc");
pub(crate) static C108_2025_04_23_3: LazyLock<MetaDataFrame> =
    preset!("C108/C108-N.2025-04-23.0.0.3.utca.ipc");

pub(crate) static C1210_2025_04_23_1: LazyLock<MetaDataFrame> =
    preset!("C1210/C1210-N.2025-04-24.0.0.1.utca.ipc");
pub(crate) static C1210_2025_04_23_2: LazyLock<MetaDataFrame> =
    preset!("C1210/C1210-N.2025-04-24.0.0.2.utca.ipc");
pub(crate) static C1210_2025_04_23_3: LazyLock<MetaDataFrame> =
    preset!("C1210/C1210-N.2025-04-24.0.0.3.utca.ipc");

pub(crate) static H626_2025_04_24: LazyLock<MetaDataFrame> =
    preset!("H626/H626-N.2025-04-24.utca.ipc");

pub(crate) static LOBOSPHERA_2025_04_24_1: LazyLock<MetaDataFrame> =
    preset!("Lobosphera/Lobosphera-N.2025-04-24.0.0.1.utca.ipc");
pub(crate) static LOBOSPHERA_2025_04_24_2: LazyLock<MetaDataFrame> =
    preset!("Lobosphera/Lobosphera-N.2025-04-24.0.0.2.utca.ipc");
pub(crate) static LOBOSPHERA_2025_04_24_3: LazyLock<MetaDataFrame> =
    preset!("Lobosphera/Lobosphera-N.2025-04-24.0.0.3.utca.ipc");
