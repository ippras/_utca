use crate::{
    app::states::{
        calculation::settings::Threshold,
        composition::settings::{Composition, Settings, Stereospecificity},
    },
    r#const::{KEY, KEYS, SPECIES, THRESHOLD, VALUE, VALUES},
    utils::{
        HashedDataFrame,
        polars::{MeanAndStandardDeviationOptions, mean_and_standard_deviation},
    },
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;

/// Table composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Table composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // | Threshold | Keys      | Values              | Species         |
        // | ---       | ---       | ---                 | ---             |
        // | bool      | struct[1] | list[array[f64, 3]] | list[struct[4]] |
        lazy_frame = filter_and_sort(lazy_frame, key);
        lazy_frame = compute(lazy_frame, key)?;
        // | Threshold | Index | Key[n] | Value[n]  | Species         |
        // | ---       | ---   | ---    | ---       | ---             |
        // | bool      | u32   | str    | struct[3] | list[struct[4]] |
        let data_frame = lazy_frame.collect()?;
        HashedDataFrame::new(data_frame)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Table composition key
#[derive(Clone, Hash, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) compositions: &'a Vec<Composition>,
    pub(crate) significant: bool,
    pub(crate) threshold: &'a Threshold,
}

impl<'a> Key<'a> {
    pub(crate) fn new(data_frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame: data_frame,
            ddof: settings.ddof,
            percent: settings.percent,
            precision: settings.precision,
            compositions: &settings.compositions,
            significant: settings.significant,
            threshold: &settings.threshold,
        }
    }
}

impl From<Key<'_>> for MeanAndStandardDeviationOptions {
    fn from(key: Key) -> Self {
        Self {
            ddof: key.ddof,
            percent: key.percent,
            precision: key.precision,
            significant: key.significant,
        }
    }
}

/// Table composition value
type Value = HashedDataFrame;

fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    concat_lf_diagonal(
        [body(lazy_frame.clone(), key)?, sum(lazy_frame, key)?],
        UnionArgs::default(),
    )
}

// Filter and sort threshold (major, minor)
fn filter_and_sort(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    if key.threshold.filter {
        lazy_frame.filter(col(THRESHOLD))
    } else if key.threshold.sort {
        lazy_frame.sort_by_exprs(
            [col(THRESHOLD)],
            SortMultipleOptions::default()
                .with_maintain_order(true)
                .with_order_reversed(),
        )
    } else {
        lazy_frame
    }
}

