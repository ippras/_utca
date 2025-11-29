use crate::{
    app::states::composition::Settings,
    utils::{
        HashedDataFrame,
        polars::{format_sample, format_standard_deviation},
    },
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::{ExprExt, ExprIfExt};
use tracing::instrument;

/// Composition symmetry sum computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition symmetry sum computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        // ┌────────────────────────────────┬────────────────────────────────┬────────────────────────────────┐
        // │ Label                          ┆ Triacylglycerol                ┆ Value                          │
        // │ ---                            ┆ ---                            ┆ ---                            │
        // │ struct[3]                      ┆ struct[3]                      ┆ struct[3]                      │
        // ╞════════════════════════════════╪════════════════════════════════╪════════════════════════════════╡
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = compute(lazy_frame, key)?;
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Composition symmetry sum key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            percent: settings.percent,
            precision: settings.precision,
            significant: settings.significant,
        }
    }
}

/// Composition symmetry sum value
type Value = DataFrame;

fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let sn1 = col(LABEL).triacylglycerol().stereospecific_number1();
    let sn2 = col(LABEL).triacylglycerol().stereospecific_number2();
    let sn3 = col(LABEL).triacylglycerol().stereospecific_number3();
    let sample = concat_arr(vec![col(r#"^Value\[\d+\]$"#).sum()])?;
    // Group, format, sort
    Ok(lazy_frame
        .group_by([
            when(sn1.clone().eq(sn2.clone()).and(sn2.clone().eq(sn3.clone())))
                .then(lit("AAA"))
                .when(
                    sn1.clone()
                        .eq(sn3.clone())
                        .and(sn2.clone().neq(sn3.clone())),
                )
                .then(lit("ABA"))
                .when(
                    sn1.clone()
                        .neq(sn3.clone())
                        .and(sn1.eq(sn2.clone()).or(sn2.eq(sn3))),
                )
                .then(lit("AAB(BAA)"))
                .otherwise(lit("ABC"))
                .alias("Group"),
        ])
        .agg([
            col(LABEL),
            as_struct(vec![
                sample
                    .clone()
                    .arr()
                    .mean()
                    .percent_if(key.percent)
                    .precision(key.precision, key.significant)
                    .cast(DataType::String)
                    .alias("Mean"),
                format_standard_deviation(
                    sample
                        .clone()
                        .arr()
                        .std(key.ddof)
                        .percent_if(key.percent)
                        .precision(key.precision, key.significant)
                        .alias("StandardDeviation"),
                )?,
                format_sample(
                    sample.arr().eval(
                        element()
                            .percent_if(key.percent)
                            .precision(key.precision, key.significant)
                            .cast(DataType::String),
                        false,
                    ),
                )?
                .alias("Sample"),
            ])
            .alias("Value"),
        ])
        .sort(["Group"], Default::default()))
}
