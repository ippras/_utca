use crate::{
    app::states::calculation::settings::{Settings, Threshold},
    r#const::*,
    utils::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use std::sync::LazyLock;
use tracing::instrument;

const SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::from_iter([
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
        Field::new(
            PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS2),
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
        Field::new(
            PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS13),
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
        Field::new(
            PlSmallStr::from_static(FACTORS),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(ENRICHMENT),
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
                Field::new(
                    PlSmallStr::from_static(SELECTIVITY),
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
            ]),
        ),
        Field::new(
            PlSmallStr::from_static(STANDARD),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(FACTOR),
                    DataType::Array(Box::new(DataType::Float64), 0),
                ),
                Field::new(PlSmallStr::from_static(MASK), DataType::Boolean),
            ]),
        ),
        Field::new(PlSmallStr::from_static(THRESHOLD), DataType::Boolean),
    ]))
});

const STEREOSPECIFIC_NUMBERS: [&str; 3] = [
    STEREOSPECIFIC_NUMBERS123,
    STEREOSPECIFIC_NUMBERS2,
    STEREOSPECIFIC_NUMBERS13,
];

/// Table calculation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Table calculation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        schema(&key.frame)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = filter_and_sort(lazy_frame, key);
        lazy_frame = format(lazy_frame, key)?;
        let data_frame = lazy_frame.collect()?;
        Ok(data_frame)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Table calculation key
#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) threshold: &'a Threshold,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            percent: settings.percent,
            precision: settings.precision,
            significant: settings.significant,
            threshold: &settings.threshold,
        }
    }
}

/// Table calculation value
type Value = DataFrame;

