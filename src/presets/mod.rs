use crate::utils::{HashedDataFrame, HashedMetaDataFrame};
use anyhow::Result;
use metadata::polars::MetaDataFrame;
use std::sync::LazyLock;

macro preset($name:literal) {
    LazyLock::new(|| parse(include_bytes!($name)).expect(concat!("preset ", $name)))
}

fn parse(bytes: &[u8]) -> Result<HashedMetaDataFrame> {
    let frame = ron::de::from_bytes::<MetaDataFrame>(bytes)?;
    Ok(MetaDataFrame {
        meta: frame.meta,
        data: HashedDataFrame::new(frame.data).unwrap(),
    })
}

pub(crate) static CHRISTIE: LazyLock<HashedMetaDataFrame> = preset!("Christie.ron");

/// IPPRAS
#[rustfmt::skip]
pub(crate) mod ippras {
    use super::*;

    pub(crate) static ACER_GINNALA_2025_07_08_1: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Ginnala.2025-07-08.0.0.1.utca.ron");
    pub(crate) static ACER_GINNALA_2025_07_08_2: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Ginnala.2025-07-08.0.0.2.utca.ron");
    pub(crate) static ACER_GINNALA_2025_07_08_3: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Ginnala.2025-07-08.0.0.3.utca.ron");
    pub(crate) static ACER_PENSYLVANICUM_2025_07_08_1: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Pensylvanicum.2025-07-08.0.0.1.utca.ron");
    pub(crate) static ACER_PENSYLVANICUM_2025_07_08_2: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Pensylvanicum.2025-07-08.0.0.2.utca.ron");
    pub(crate) static ACER_PENSYLVANICUM_2025_07_08_3: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Pensylvanicum.2025-07-08.0.0.3.utca.ron");
    pub(crate) static ACER_RUBRUM_2025_07_09_1: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Rubrum.2025-07-09.0.0.1.utca.ron");
    pub(crate) static ACER_RUBRUM_2025_07_09_2: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Rubrum.2025-07-09.0.0.2.utca.ron");
    pub(crate) static ACER_RUBRUM_2025_07_09_3: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Rubrum.2025-07-09.0.0.3.utca.ron");
    pub(crate) static ACER_SPICATUM_2025_07_09_1: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Spicatum.2025-07-09.0.0.1.utca.ron");
    pub(crate) static ACER_SPICATUM_2025_07_09_2: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Spicatum.2025-07-09.0.0.2.utca.ron");
    pub(crate) static ACER_SPICATUM_2025_07_09_3: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Spicatum.2025-07-09.0.0.3.utca.ron");
    pub(crate) static ACER_UKURUNDUENSE_2025_07_08_1: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Ukurunduense.2025-07-08.0.0.1.utca.ron");
    pub(crate) static ACER_UKURUNDUENSE_2025_07_08_2: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Ukurunduense.2025-07-08.0.0.2.utca.ron");
    pub(crate) static ACER_UKURUNDUENSE_2025_07_08_3: LazyLock<HashedMetaDataFrame> = preset!("Acer/Acer Ukurunduense.2025-07-08.0.0.3.utca.ron");

    pub(crate) static HELIANTHUS_ANNUUS_2025_10_29_1: LazyLock<HashedMetaDataFrame> = preset!("HelianthusAnnuus/К-2233.25.10.29.0.0.1.utca.ron");
    pub(crate) static HELIANTHUS_ANNUUS_2025_10_29_2: LazyLock<HashedMetaDataFrame> = preset!("HelianthusAnnuus/К-2233.25.10.29.0.0.2.utca.ron");
    pub(crate) static HELIANTHUS_ANNUUS_2025_10_29_3: LazyLock<HashedMetaDataFrame> = preset!("HelianthusAnnuus/К-2233.25.10.29.0.0.3.utca.ron");

    pub(crate) static CEDRUS_2023_05_19: LazyLock<HashedMetaDataFrame> = preset!("Cedrus/Cedrus.2023-05-19.utca.ron");
    pub(crate) static CEDRUS_2023_05_19_1: LazyLock<HashedMetaDataFrame> = preset!("Cedrus/Cedrus.2023-05-19.0.0.1.utca.ron");
    pub(crate) static CEDRUS_2023_05_19_2: LazyLock<HashedMetaDataFrame> = preset!("Cedrus/Cedrus.2023-05-19.0.0.2.utca.ron");

    pub(crate) static C108_2025_04_23_1: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-108(-N).2025-04-23.0.0.1.utca.ron");
    pub(crate) static C108_2025_04_23_2: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-108(-N).2025-04-23.0.0.2.utca.ron");
    pub(crate) static C108_2025_04_23_3: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-108(-N).2025-04-23.0.0.3.utca.ron");
    pub(crate) static C1210_2025_04_23_1: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-1210(-N).2025-04-24.0.0.1.utca.ron");
    pub(crate) static C1210_2025_04_23_2: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-1210(-N).2025-04-24.0.0.2.utca.ron");
    pub(crate) static C1210_2025_04_23_3: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-1210(-N).2025-04-24.0.0.3.utca.ron");
    pub(crate) static C1540_2025_04_24_1: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-1540(-N).2025-04-24.0.0.1.utca.ron");
    pub(crate) static C1540_2025_04_24_2: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-1540(-N).2025-04-24.0.0.2.utca.ron");
    pub(crate) static C1540_2025_04_24_3: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/C-1540(-N).2025-04-24.0.0.3.utca.ron");
    pub(crate) static P519_2025_04_23_1: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/P-519(-N).2025-04-23.0.0.1.utca.ron");
    pub(crate) static P519_2025_04_23_2: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/P-519(-N).2025-04-23.0.0.2.utca.ron");
    pub(crate) static H242_2023_10_24_1: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/H-242(Control).2023-10-24.0.0.1.utca.ron");
    pub(crate) static H242_2023_10_24_2: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/H-242(Control).2023-10-24.0.0.2.utca.ron");
    pub(crate) static H626_2025_04_24: LazyLock<HashedMetaDataFrame> = preset!("Microalgae/H-626(-N).2025-04-24.utca.ron");
}

