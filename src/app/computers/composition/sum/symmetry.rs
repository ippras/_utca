use crate::{
    app::states::composition::{Order, Settings, Sort},
    r#const::{GROUP, MEAN, SAMPLE, STANDARD_DEVIATION, TRIACYLGLYCEROLS, VALUE},
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use tracing::instrument;

/// Composition symmetry sum computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition symmetry sum computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        // | Label                          | Triacylglycerol                | Value                          |
        // | ---                            | ---                            | ---                            |
        // | struct[3]                      | struct[3]                      | array[f64, 3]                  |
        // |--------------------------------|--------------------------------|--------------------------------|
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        println!("SYM 0: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = compute(lazy_frame, key)?;
        println!("SYM 1: {}", lazy_frame.clone().collect().unwrap());
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
    pub(crate) order: Order,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) sort: Sort,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            order: settings.order,
            percent: settings.percent,
            precision: settings.precision,
            significant: settings.significant,
            sort: settings.sort,
        }
    }
}

/// Composition symmetry sum value
///
/// | Group | Label           | Value     |
/// | ---   | ---             | ---       |
/// | str   | list[struct[3]] | struct[3] |
/// |-------|-----------------|-----------|
type Value = DataFrame;

fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let sn1 = col(LABEL).triacylglycerol().stereospecific_number1();
    let sn2 = col(LABEL).triacylglycerol().stereospecific_number2();
    let sn3 = col(LABEL).triacylglycerol().stereospecific_number3();
    let mut triacylglycerols = as_struct(vec![
        col(LABEL),
        mean_and_standard_deviation(col(VALUE), key).alias(VALUE),
    ])
    .alias(TRIACYLGLYCEROLS);
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
                .alias(GROUP),
        ])
        .agg([
            mean_and_standard_deviation(eval_arr(col(VALUE), |element| element.sum())?, key)
                .alias(VALUE),
            sort(
                as_struct(vec![
                    col(LABEL),
                    mean_and_standard_deviation(col(VALUE), key).alias(VALUE),
                ])
                .alias(TRIACYLGLYCEROLS),
                key,
            ),
        ])
        .sort([GROUP], SortMultipleOptions::new()))
}

// as_struct(vec![
//     sample
//         .clone()
//         .arr()
//         .mean()
//         .percent(key.percent)
//         .precision(key.precision, key.significant)
//         .cast(DataType::String)
//         .alias(MEAN),
//     format_standard_deviation(
//         sample
//             .clone()
//             .arr()
//             .std(key.ddof)
//             .percent(key.percent)
//             .precision(key.precision, key.significant)
//             .alias(STANDARD_DEVIATION),
//     )?,
//     format_sample(
//         sample.arr().eval(
//             element()
//                 .percent(key.percent)
//                 .precision(key.precision, key.significant)
//                 .cast(DataType::String),
//             false,
//         ),
//     )?
//     .alias(SAMPLE),
// ])
// .alias("Value"),
fn mean_and_standard_deviation(array: Expr, key: Key) -> Expr {
    as_struct(vec![
        array
            .clone()
            .arr()
            .mean()
            .percent(key.percent)
            .precision(key.precision, key.significant)
            .alias(MEAN),
        array
            .clone()
            .arr()
            .std(key.ddof)
            .percent(key.percent)
            .precision(key.precision + 1, key.significant)
            .alias(STANDARD_DEVIATION),
        array
            .arr()
            .eval(
                element()
                    .percent(key.percent)
                    .precision(key.precision, key.significant),
                false,
            )
            .alias(SAMPLE),
    ])
}

fn sort(expr: Expr, key: Key) -> Expr {
    let mut sort_options = SortMultipleOptions::default();
    if let Order::Descending = key.order {
        sort_options = sort_options
            .with_maintain_order(true)
            .with_order_descending(true)
            .with_nulls_last(true);
    }
    match key.sort {
        Sort::Key => expr.sort_by([col(LABEL)], sort_options),
        Sort::Value => expr.sort_by([col(VALUE)], sort_options),
    }
}