fn schema(data_frame: &DataFrame) -> PolarsResult<()> {
    let _cast = data_frame.schema().matches_schema(&SCHEMA)?;
    Ok(())
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

fn format(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    concat_lf_diagonal(
        [body(lazy_frame.clone(), key)?, sum(lazy_frame, key)?],
        UnionArgs::default(),
    )
}

fn body(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Stereospecific numbers
    lazy_frame = lazy_frame.with_columns(
        STEREOSPECIFIC_NUMBERS
            .map(|name| {
                as_struct(vec![
                    col(name)
                        .struct_()
                        .field_by_name(MEAN)
                        .percent(key.percent)
                        .precision(key.precision, key.significant),
                    col(name)
                        .struct_()
                        .field_by_name(STANDARD_DEVIATION)
                        .percent(key.percent)
                        .precision(key.precision + 1, key.significant),
                    col(name).struct_().field_by_name(SAMPLE).arr().eval(
                        element()
                            .percent(key.percent)
                            .precision(key.precision, key.significant),
                        false,
                    ),
                ])
                .alias(name)
            })
            .to_vec(),
    );
    // Factors
    lazy_frame = lazy_frame.with_columns([as_struct(
        [ENRICHMENT, SELECTIVITY]
            .map(|name| {
                let expr = col(FACTORS).struct_().field_by_name(name);
                as_struct(vec![
                    expr.clone()
                        .struct_()
                        .field_by_name(MEAN)
                        .precision(key.precision, key.significant),
                    expr.clone()
                        .struct_()
                        .field_by_name(STANDARD_DEVIATION)
                        .precision(key.precision + 1, key.significant),
                    expr.struct_()
                        .field_by_name(SAMPLE)
                        .arr()
                        .eval(element().precision(key.precision, key.significant), false),
                ])
                .alias(name)
            })
            .to_vec(),
    )
    .alias(FACTORS)]);
    // Properties
    lazy_frame = lazy_frame.with_columns([as_struct(vec![
        col(FATTY_ACID)
            .fatty_acid()
            .iodine_value()
            .precision(key.precision, key.significant)
            .alias(IODINE_VALUE),
        col(FATTY_ACID)
            .fatty_acid()
            .relative_atomic_mass(None)
            .precision(key.precision, key.significant)
            .alias(RELATIVE_ATOMIC_MASS),
    ])
    .alias(PROPERTIES)]);
    // Standard
    lazy_frame = lazy_frame.with_columns([col(STANDARD)
        .struct_()
        .field_by_name(FACTOR)
        .name()
        .keep()
        .arr()
        .eval(element().precision(key.precision, key.significant), false)]);
    // Calculations
    let predicate = col(STEREOSPECIFIC_NUMBERS123)
        .struct_()
        .field_by_name(STANDARD_DEVIATION)
        .is_null();
    lazy_frame = lazy_frame.with_columns([
        calculation_sn13(
            predicate.clone(),
            col(STEREOSPECIFIC_NUMBERS123).struct_().field_by_name(MEAN),
            col(STEREOSPECIFIC_NUMBERS2).struct_().field_by_name(MEAN),
        )?
        .alias(formatcp!("{STEREOSPECIFIC_NUMBERS13}.{CALCULATION}")),
        calculation_ef(
            predicate.clone(),
            col(STEREOSPECIFIC_NUMBERS123).struct_().field_by_name(MEAN),
            col(STEREOSPECIFIC_NUMBERS2).struct_().field_by_name(MEAN),
        )?
        .alias(formatcp!("{FACTORS}.{ENRICHMENT}.{CALCULATION}")),
        calculation_sf(
            predicate.clone(),
            col(STEREOSPECIFIC_NUMBERS123).struct_().field_by_name(MEAN),
            col(STEREOSPECIFIC_NUMBERS2).struct_().field_by_name(MEAN),
            col(STEREOSPECIFIC_NUMBERS123)
                .struct_()
                .field_by_name(MEAN)
                .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                .sum(),
            col(STEREOSPECIFIC_NUMBERS2)
                .struct_()
                .field_by_name(MEAN)
                .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                .sum(),
        )?
        .alias(formatcp!("{FACTORS}.{SELECTIVITY}.{CALCULATION}")),
        // col(FACTORS)
        //     .struct_()
        //     .field_by_name(ENRICHMENT)
        //     .struct_()
        //     .with_fields(vec![
        //         format_ef(
        //             predicate.clone(),
        //             col(STEREOSPECIFIC_NUMBERS123).struct_().field_by_name(MEAN),
        //             col(STEREOSPECIFIC_NUMBERS2).struct_().field_by_name(MEAN),
        //         )?
        //         .alias(CALCULATION),
        //     ]),
        // col(FACTORS)
        //     .struct_()
        //     .field_by_name(SELECTIVITY)
        //     .struct_()
        //     .with_fields(vec![
        //         format_sf(
        //             predicate.clone(),
        //             col(STEREOSPECIFIC_NUMBERS123).struct_().field_by_name(MEAN),
        //             col(STEREOSPECIFIC_NUMBERS2).struct_().field_by_name(MEAN),
        //             col(STEREOSPECIFIC_NUMBERS123)
        //                 .struct_()
        //                 .field_by_name(MEAN)
        //                 .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
        //                 .sum(),
        //             col(STEREOSPECIFIC_NUMBERS2)
        //                 .struct_()
        //                 .field_by_name(MEAN)
        //                 .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
        //                 .sum(),
        //         )?
        //         .alias(CALCULATION),
        //     ]),
    ]);
    Ok(lazy_frame)
}

fn sum(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Stereospecific numbers
    Ok(lazy_frame.select(
        STEREOSPECIFIC_NUMBERS
            .try_map(|name| -> PolarsResult<_> {
                let array = eval_arr(col(name).struct_().field_by_name(SAMPLE), |expr| {
                    expr.filter(THRESHOLD).sum()
                })?;
                Ok(as_struct(vec![
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
                .alias(name))
            })?
            .to_vec(),
    ))
}

fn calculation_sn13(predicate: Expr, sn123: Expr, sn2: Expr) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        predicate,
        format_str("(3 * {} - {}) / 2", [sn123, sn2])?,
        lit(NULL),
    ))
}

fn calculation_ef(predicate: Expr, sn123: Expr, sn2: Expr) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        predicate,
        format_str("1 / 3 *  ({} / {})", [sn2, sn123])?,
        lit(NULL),
    ))
}

fn calculation_sf(
    predicate: Expr,
    sn123: Expr,
    sn2: Expr,
    u123: Expr,
    u2: Expr,
) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        predicate,
        format_str("1 / 3 * ({} * {}) / ({} * {})", [sn2, u123, sn123, u2])?,
        lit(NULL),
    ))
}
