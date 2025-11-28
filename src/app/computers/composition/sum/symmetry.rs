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
use std::num::NonZeroI8;
use tracing::instrument;

/// Composition symmetry sum computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition symmetry sum computer
#[derive(Default)]
pub(crate) struct Computer;

// ┌────────────────────────────────┬────────────────────────────────┬────────────────────────────────┐
// │ Label                          ┆ Triacylglycerol                ┆ Value                          │
// │ ---                            ┆ ---                            ┆ ---                            │
// │ struct[3]                      ┆ struct[3]                      ┆ struct[3]                      │
// ╞════════════════════════════════╪════════════════════════════════╪════════════════════════════════╡
impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let lazy_frame = key.frame.data_frame.clone().lazy();
        concat_lf_horizontal(
            [
                aaa(lazy_frame.clone(), key)?,
                aba(lazy_frame.clone(), key)?,
                aab_or_baa(lazy_frame.clone(), key)?,
                abc(lazy_frame, key)?,
            ],
            Default::default(),
        )?
        .collect()
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
            significant: false,
        }
    }
}

/// Composition symmetry sum value
type Value = DataFrame;

fn aaa(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Filter
    lazy_frame = lazy_frame.filter(
        col(LABEL)
            .triacylglycerol()
            .stereospecific_number1()
            .eq(col(LABEL).triacylglycerol().stereospecific_number2())
            .and(
                col(LABEL)
                    .triacylglycerol()
                    .stereospecific_number1()
                    .eq(col(LABEL).triacylglycerol().stereospecific_number3()),
            ),
    );
    // Build
    Ok(lazy_frame.select([build(key)?.alias("AAA")]))
}

fn aba(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Filter
    lazy_frame = lazy_frame.filter(
        col(LABEL)
            .triacylglycerol()
            .stereospecific_number1()
            .eq(col(LABEL).triacylglycerol().stereospecific_number3())
            .and(
                col(LABEL)
                    .triacylglycerol()
                    .stereospecific_number1()
                    .neq(col(LABEL).triacylglycerol().stereospecific_number2()),
            ),
    );
    // Build
    Ok(lazy_frame.select([build(key)?.alias("ABA")]))
}

fn aab_or_baa(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Filter
    lazy_frame = lazy_frame.filter(
        col(LABEL)
            .triacylglycerol()
            .stereospecific_number1()
            .eq(col(LABEL).triacylglycerol().stereospecific_number2())
            .and(
                col(LABEL)
                    .triacylglycerol()
                    .stereospecific_number1()
                    .neq(col(LABEL).triacylglycerol().stereospecific_number3()),
            )
            .or(col(LABEL)
                .triacylglycerol()
                .stereospecific_number3()
                .eq(col(LABEL).triacylglycerol().stereospecific_number2())
                .and(
                    col(LABEL)
                        .triacylglycerol()
                        .stereospecific_number3()
                        .neq(col(LABEL).triacylglycerol().stereospecific_number1()),
                )),
    );
    // Build
    Ok(lazy_frame.select([build(key)?.alias("AAB(BAA)")]))
}

fn abc(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Filter
    lazy_frame = lazy_frame.filter(
        col(LABEL)
            .triacylglycerol()
            .stereospecific_number1()
            .neq(col(LABEL).triacylglycerol().stereospecific_number2())
            .and(
                col(LABEL)
                    .triacylglycerol()
                    .stereospecific_number1()
                    .neq(col(LABEL).triacylglycerol().stereospecific_number3()),
            )
            .and(
                col(LABEL)
                    .triacylglycerol()
                    .stereospecific_number2()
                    .neq(col(LABEL).triacylglycerol().stereospecific_number3()),
            ),
    );
    // Build
    Ok(lazy_frame.select([build(key)?.alias("ABC")]))
}

/// Build
fn build(key: Key) -> PolarsResult<Expr> {
    let sample = || concat_arr(vec![col(r#"^Value\[\d+\]$"#).sum()]);
    Ok(as_struct(vec![
        col(LABEL).implode(),
        as_struct(vec![
            sample()?
                .arr()
                .mean()
                .percent_if(key.percent)
                .precision(key.precision, key.significant)
                .cast(DataType::String)
                .alias("Mean"),
            format_standard_deviation(
                sample()?
                    .arr()
                    .std(key.ddof)
                    .precision(key.precision, key.significant)
                    .alias("StandardDeviation"),
            )?,
            format_sample(
                sample()?.arr().eval(
                    element()
                        .precision(key.precision, key.significant)
                        .cast(DataType::String),
                    false,
                ),
            )?
            .alias("Sample"),
        ])
        .alias("Value"),
    ]))
}
