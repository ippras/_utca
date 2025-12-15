use crate::{
    app::states::calculation::settings::Settings,
    r#const::*,
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

/// Calculation biodiesel sum computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation biodiesel sum computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter threshold
        if key.threshold_filter {
            lazy_frame = lazy_frame.filter(col(THRESHOLD));
        }
        // Compute
        lazy_frame = compute(lazy_frame, key)?;
        // Format
        lazy_frame = format(lazy_frame, key)?;
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation biodiesel sum key
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

/// Calculation biodiesel sum value
type Value = DataFrame;

fn length(data_frame: &DataFrame) -> PolarsResult<usize> {
    fn stereospecific_numbers(data_type: &DataType) -> Option<usize> {
        if let DataType::Struct(fields) = data_type
            && let [mean, standard_deviation, sample] = &**fields
            && mean.name == MEAN
            && mean.dtype == DataType::Float64
            && standard_deviation.name == STANDARD_DEVIATION
            && standard_deviation.dtype == DataType::Float64
            && sample.name == SAMPLE
            && let DataType::Array(box DataType::Float64, length) = sample.dtype
        {
            Some(length)
        } else {
            None
        }
    }

    const SCHEMA: LazyLock<Schema> = LazyLock::new(|| {
        Schema::from_iter([
            Field::new(PlSmallStr::from_static(LABEL), DataType::String),
            field!(FATTY_ACID),
            Field::new(
                PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS123),
                DataType::Struct(vec![
                    Field::new(PlSmallStr::from_static(MEAN), DataType::Float64),
                    Field::new(
                        PlSmallStr::from_static(STANDARD_DEVIATION),
                        DataType::Float64,
                    ),
                    Field::new(
                        PlSmallStr::from_static(SAMPLE),
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

fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let length = length(&key.frame)?;
    // Пока не будет готов
    // https://github.com/pola-rs/polars/pull/23316
    let iter = |expr: Expr| {
        (0..length).map(move |index| {
            expr.clone()
                .struct_()
                .field_by_name(SAMPLE)
                .arr()
                .get(lit(index as IdxSize), false)
        })
    };
    let sample = |name: &str, f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        concat_arr(iter(col(name)).map(f).collect())
    };
    let stereospecific_numbers = |name: &str, f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            sample(name, f)?.arr().mean().alias(MEAN),
            sample(name, f)?
                .arr()
                .std(key.ddof)
                .alias(STANDARD_DEVIATION),
            sample(name, f)?.alias(SAMPLE),
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
    Ok(lazy_frame.select([
        property(cetane_number)?.alias("CetaneNumber"),
        property(cold_filter_plugging_point)?.alias("ColdFilterPluggingPoint"),
        property(degree_of_unsaturation)?.alias("DegreeOfUnsaturation"),
        property(iodine_value)?.alias("IodineValue"),
        property(long_chain_saturated_factor)?.alias("LongChainSaturatedFactor"),
        property(oxidation_stability)?.alias("OxidationStability"),
    ]))
}

fn format(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let stereospecific_numbers = |expr: Expr| -> PolarsResult<Expr> {
        Ok(expr.clone().struct_().with_fields(vec![
            expr.clone()
                .struct_()
                .field_by_name(MEAN)
                .percent(true)
                .precision(key.precision, key.significant)
                .cast(DataType::String),
            format_standard_deviation(
                expr.clone()
                    .struct_()
                    .field_by_name(STANDARD_DEVIATION)
                    .percent(true)
                    .precision(key.precision, key.significant),
            )?,
            format_sample(
                expr.struct_().field_by_name(SAMPLE).arr().eval(
                    element()
                        .percent(true)
                        .precision(key.precision, key.significant)
                        .cast(DataType::String),
                    false,
                ),
            )?,
        ]))
    };
    let property = |name: &str| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            stereospecific_numbers(col(name).struct_().field_by_name(STEREOSPECIFIC_NUMBERS123))?,
            stereospecific_numbers(col(name).struct_().field_by_name(STEREOSPECIFIC_NUMBERS13))?,
            stereospecific_numbers(col(name).struct_().field_by_name(STEREOSPECIFIC_NUMBERS2))?,
        ])
        .alias(name))
    };
    Ok(lazy_frame.select([
        property("CetaneNumber")?,
        property("ColdFilterPluggingPoint")?,
        property("DegreeOfUnsaturation")?,
        property("IodineValue")?,
        property("LongChainSaturatedFactor")?,
        property("OxidationStability")?,
    ]))
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
