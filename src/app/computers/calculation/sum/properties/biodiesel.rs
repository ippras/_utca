use crate::{
    app::states::calculation::settings::Settings,
    r#const::{MEAN, NAME, SAMPLE, STANDARD_DEVIATION, THRESHOLD},
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use tracing::instrument;

const BIODIESEL_PROPERTIES: [&str; 6] = [
    "CetaneNumber",
    "ColdFilterPluggingPoint",
    "DegreeOfUnsaturation",
    "IodineValue",
    "LongChainSaturatedFactor",
    "OxidationStability",
];

/// Calculation biodiesel properties computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation biodiesel properties computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter
        lazy_frame = filter(lazy_frame, key);
        // Compute
        lazy_frame = compute(lazy_frame, key)?;
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation biodiesel properties key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) threshold_filter: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            precision: settings.precision,
            significant: settings.significant,
            threshold_filter: settings.threshold.filter,
        }
    }
}

/// Calculation biodiesel properties value
type Value = DataFrame;

fn filter(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    if key.threshold_filter {
        lazy_frame.filter(col(THRESHOLD))
    } else {
        lazy_frame
    }
}

// Пока не будет готов
// https://github.com/pola-rs/polars/pull/23316
fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let mut exprs = Vec::with_capacity(4);
    // Names
    exprs.push(lit(
        Series::from_iter(BIODIESEL_PROPERTIES).with_name(PlSmallStr::from_static(NAME))
    ));
    // Stereospecific numbers
    for stereospecific_numbers in [
        STEREOSPECIFIC_NUMBERS123,
        STEREOSPECIFIC_NUMBERS13,
        STEREOSPECIFIC_NUMBERS2,
    ] {
        let expr = concat_arr(
            BIODIESEL_PROPERTIES
                .try_map(|property| -> PolarsResult<_> {
                    let array = eval_arr(
                        col(stereospecific_numbers).struct_().field_by_name(SAMPLE),
                        |expr| match property {
                            "CetaneNumber" => cetane_number(expr),
                            "ColdFilterPluggingPoint" => cold_filter_plugging_point(expr),
                            "DegreeOfUnsaturation" => degree_of_unsaturation(expr),
                            "IodineValue" => iodine_value(expr),
                            "LongChainSaturatedFactor" => long_chain_saturated_factor(expr),
                            "OxidationStability" => oxidation_stability(expr),
                            _ => unreachable!(),
                        },
                    )?;
                    Ok(as_struct(vec![
                        array
                            .clone()
                            .arr()
                            .mean()
                            .precision(key.precision, key.significant)
                            .alias(MEAN),
                        array
                            .clone()
                            .arr()
                            .std(key.ddof)
                            .precision(key.precision + 1, key.significant)
                            .alias(STANDARD_DEVIATION),
                        array
                            .arr()
                            .eval(element().precision(key.precision, key.significant), false)
                            .alias(SAMPLE),
                    ]))
                })?
                .to_vec(),
        )?
        .explode()
        .alias(stereospecific_numbers);
        exprs.push(expr);
    }
    Ok(lazy_frame.select(exprs))
}

fn cetane_number(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().cetane_number(expr)
}

fn cold_filter_plugging_point(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .cold_filter_plugging_point(expr)
}

fn degree_of_unsaturation(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().degree_of_unsaturation(expr)
}

fn iodine_value(expr: Expr) -> Expr {
    BiodieselProperties::iodine_value(col(FATTY_ACID).fatty_acid(), expr)
}

fn long_chain_saturated_factor(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .long_chain_saturated_factor(expr)
}

fn oxidation_stability(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().oxidation_stability(expr)
}
