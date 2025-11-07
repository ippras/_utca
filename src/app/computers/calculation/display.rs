use crate::{
    app::panes::calculation::state::Settings as CalculationSettings,
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
        // println!("Display 0: {}", lazy_frame.clone().collect().unwrap());
        let length = schema(&key.frame)?;
        let body = lazy_frame.clone().select(format(key.settings)?);
        let sum = lazy_frame.select(format_sum(key.settings, length)?);
        lazy_frame = concat_lf_diagonal([body, sum], UnionArgs::default())?;
        // println!("Display 1: {}", lazy_frame.clone().collect().unwrap());
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
    pub(crate) settings: Settings,
}

/// Display calculation settings
#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct Settings {
    pub(crate) kind: Kind,
    pub(crate) ddof: u8,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl Settings {
    fn new(kind: Kind, settings: &CalculationSettings) -> Self {
        Self {
            kind,
            ddof: 1,
            percent: settings.percent,
            precision: settings.precision,
            significant: settings.significant,
        }
    }

    pub(crate) fn stereospecific_numbers123(settings: &CalculationSettings) -> Self {
        Self::new(Kind::StereospecificNumbers123, settings)
    }

    pub(crate) fn stereospecific_numbers13(settings: &CalculationSettings) -> Self {
        Self::new(Kind::StereospecificNumbers13, settings)
    }

    pub(crate) fn stereospecific_numbers2(settings: &CalculationSettings) -> Self {
        Self::new(Kind::StereospecificNumbers2, settings)
    }

    pub(crate) fn enrichment_factor(settings: &CalculationSettings) -> Self {
        Self::new(Kind::EnrichmentFactor, settings)
    }

    pub(crate) fn selectivity_factor(settings: &CalculationSettings) -> Self {
        Self::new(Kind::SelectivityFactor, settings)
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

fn format_sum(settings: Settings, length: usize) -> PolarsResult<[Expr; 3]> {
    let array = concat_arr(
        (0..length)
            .map(|index| {
                expr(settings)
                    .struct_()
                    .field_by_name("Array")
                    .arr()
                    .get(lit(index as IdxSize), false)
                    .sum()
            })
            .collect(),
    )?;
    Ok([
        format_mean(
            expr(settings).struct_().field_by_name("Mean").sum(),
            settings,
        ),
        format_standard_deviation(array.clone().arr().std(settings.ddof), settings)?,
        format_array(array, settings)?,
    ])
}

fn format(mut settings: Settings) -> PolarsResult<[Expr; 4]> {
    let calculation = format_calculation(settings)?;
    if let Kind::EnrichmentFactor | Kind::SelectivityFactor = settings.kind {
        settings.percent = false;
    };
    Ok([
        format_mean(expr(settings).struct_().field_by_name("Mean"), settings),
        format_standard_deviation(
            expr(settings).struct_().field_by_name("StandardDeviation"),
            settings,
        )?,
        format_array(expr(settings).struct_().field_by_name("Array"), settings)?,
        calculation,
    ])
}

fn format_mean(expr: Expr, settings: Settings) -> Expr {
    format_float(expr, settings)
}

fn format_standard_deviation(expr: Expr, settings: Settings) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        expr.clone().is_not_null(),
        format_str("±{}", [format_float(expr, settings)])?.alias("StandardDeviation"),
        lit(NULL),
    ))
}

fn format_array(expr: Expr, settings: Settings) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        expr.clone().arr().len().neq(1),
        format_str(
            "[{}]",
            [expr
                .arr()
                .to_list()
                .list()
                .eval(format_float(col(""), settings))
                .list()
                .join(lit(", "), false)],
        )?,
        lit(NULL),
    )
    .alias("Array"))
}

fn format_calculation(settings: Settings) -> PolarsResult<Expr> {
    let mean = |name| col(name).struct_().field_by_name("Mean");
    let standard_deviation = |name| col(name).struct_().field_by_name("StandardDeviation");
    let predicate = standard_deviation(STEREOSPECIFIC_NUMBERS2)
        .is_null()
        .or(standard_deviation(STEREOSPECIFIC_NUMBERS123).is_null());
    Ok(match settings.kind {
        Kind::StereospecificNumbers13 => ternary_expr(
            predicate,
            format_str(
                "(3 * {} - {}) / 2",
                [
                    format_float(mean(STEREOSPECIFIC_NUMBERS123), settings),
                    format_float(mean(STEREOSPECIFIC_NUMBERS2), settings),
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
                        Settings {
                            kind: Kind::StereospecificNumbers2,
                            ..settings
                        },
                    ),
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS123),
                        Settings {
                            kind: Kind::StereospecificNumbers123,
                            ..settings
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
                        Settings {
                            kind: Kind::StereospecificNumbers2,
                            ..settings
                        },
                    ),
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS123)
                            .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                            .sum(),
                        Settings {
                            kind: Kind::StereospecificNumbers123,
                            ..settings
                        },
                    ),
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS123),
                        Settings {
                            kind: Kind::StereospecificNumbers123,
                            ..settings
                        },
                    ),
                    format_float(
                        mean(STEREOSPECIFIC_NUMBERS2)
                            .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                            .sum(),
                        Settings {
                            kind: Kind::StereospecificNumbers2,
                            ..settings
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

fn format_float(expr: Expr, settings: Settings) -> Expr {
    expr.percent_if(settings.percent)
        .precision(settings.precision, settings.significant)
        .cast(DataType::String)
}

fn expr(settings: Settings) -> Expr {
    match settings.kind {
        Kind::StereospecificNumbers123 => col(STEREOSPECIFIC_NUMBERS123),
        Kind::StereospecificNumbers13 => col(STEREOSPECIFIC_NUMBERS13),
        Kind::StereospecificNumbers2 => col(STEREOSPECIFIC_NUMBERS2),
        Kind::EnrichmentFactor => col("Factors").struct_().field_by_name("Enrichment"),
        Kind::SelectivityFactor => col("Factors").struct_().field_by_name("Selectivity"),
    }
}
