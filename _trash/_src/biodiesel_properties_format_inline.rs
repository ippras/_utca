use crate::{
    app::states::calculation::Settings,
    utils::{
        HashedDataFrame,
        polars::{format_sample, format_standard_deviation},
    },
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use std::sync::LazyLock;
use tracing::instrument;

/// Calculation biodiesel properties computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation biodiesel properties computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let length = length(&key.frame)?;
        compute(key, length)
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
    pub(crate) save: bool,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            precision: settings.precision,
            save: settings.threshold.save,
            significant: settings.significant,
        }
    }
}

/// Calculation biodiesel properties value
type Value = DataFrame;

fn length(data_frame: &DataFrame) -> PolarsResult<usize> {
    const SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
        Schema::from_iter([
            Field::new(PlSmallStr::from_static(LABEL), DataType::String),
            field!(FATTY_ACID),
            Field::new(
                PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS123),
                DataType::Struct(vec![
                    Field::new(PlSmallStr::from_static("Mean"), DataType::Float64),
                    Field::new(
                        PlSmallStr::from_static("StandardDeviation"),
                        DataType::Float64,
                    ),
                    Field::new(
                        PlSmallStr::from_static("Array"),
                        DataType::Array(Box::new(DataType::Float64), 0),
                    ),
                ]),
            ),
        ])
    });

    let schema = data_frame.schema();
    if let Some(label) = schema.get(LABEL)
        && *label == DataType::String
        && let Some(fatty_acid) = schema.get(FATTY_ACID)
        && *fatty_acid == data_type!(FATTY_ACID)
        && let Some(length) = schema
            .get(STEREOSPECIFIC_NUMBERS123)
            .and_then(stereospecific_numbers)
        && Some(length)
            == schema
                .get(STEREOSPECIFIC_NUMBERS13)
                .and_then(stereospecific_numbers)
        && Some(length)
            == schema
                .get(STEREOSPECIFIC_NUMBERS2)
                .and_then(stereospecific_numbers)
    {
        return Ok(length);
    }
    polars_bail!(SchemaMismatch: "Invalid calculation properties biodiesel schema: expected `{SCHEMA:?}`, got = `{schema:?}`");
}

fn stereospecific_numbers(data_type: &DataType) -> Option<usize> {
    if let DataType::Struct(fields) = data_type
        && let [mean, standard_deviation, sample] = &**fields
        && mean.name == "Mean"
        && mean.dtype == DataType::Float64
        && standard_deviation.name == "StandardDeviation"
        && standard_deviation.dtype == DataType::Float64
        && sample.name == "Array"
        && let DataType::Array(box DataType::Float64, length) = sample.dtype
    {
        Some(length)
    } else {
        None
    }
}

fn compute(key: Key, length: usize) -> PolarsResult<Value> {
    // Пока не будет готов
    // https://github.com/pola-rs/polars/pull/23316
    let mut lazy_frame = key.frame.data_frame.clone().lazy();
    // Filter minor
    if !key.save {
        // true or null (standard)
        lazy_frame = lazy_frame.filter(col("Filter").or(col("Filter").is_null()));
    }
    // Calculate
    let iter = |expr: Expr| {
        (0..length).map(move |index| {
            expr.clone()
                .struct_()
                .field_by_name("Array")
                .arr()
                .get(lit(index as IdxSize), false)
        })
    };
    let sample = |name: &str, f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        concat_arr(iter(col(name)).map(f).collect())
    };
    let stereospecific_numbers = |name: &str, f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            sample(name, f)?
                .arr()
                .mean()
                .precision(key.precision, key.significant)
                .cast(DataType::String)
                .alias("Mean"),
            format_standard_deviation(
                sample(name, f)?
                    .arr()
                    .std(key.ddof)
                    .precision(key.precision, key.significant),
            )?
            .alias("StandardDeviation"),
            format_sample(
                sample(name, f)?.arr().eval(
                    element()
                        .precision(key.precision, key.significant)
                        .cast(DataType::String),
                    false,
                ),
            )?
            .alias("Sample"),
        ])
        .alias(name))
    };
    let property = |f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS123, f)?,
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS13, f)?,
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS2, f)?,
        ]))
    };
    lazy_frame = lazy_frame.select([
        property(cetane_number)?.alias("CetaneNumber"),
        property(cold_filter_plugging_point)?.alias("ColdFilterPluggingPoint"),
        property(degree_of_unsaturation)?.alias("DegreeOfUnsaturation"),
        property(iodine_value)?.alias("IodineValue"),
        property(long_chain_saturated_factor)?.alias("LongChainSaturatedFactor"),
        property(oxidation_stability)?.alias("OxidationStability"),
    ]);
    lazy_frame.collect()
}

// [1|1|1]
// fn temp(name: &str) -> PolarsResult<Expr> {
//     as_struct(vec![
//         col(name)
//             .arr()
//             .mean()
//             .precision(key.precision, key.significant)
//             .cast(DataType::String)
//             .alias("Mean"),
//         col(name),
//     ])
// }

fn cetane_number(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().cetane_number(expr).percent()
}

fn cold_filter_plugging_point(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .cold_filter_plugging_point(expr)
        .percent()
}

fn degree_of_unsaturation(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .degree_of_unsaturation(expr)
        .percent()
}

fn iodine_value(expr: Expr) -> Expr {
    BiodieselProperties::iodine_value(col(FATTY_ACID).fatty_acid(), expr).percent()
}

fn long_chain_saturated_factor(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .long_chain_saturated_factor(expr)
        .percent()
}

fn oxidation_stability(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .oxidation_stability(expr)
        .percent()
}
