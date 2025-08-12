use super::cartesian_product;
use crate::utils::Hashed;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
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
    pub(super) fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let lazy_frame = match key.index {
            Some(index) => {
                let frame = &key.frames[index];
                let mut lazy_frame = frame.data.clone().lazy();
                lazy_frame = compute(lazy_frame)?;
                lazy_frame
            }
            None => {
                let compute = |frame: &MetaDataFrame| -> PolarsResult<LazyFrame> {
                    Ok(compute(frame.data.clone().lazy())?.select([
                        as_struct(vec![col(LABEL), col(TRIACYLGLYCEROL)]).hash(),
                        col(LABEL),
                        col(TRIACYLGLYCEROL),
                        col("Value").alias(frame.meta.format(".").to_string()),
                    ]))
                };
                let mut lazy_frame = compute(&key.frames[0])?;
                for frame in &key.frames[1..] {
                    lazy_frame = lazy_frame.join(
                        compute(frame)?,
                        [col("Hash"), col(LABEL), col(TRIACYLGLYCEROL)],
                        [col("Hash"), col(LABEL), col(TRIACYLGLYCEROL)],
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                    );
                }
                lazy_frame = lazy_frame.drop(by_name(["Hash"], true));
                lazy_frame = lazy_frame.select(mean_and_standard_deviation(key.ddof)?);
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
    pub(crate) index: Option<usize>,
    pub(crate) ddof: u8,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frames.hash(state);
        self.index.hash(state);
        if self.index.is_none() {
            self.ddof.hash(state);
        }
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
    // Cartesian product (TAG from FA)
    lazy_frame = cartesian_product(lazy_frame)?;
    Ok(lazy_frame)
}

// pub(super) fn cartesian_product(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
//     lazy_frame = lazy_frame
//         .clone()
//         .select([as_struct(vec![
//             col(LABEL),
//             col(FATTY_ACID),
//             col(STEREOSPECIFIC_NUMBER13).alias("Value"),
//         ])
//         .alias(STEREOSPECIFIC_NUMBER1)])
//         .cross_join(
//             lazy_frame.clone().select([as_struct(vec![
//                 col(LABEL),
//                 col(FATTY_ACID),
//                 col(STEREOSPECIFIC_NUMBER2).alias("Value"),
//             ])
//             .alias(STEREOSPECIFIC_NUMBER2)]),
//             None,
//         )
//         .cross_join(
//             lazy_frame.clone().select([as_struct(vec![
//                 col(LABEL),
//                 col(FATTY_ACID),
//                 col(STEREOSPECIFIC_NUMBER13).alias("Value"),
//             ])
//             .alias(STEREOSPECIFIC_NUMBER3)]),
//             None,
//         );
//     // Restruct
//     let label = |name| col(name).struct_().field_by_name(LABEL).alias(name);
//     let fatty_acid = |name| col(name).struct_().field_by_name(FATTY_ACID).alias(name);
//     let value = |name| col(name).struct_().field_by_name("Value");
//     lazy_frame = lazy_frame.select([
//         as_struct(vec![
//             label(STEREOSPECIFIC_NUMBER1),
//             label(STEREOSPECIFIC_NUMBER2),
//             label(STEREOSPECIFIC_NUMBER3),
//         ])
//         .alias(LABEL),
//         as_struct(vec![
//             fatty_acid(STEREOSPECIFIC_NUMBER1),
//             fatty_acid(STEREOSPECIFIC_NUMBER2),
//             fatty_acid(STEREOSPECIFIC_NUMBER3),
//         ])
//         .alias(TRIACYLGLYCEROL),
//         value(STEREOSPECIFIC_NUMBER1)
//             * value(STEREOSPECIFIC_NUMBER2)
//             * value(STEREOSPECIFIC_NUMBER3),
//     ]);
//     Ok(lazy_frame)
// }

fn mean_and_standard_deviation(ddof: u8) -> PolarsResult<[Expr; 3]> {
    let array = || concat_arr(vec![all().exclude_cols([LABEL, TRIACYLGLYCEROL]).as_expr()]);
    Ok([
        col(LABEL),
        col(TRIACYLGLYCEROL),
        as_struct(vec![
            array()?.arr().mean().alias("Mean"),
            array()?.arr().std(ddof).alias("StandardDeviation"),
            array()?.alias("Repetitions"),
        ])
        .alias("Value"),
    ])
}
