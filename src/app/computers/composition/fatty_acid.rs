use crate::app::panes::composition::settings::Settings;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_ext::ExprExt as _;
use std::hash::{Hash, Hasher};

/// Fatty acids computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Fatty acids computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let settings = key.settings.clone();
        let select = |data_frame: &DataFrame| {
            data_frame
                .clone()
                .lazy()
                .select([col("FattyAcid"), col("Label").alias("Species")])
        };
        let mut lazy_frame = match settings.index {
            Some(index) => select(&key.frames[index].data),
            None => {
                let hash = |data_frame: &DataFrame| {
                    select(data_frame)
                        .with_column(as_struct(vec![col("FattyAcid"), col("Species")]).hash())
                };
                let mut lazy_frame = hash(&key.frames[0].data);
                for frame in &key.frames[1..] {
                    lazy_frame = lazy_frame.join(
                        hash(&frame.data),
                        [col("Hash"), col("FattyAcid"), col("Species")],
                        [col("Hash"), col("FattyAcid"), col("Species")],
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                    );
                }
                lazy_frame = lazy_frame.drop([col("Hash")]);
                lazy_frame
            }
        };
        lazy_frame = lazy_frame.with_columns([
            col("FattyAcid").fa().ecn().alias("EquivalentCarbonNumber"),
            col("FattyAcid").fa().mass(None).alias("Mass"),
            col("FattyAcid").fa().is_saturated().alias("Type"),
            col("FattyAcid")
                .fa()
                .unsaturated()
                .sum()
                .alias("Unsaturation"),
        ]);
        // lazy_frame = lazy_frame.sort(["Species", "FattyAcid"], Default::default());
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Fatty acids key
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frames: &'a [MetaDataFrame],
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frames.hash(state);
        self.settings.index.hash(state);
    }
}

/// Fatty acids value
type Value = DataFrame;
