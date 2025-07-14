use crate::{app::panes::composition::settings::Settings, utils::Hashed};
use egui::util::cache::{ComputerMut, FrameCache};
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_ext::prelude::ExprExt as _;
use std::hash::{Hash, Hasher};
use tracing::instrument;

/// Composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        // warn!("index: {:?}", key.settings.index);
        let settings = &key.settings;
        let lazy_frame = match settings.index {
            Some(index) => {
                let frame = &key.frames[index];
                let mut lazy_frame = frame.data.clone().lazy();
                lazy_frame = compute(lazy_frame)?;
                lazy_frame
            }
            None => {
                let compute = |frame: &MetaDataFrame| -> PolarsResult<LazyFrame> {
                    Ok(compute(frame.data.clone().lazy())?.select([
                        as_struct(vec![col("Label"), col("Triacylglycerol")]).hash(),
                        col("Label"),
                        col("Triacylglycerol"),
                        col("Value").alias(frame.meta.format(".").to_string()),
                    ]))
                };
                let mut lazy_frame = compute(&key.frames[0])?;
                for frame in &key.frames[1..] {
                    lazy_frame = lazy_frame.join(
                        compute(frame)?,
                        [col("Hash"), col("Label"), col("Triacylglycerol")],
                        [col("Hash"), col("Label"), col("Triacylglycerol")],
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                    );
                }
                lazy_frame = lazy_frame.drop(by_name(["Hash"], true));
                lazy_frame = lazy_frame.select(mean_and_standard_deviation(key)?);
                lazy_frame
            }
        };
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Composition key
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frames: &'a Hashed<Vec<MetaDataFrame>>,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frames.hash(state);
        self.settings.index.hash(state);
        self.settings.special.hash(state);
    }
}

/// Composition value
type Value = DataFrame;

fn compute(lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    // match &settings.special.method {
    //     Method::Gunstone => gunstone(lazy_frame, settings),
    //     Method::VanderWal => vander_wal(lazy_frame, settings),
    // }
    vander_wal(lazy_frame)
}

fn vander_wal(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame.select([
        col("Label"),
        col("FattyAcid"),
        col("Calculated")
            .struct_()
            .field_by_names(["Diacylglycerol13", "Monoacylglycerol2"]),
    ]);
    // Cartesian product (TAG from FA)
    lazy_frame = cartesian_product(lazy_frame)?;
    Ok(lazy_frame)
}

fn cartesian_product(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
        .clone()
        .select([as_struct(vec![
            col("Label"),
            col("FattyAcid"),
            col("Diacylglycerol13").alias("Value"),
        ])
        .alias("StereospecificNumber1")])
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col("Label"),
                col("FattyAcid"),
                col("Monoacylglycerol2").alias("Value"),
            ])
            .alias("StereospecificNumber2")]),
            None,
        )
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col("Label"),
                col("FattyAcid"),
                col("Diacylglycerol13").alias("Value"),
            ])
            .alias("StereospecificNumber3")]),
            None,
        );
    // Restruct
    let label = |name| col(name).struct_().field_by_name("Label").alias(name);
    let fatty_acid = |name| col(name).struct_().field_by_name("FattyAcid").alias(name);
    let value = |name| col(name).struct_().field_by_name("Value");
    lazy_frame = lazy_frame.select([
        as_struct(vec![
            label("StereospecificNumber1"),
            label("StereospecificNumber2"),
            label("StereospecificNumber3"),
        ])
        .alias("Label"),
        as_struct(vec![
            fatty_acid("StereospecificNumber1"),
            fatty_acid("StereospecificNumber2"),
            fatty_acid("StereospecificNumber3"),
        ])
        .alias("Triacylglycerol"),
        value("StereospecificNumber1")
            * value("StereospecificNumber2")
            * value("StereospecificNumber3"),
    ]);
    Ok(lazy_frame)
}

fn mean_and_standard_deviation(key: Key) -> PolarsResult<[Expr; 3]> {
    let values = || concat_list(vec![all().exclude_cols(["Label", "Triacylglycerol"])]);
    Ok([
        col("Label"),
        col("Triacylglycerol"),
        as_struct(vec![
            values()?.list().mean().alias("Mean"),
            values()?
                .list()
                .std(key.settings.special.ddof)
                .alias("StandardDeviation"),
            values()?.alias("Repetitions"),
        ])
        .alias("Value"),
    ])
}