/// Body
fn body(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let mut exprs = Vec::new();
    // Threshold
    exprs.push(col(THRESHOLD));
    for index in 0..key.compositions.len() {
        // Key
        let triacylglycerol = col(KEYS)
            .struct_()
            .field_by_index(index as _)
            .triacylglycerol();
        let args = [
            triacylglycerol.clone().stereospecific_number1(),
            triacylglycerol.clone().stereospecific_number2(),
            triacylglycerol.stereospecific_number3(),
        ];
        exprs.push(
            match key.compositions[index].stereospecificity() {
                Some(Stereospecificity::Stereo) => format_str("[{}; {}; {}]", args)?,
                Some(Stereospecificity::Positional) => format_str("[{}/2; {}; {}/2]", args)?,
                None => format_str("[{}/3; {}/3; {}/3]", args)?,
            }
            .alias(format!("{KEY}[{index}]")),
        );
        // Value
        exprs.push(
            mean_and_standard_deviation(col(VALUES).list().get(lit(index as IdxSize), false), key)
                .alias(format!("{VALUE}[{index}]")),
        );
    }
    // Species
    // let mut species = species(key)?;
    // if key.threshold.filter {
    //     species = species
    //         .list()
    //         .eval(element().filter(element().struct_().field_by_name(THRESHOLD)));
    // }
    // species = species.list().eval(as_struct(vec![
    //     // Label
    //     {
    //         let label = element().struct_().field_by_name(LABEL).triacylglycerol();
    //         format_str(
    //             "[{}; {}; {}]",
    //             [
    //                 label.clone().stereospecific_number1(),
    //                 label.clone().stereospecific_number2(),
    //                 label.stereospecific_number3(),
    //             ],
    //         )?
    //         .alias(LABEL)
    //     },
    //     // Triacylglycerol
    //     {
    //         let triacylglycerol = {
    //             element()
    //                 .struct_()
    //                 .field_by_name(TRIACYLGLYCEROL)
    //                 .triacylglycerol()
    //         };
    //         format_str(
    //             "[{}; {}; {}]",
    //             [
    //                 triacylglycerol
    //                     .clone()
    //                     .stereospecific_number1()
    //                     .fatty_acid()
    //                     .format(),
    //                 triacylglycerol
    //                     .clone()
    //                     .stereospecific_number2()
    //                     .fatty_acid()
    //                     .format(),
    //                 triacylglycerol
    //                     .stereospecific_number3()
    //                     .fatty_acid()
    //                     .format(),
    //             ],
    //         )?
    //         .alias(TRIACYLGLYCEROL)
    //     },
    //     // Value
    //     mean_and_standard_deviation(element().struct_().field_by_name(VALUE), key).alias(VALUE),
    //     // Threshold
    //     element().struct_().field_by_name(THRESHOLD),
    // ]));
    // if key.threshold.sort {
    //     species = species.list().sort(
    //         SortOptions::default()
    //             .with_maintain_order(true)
    //             .with_order_reversed(),
    //     )
    // }
    exprs.push(species(key)?);
    println!(
        "!!!!!!1: {}",
        lazy_frame
            .clone()
            .select([species(key)?])
            .collect()
            .unwrap()
    );
    Ok(lazy_frame.select(exprs))
}

fn species(key: Key) -> PolarsResult<Expr> {
    let mut expr = col(SPECIES);
    // Filter and sort threshold (major, minor)
    if key.threshold.filter {
        expr = expr
            .list()
            .eval(element().filter(element().struct_().field_by_name(THRESHOLD)));
    } else if key.threshold.sort {
        expr = expr.list().eval(
            element().sort_by(
                [element().struct_().field_by_name(THRESHOLD)],
                SortMultipleOptions::default()
                    .with_maintain_order(true)
                    .with_order_reversed(),
            ),
        );
    }
    expr = expr.list().eval(as_struct(vec![
        // Threshold
        element().struct_().field_by_name(THRESHOLD),
        // Label
        {
            let label = element().struct_().field_by_name(LABEL).triacylglycerol();
            format_str(
                "[{}; {}; {}]",
                [
                    label.clone().stereospecific_number1(),
                    label.clone().stereospecific_number2(),
                    label.stereospecific_number3(),
                ],
            )?
            .alias(LABEL)
        },
        // Triacylglycerol
        {
            let triacylglycerol = {
                element()
                    .struct_()
                    .field_by_name(TRIACYLGLYCEROL)
                    .triacylglycerol()
            };
            format_str(
                "[{}; {}; {}]",
                [
                    triacylglycerol
                        .clone()
                        .stereospecific_number1()
                        .fatty_acid()
                        .format(),
                    triacylglycerol
                        .clone()
                        .stereospecific_number2()
                        .fatty_acid()
                        .format(),
                    triacylglycerol
                        .stereospecific_number3()
                        .fatty_acid()
                        .format(),
                ],
            )?
            .alias(TRIACYLGLYCEROL)
        },
        // Value
        mean_and_standard_deviation(element().struct_().field_by_name(VALUE), key).alias(VALUE),
    ]));
    Ok(expr)
}

/// Sum
fn sum(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame.select([mean_and_standard_deviation(
        eval_arr(col(VALUES).list().last(), |mut element| {
            if key.threshold.filter {
                element = element.filter(THRESHOLD);
            }
            element.sum()
        })?,
        key,
    )
    .alias(format!("{VALUE}[{}]", key.compositions.len() - 1))]))
}
