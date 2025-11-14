use crate::{
    app::states::calculation::{Correlation, Settings},
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::ExprExt;
use tracing::instrument;

/// Calculation correlation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation correlation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        compute(key)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation correlation key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) chaddock: bool,
    pub(crate) correlation: Correlation,
    pub(crate) precision: usize,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            chaddock: settings.chaddock,
            correlation: settings.correlation,
            precision: settings.precision,
        }
    }
}

/// Calculation correlation value
type Value = DataFrame;

fn compute(key: Key) -> PolarsResult<Value> {
    let labels = key.frame.data_frame[LABEL].as_materialized_series();
    let mut lazy_frame = key.frame.data_frame.clone().lazy();
    // Select
    lazy_frame = lazy_frame.select([
        col(LABEL),
        col(STEREOSPECIFIC_NUMBERS123)
            .struct_()
            .field_by_name("Array"),
    ]);
    // Cross join
    lazy_frame = lazy_frame
        .clone()
        .select([col(LABEL).name().suffix("[1]"), col("Array").alias("[1]")])
        .cross_join(
            lazy_frame.select([col(LABEL).name().suffix("[2]"), col("Array").alias("[2]")]),
            None,
        )
        .explode(cols(["[1]", "[2]"]));
    // Correlation
    lazy_frame = lazy_frame
        .group_by_stable([col("Label[1]"), col("Label[2]")])
        .agg([match key.correlation {
            Correlation::Pearson => pearson_corr(col("[1]"), col("[2]")),
            Correlation::SpearmanRank => spearman_rank_corr(col("[1]"), col("[2]"), false),
        }
        .alias("Correlation")]);
    // Pivot
    lazy_frame = lazy_frame.pivot(
        by_name(["Label[2]"], true),
        Arc::new(df! { "" => labels }?),
        by_name(["Label[1]"], true),
        by_name(["Correlation"], true),
        element().item(true),
        true,
        PlSmallStr::EMPTY,
    );
    // Format
    lazy_frame = lazy_frame.select([
        col("Label[1]").alias(LABEL),
        dtype_col(&DataType::Float64)
            .as_selector()
            .as_expr()
            .precision(key.precision, false),
    ]);
    lazy_frame.collect()
}
