use crate::{
    app::states::calculation::settings::{Settings, Threshold},
    r#const::*,
    utils::{
        HashedDataFrame,
        polars::{MeanAndStandardDeviationOptions, mean_and_standard_deviation},
    },
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use std::sync::LazyLock;
use tracing::instrument;

// const SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
//     Arc::new(Schema::from_iter([
//         Field::new(PlSmallStr::from_static(LABEL), DataType::String),
//         field!(FATTY_ACID),
//         Field::new(
//             PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS123),
//             DataType::Struct(vec![
//                 Field::new(PlSmallStr::from_static(MEAN), DataType::Float64),
//                 Field::new(
//                     PlSmallStr::from_static(STANDARD_DEVIATION),
//                     DataType::Float64,
//                 ),
//                 Field::new(
//                     PlSmallStr::from_static(SAMPLE),
//                     DataType::Array(Box::new(DataType::Float64), 0),
//                 ),
//             ]),
//         ),
//         Field::new(
//             PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS2),
//             DataType::Struct(vec![
//                 Field::new(PlSmallStr::from_static(MEAN), DataType::Float64),
//                 Field::new(
//                     PlSmallStr::from_static(STANDARD_DEVIATION),
//                     DataType::Float64,
//                 ),
//                 Field::new(
//                     PlSmallStr::from_static(SAMPLE),
//                     DataType::Array(Box::new(DataType::Float64), 0),
//                 ),
//             ]),
//         ),
//         Field::new(
//             PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS13),
//             DataType::Struct(vec![
//                 Field::new(PlSmallStr::from_static(MEAN), DataType::Float64),
//                 Field::new(
//                     PlSmallStr::from_static(STANDARD_DEVIATION),
//                     DataType::Float64,
//                 ),
//                 Field::new(
//                     PlSmallStr::from_static(SAMPLE),
//                     DataType::Array(Box::new(DataType::Float64), 0),
//                 ),
//             ]),
//         ),
//         // Field::new(
//         //     PlSmallStr::from_static(FACTORS),
//         //     DataType::Struct(vec![
//         //         Field::new(
//         //             PlSmallStr::from_static(ENRICHMENT),
//         //             DataType::Struct(vec![
//         //                 Field::new(PlSmallStr::from_static(MEAN), DataType::Float64),
//         //                 Field::new(
//         //                     PlSmallStr::from_static(STANDARD_DEVIATION),
//         //                     DataType::Float64,
//         //                 ),
//         //                 Field::new(
//         //                     PlSmallStr::from_static(SAMPLE),
//         //                     DataType::Array(Box::new(DataType::Float64), 0),
//         //                 ),
//         //             ]),
//         //         ),
//         //         Field::new(
//         //             PlSmallStr::from_static(SELECTIVITY),
//         //             DataType::Struct(vec![
//         //                 Field::new(PlSmallStr::from_static(MEAN), DataType::Float64),
//         //                 Field::new(
//         //                     PlSmallStr::from_static(STANDARD_DEVIATION),
//         //                     DataType::Float64,
//         //                 ),
//         //                 Field::new(
//         //                     PlSmallStr::from_static(SAMPLE),
//         //                     DataType::Array(Box::new(DataType::Float64), 0),
//         //                 ),
//         //             ]),
//         //         ),
//         //     ]),
//         // ),
//         Field::new(
//             PlSmallStr::from_static(STANDARD),
//             DataType::Struct(vec![
//                 Field::new(
//                     PlSmallStr::from_static(FACTOR),
//                     DataType::Array(Box::new(DataType::Float64), 0),
//                 ),
//                 Field::new(PlSmallStr::from_static(MASK), DataType::Boolean),
//             ]),
//         ),
//         Field::new(PlSmallStr::from_static(THRESHOLD), DataType::Boolean),
//     ]))
// });
const SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::from_iter([
        Field::new(PlSmallStr::from_static(LABEL), DataType::String),
        field!(FATTY_ACID),
        Field::new(
            PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS123),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
        Field::new(
            PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS2),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
        Field::new(
            PlSmallStr::from_static(STEREOSPECIFIC_NUMBERS13),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
        Field::new(
            PlSmallStr::from_static(STANDARD),
            DataType::Struct(vec![
                Field::new(
                    PlSmallStr::from_static(FACTORS),
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
        println!("T: {:?}", key);
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
    pub(crate) normalize_factors: bool,
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
            normalize_factors: settings.normalize_factors,
            percent: settings.percent,
            precision: settings.precision,
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
    // Factors
    let r#struct = |name| {
        col(name)
            .arr()
            .to_struct(Some(PlanCallback::new(move |index| {
                Ok(format!("{name}[{index}]"))
            })))
    };
    let mut enrichment_factor = FattyAcidExpr::enrichment_factor(
        col(STEREOSPECIFIC_NUMBERS2),
        col(STEREOSPECIFIC_NUMBERS123),
    );
    let mut selectivity_factor = concat_arr(vec![
        col(FATTY_ACID).fatty_acid().selectivity_factor(
            r#struct(STEREOSPECIFIC_NUMBERS2)
                .struct_()
                .field_by_name("*"),
            r#struct(STEREOSPECIFIC_NUMBERS123)
                .struct_()
                .field_by_name("*"),
        ),
    ])?;
    if key.normalize_factors {
        enrichment_factor = enrichment_factor / lit(3);
        selectivity_factor = selectivity_factor / lit(3);
    }
    lazy_frame = lazy_frame.with_columns([as_struct(vec![
        mean_and_standard_deviation(enrichment_factor, key).alias(ENRICHMENT),
        mean_and_standard_deviation(selectivity_factor, key).alias(SELECTIVITY),
    ])
    .alias(FACTORS)]);
    // Stereospecific numbers
    lazy_frame = lazy_frame.with_columns(
        STEREOSPECIFIC_NUMBERS
            .map(|name| mean_and_standard_deviation(col(name), key).alias(name))
            .to_vec(),
    );
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
    lazy_frame = lazy_frame.with_column(
        as_struct(vec![
            mean_and_standard_deviation(
                col(STANDARD)
                    .struct_()
                    .field_by_name(STEREOSPECIFIC_NUMBERS123),
                Key {
                    percent: false,
                    ..key
                },
            )
            .alias(STEREOSPECIFIC_NUMBERS123),
            col(STANDARD).struct_().field_by_name(MASK),
        ])
        .alias(STANDARD),
    );
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
                let array = eval_arr(col(name), |expr| expr.filter(THRESHOLD).sum())?;
                Ok(mean_and_standard_deviation(array, key).alias(name))
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

// fn mean_and_standard_deviation(array: Expr, key: Key) -> Expr {
//     as_struct(vec![
//         array
//             .clone()
//             .arr()
//             .mean()
//             .percent(key.percent)
//             .precision(key.precision, key.significant)
//             .alias(MEAN),
//         array
//             .clone()
//             .arr()
//             .std(key.ddof)
//             .percent(key.percent)
//             .precision(key.precision + 1, key.significant)
//             .alias(STANDARD_DEVIATION),
//         array
//             .arr()
//             .eval(
//                 element()
//                     .percent(key.percent)
//                     .precision(key.precision, key.significant),
//                 false,
//             )
//             .alias(SAMPLE),
//     ])
// }
