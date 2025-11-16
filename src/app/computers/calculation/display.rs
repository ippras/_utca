use crate::{
    app::states::calculation::Settings,
    utils::{HashedDataFrame, polars::SchemaExt},
};
use egui::util::cache::{ComputerMut, FrameCache};
use itertools::Itertools;
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::{ExprExt as _, ExprIfExt as _};
use std::sync::LazyLock;
use tracing::instrument;

const SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::from_iter([
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
        Field::new(
            PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS2),
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
        Field::new(
            PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS13),
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
        Field::new(
            PlSmallStr::from_static("Factors"),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static("Enrichment"),
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
                Field::new(
                    PlSmallStr::from_static("Selectivity"),
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
            ]),
        ),
    ]))
});

/// Display calculation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Display calculation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        let length = schema(&key.frame)?;
        let body = lazy_frame.clone().select(format(key)?);
        let sum = lazy_frame.select(format_sum(key, length)?);
        lazy_frame = concat_lf_diagonal([body, sum], Default::default())?;
        let data_frame = lazy_frame.collect()?;
        Ok(data_frame)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Display calculation key
#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) kind: Kind,
    pub(crate) ddof: u8,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    fn new(frame: &'a HashedDataFrame, kind: Kind, settings: &Settings) -> Self {
        Self {
            frame,
            kind,
            ddof: 1,
            percent: settings.percent,
            precision: settings.float_precision,
            significant: settings.significant,
        }
    }

    pub(crate) fn stereospecific_numbers123(
        frame: &'a HashedDataFrame,
        settings: &Settings,
    ) -> Self {
        Self::new(frame, Kind::StereospecificNumbers123, settings)
    }

    pub(crate) fn stereospecific_numbers13(
        frame: &'a HashedDataFrame,
        settings: &Settings,
    ) -> Self {
        Self::new(frame, Kind::StereospecificNumbers13, settings)
    }

    pub(crate) fn stereospecific_numbers2(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self::new(frame, Kind::StereospecificNumbers2, settings)
    }

    pub(crate) fn enrichment_factor(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self::new(frame, Kind::EnrichmentFactor, settings)
    }

    pub(crate) fn selectivity_factor(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self::new(frame, Kind::SelectivityFactor, settings)
    }
}

/// Display calculation value
type Value = DataFrame;

// Display kind
#[derive(Clone, Copy, Debug, Hash)]
pub enum Kind {
    StereospecificNumbers123,
    StereospecificNumbers2,
    StereospecificNumbers13,
    EnrichmentFactor,
    SelectivityFactor,
}

fn schema(data_frame: &DataFrame) -> PolarsResult<usize> {
    let schema = data_frame.schema();
    let _cast = schema.matches_schema(&SCHEMA)?;
    let length = schema
        .array_lengths_recursive()?
        .into_iter()
        .all_equal_value()
        .map_err(|lengths| polars_err!(SchemaMismatch: "Invalid array lengths: expected all equal, got = {lengths:?}"))?;
    Ok(length)
}

fn format_sum(key: Key, length: usize) -> PolarsResult<[Expr; 3]> {
    let array = concat_arr(
        (0..length)
            .map(|index| {
                expr(key)
                    .struct_()
                    .field_by_name("Array")
                    .arr()
                    .get(lit(index as IdxSize), false)
                    .sum()
            })
            .collect(),
    )?;
    Ok([
        format_mean(expr(key).struct_().field_by_name("Mean").sum(), key),
        format_standard_deviation(array.clone().arr().std(key.ddof), key)?,
        format_array(array, key)?,
    ])
}

fn format(mut key: Key) -> PolarsResult<[Expr; 4]> {
    let calculation = format_calculation(key)?;
    if let Kind::EnrichmentFactor | Kind::SelectivityFactor = key.kind {
        key.percent = false;
    };
    Ok([
        format_mean(expr(key).struct_().field_by_name("Mean"), key),
        format_standard_deviation(expr(key).struct_().field_by_name("StandardDeviation"), key)?,
        format_array(expr(key).struct_().field_by_name("Array"), key)?,
        calculation,
    ])
}

fn format_mean(expr: Expr, key: Key) -> Expr {
    format_float(expr, key)
}

fn format_standard_deviation(expr: Expr, key: Key) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        expr.clone().is_not_null(),
        format_str("±{}", [format_float(expr, key)])?.alias("StandardDeviation"),
        lit(NULL),
    ))
}

fn format_array(expr: Expr, key: Key) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        expr.clone().arr().len().neq(1),
        format_str(
            "[{}]",
            [expr
                .arr()
                .to_list()
                .list()
                .eval(format_float(col(""), key))
                .list()
                .join(lit(", "), false)],
        )?,
        lit(NULL),
    )
    .alias("Array"))
}

fn format_calculation(key: Key) -> PolarsResult<Expr> {
    let mean = |name| col(name).struct_().field_by_name("Mean");
    let standard_deviation = |name| col(name).struct_().field_by_name("StandardDeviation");
    let predicate = standard_deviation(STEREOSPECIFIC_NUMBERS2)
        .is_null()
        .or(standard_deviation(STEREOSPECIFIC_NUMBERS123).is_null());
    Ok(match key.kind {
        Kind::StereospecificNumbers13 => ternary_expr(
            predicate,
            format_str(
                "(3 * {} - {}) / 2",
                [
                    format_float(mean(STEREOSPECIFIC_NUMBERS123), key),
                    format_float(mean(STEREOSPECIFIC_NUMBERS2), key),
                ],
            )?,
            lit(NULL),
        ),
        Kind::EnrichmentFactor => ternary_expr(
            predicate,
            format_str(
                "{} / (3 * {})",
                [
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS2),
                        Key {
                            kind: Kind::StereospecificNumbers2,
                            ..key
                        },
                    ),
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS123),
                        Key {
                            kind: Kind::StereospecificNumbers123,
                            ..key
                        },
                    ),
                ],
            )?,
            lit(NULL),
        ),
        Kind::SelectivityFactor => ternary_expr(
            predicate,
            format_str(
                "({} * {}) / ({} * {})",
                [
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS2),
                        Key {
                            kind: Kind::StereospecificNumbers2,
                            ..key
                        },
                    ),
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS123)
                            .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                            .sum(),
                        Key {
                            kind: Kind::StereospecificNumbers123,
                            ..key
                        },
                    ),
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS123),
                        Key {
                            kind: Kind::StereospecificNumbers123,
                            ..key
                        },
                    ),
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS2)
                            .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                            .sum(),
                        Key {
                            kind: Kind::StereospecificNumbers2,
                            ..key
                        },
                    ),
                ],
            )?,
            lit(NULL),
        ),
        _ => lit(NULL),
    }
    .alias("Calculation"))
}

fn format_float(expr: Expr, key: Key) -> Expr {
    expr.percent_if(key.percent)
        .precision(key.precision, key.significant)
        .cast(DataType::String)
}

fn expr(key: Key) -> Expr {
    match key.kind {
        Kind::StereospecificNumbers123 => col(STEREOSPECIFIC_NUMBERS123),
        Kind::StereospecificNumbers13 => col(STEREOSPECIFIC_NUMBERS13),
        Kind::StereospecificNumbers2 => col(STEREOSPECIFIC_NUMBERS2),
        Kind::EnrichmentFactor => col("Factors").struct_().field_by_name("Enrichment"),
        Kind::SelectivityFactor => col("Factors").struct_().field_by_name("Selectivity"),
    }
}