// [Sidorov2014](https://doi.org/10.1007/s11746-014-2553-8)
#[rustfmt::skip]
pub(crate) mod sidorov2014 {
    use super::*;

    pub(crate) static EUONYMUS_ALATUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Alatus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_BUNGEANUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Bungeanus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_EUROPAEUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Europaeus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_HAMILTONIANUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Hamiltonianus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_LATIFOLIUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Latifolius.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_MACROPTERUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Macropterus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_MAXIMOWICZIANUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Maximowiczianus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_PAUCIFLORUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Pauciflorus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_PHELLOMANUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Phellomanus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_SACHALINENSIS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Sachalinensis.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_SACROSANCTUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Sacrosanctus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_SEMIEXSERTUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Semiexsertus.2014-06-19.utca.ron");
    pub(crate) static EUONYMUS_SIEBOLDIANUS: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2014/Euonymus Sieboldianus.2014-06-19.utca.ron");
}

// [Sidorov2025](https://doi.org/10.3390/plants14040612)
#[rustfmt::skip]
pub(crate) mod sidorov2025 {
    use super::*;

    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_1_1: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.1.1.1.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_1_2: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.1.1.2.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_2_1: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.1.2.1.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_2_2: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.1.2.2.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_3_1: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.1.3.1.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_1_3_2: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.1.3.2.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_2_1_1: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.2.1.1.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_2_1_2: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.2.1.2.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_2_2_1: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.2.2.1.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_2_2_2: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.2.2.2.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_1_1: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.3.1.1.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_1_2: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.3.1.2.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_2_1: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.3.2.1.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_2_2: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.3.2.2.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_3_1: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.3.3.1.utca.ron");
    pub(crate) static LUNARIA_REDIVIVA_2024_01_24_3_3_2: LazyLock<HashedMetaDataFrame> = preset!("Sidorov2025/Lunaria Rediviva.2024-01-24.3.3.2.utca.ron");
}

// Third party

// [Martínez-Force2004](https://doi.org/10.1016/j.ab.2004.07.019)
#[rustfmt::skip]
pub(crate) mod martínez_force2004 {
    use super::*;

    pub(crate) static HAZELNUT: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Martinez-Force2004/Hazelnut.2025-08-19.utca.ron");
    pub(crate) static OLIVE: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Martinez-Force2004/Olive.2025-08-19.utca.ron");
    pub(crate) static RICE: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Martinez-Force2004/Rice.2025-08-19.utca.ron");
    pub(crate) static SOYBEAN: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Martinez-Force2004/Soybean.2025-08-19.utca.ron");
    pub(crate) static SUNFLOWER_CAS3: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Martinez-Force2004/Sunflower CAS-3.2025-08-19.utca.ron");
    pub(crate) static SUNFLOWER_RHA274: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Martinez-Force2004/Sunflower RHA-274.2025-08-19.utca.ron");
    pub(crate) static WALNUT: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Martinez-Force2004/Walnut.2025-08-19.utca.ron");
}

// [Reske1997](https://doi.org/10.1007/s11746-997-0016-1)
#[rustfmt::skip]
pub(crate) mod reske1997 {
    use super::*;

    pub(crate) static SUNFLOWER_SEED_COMMODITY: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Reske1997/Sunﬂower Seed (Commodity).1997-08-01.utca.ron");
    pub(crate) static SUNFLOWER_SEED_HIGH_LINOLEIC: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Reske1997/Sunﬂower Seed (High linoleic).1997-08-01.utca.ron");
    pub(crate) static SUNFLOWER_SEED_HIGH_OLEIC: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Reske1997/Sunﬂower Seed (High oleic).1997-08-01.utca.ron");
    pub(crate) static SUNFLOWER_SEED_HIGH_PALMITIC_HIGH_LINOLEIC: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Reske1997/Sunﬂower Seed (High palmitic, high linoleic).1997-08-01.utca.ron");
    pub(crate) static SUNFLOWER_SEED_HIGH_PALMITIC_HIGH_OLEIC: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Reske1997/Sunﬂower Seed (High palmitic, high oleic).1997-08-01.utca.ron");
    pub(crate) static SUNFLOWER_SEED_HIGH_STEARIC_HIGH_OLEIC: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Reske1997/Sunﬂower Seed (High stearic, high oleic).1997-08-01.utca.ron");

    // pub(crate) static SOYBEAN_SEED_COMMODITY: LazyLock<HashedMetaDataFrame> = preset!("ThirdParty/Reske1997/Soybean Seed Commodity.2025-08-11.utca.ron");
}
