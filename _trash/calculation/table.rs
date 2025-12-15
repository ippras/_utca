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

// fn _format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
//     // println!("Display 0: {}", lazy_frame.clone().collect().unwrap());
//     // // Unnest
//     // lazy_frame = lazy_frame
//     //     .unnest(
//     //         cols([
//     //             STEREOSPECIFIC_NUMBERS123,
//     //             STEREOSPECIFIC_NUMBERS2,
//     //             STEREOSPECIFIC_NUMBERS13,
//     //             FACTORS,
//     //         ]),
//     //         Some(PlSmallStr::from_static(".")),
//     //     )
//     //     .unnest(
//     //         cols([
//     //             formatcp!("{FACTORS}.{ENRICHMENT}"),
//     //             formatcp!("{FACTORS}.{SELECTIVITY}"),
//     //         ]),
//     //         Some(PlSmallStr::from_static(".")),
//     //     );
//     println!("Table 0: {}", lazy_frame.clone().collect().unwrap());
//     // let sum = lazy_frame.clone().select([
//     //     // Фильтрует и считает сумму.
//     //     format_mean(
//     //         col(formatcp!(r#"^StereospecificNumbers.*\.{MEAN}$"#))
//     //             .filter(col(FILTER))
//     //             .sum(),
//     //         key,
//     //     ),
//     //     // Не фильтрует и считает стандартное отклонение суммы.
//     //     ternary_expr(
//     //         col(formatcp!(
//     //             r#"^StereospecificNumbers.*\.{STANDARD_DEVIATION}$"#
//     //         ))
//     //         .is_not_null()
//     //         .any(true),
//     //         format_standard_deviation(
//     //             col(formatcp!(
//     //                 r#"^StereospecificNumbers.*\.{STANDARD_DEVIATION}$"#
//     //             ))
//     //             .pow(2)
//     //             .sum()
//     //             .sqrt(),
//     //             key,
//     //         ),
//     //         lit(NULL),
//     //     ),
//     // ]);

//     // Format sum
//     let sum = format_sum(lazy_frame.clone(), key)?;
//     lazy_frame = filter_and_sort(lazy_frame, key);
//     // Format
//     let mut exprs = Vec::new();
//     for stereospecific_number in STEREOSPECIFIC_NUMBERS {
//         let name = stereospecific_number.id();
//         let array = eval_arr(col(name).struct_().field_by_name(SAMPLE), |expr| {
//             expr.filter(FILTER).sum()
//         })?;
//         exprs.push(
//             as_struct(vec![
//                 format_mean(array.clone().arr().mean().alias(MEAN), key),
//                 format_standard_deviation(
//                     array.clone().arr().std(key.ddof).alias(STANDARD_DEVIATION),
//                     key,
//                 ),
//                 format_sample(array.alias(SAMPLE), key),
//             ])
//             .alias(name),
//         );
//     }
//     lazy_frame = lazy_frame.with_columns(exprs);
//     let predicate = col(formatcp!(
//         "{STEREOSPECIFIC_NUMBERS123}.{STANDARD_DEVIATION}"
//     ))
//     .is_null()
//     .or(col(formatcp!("{STEREOSPECIFIC_NUMBERS2}.{STANDARD_DEVIATION}")).is_null());
//     lazy_frame = lazy_frame.with_columns([
//         // Stereospecific numbers
//         format_mean(col(formatcp!(r#"^StereospecificNumbers.*\.{MEAN}$"#)), key),
//         format_standard_deviation(
//             col(formatcp!(
//                 r#"^StereospecificNumbers.*\.{STANDARD_DEVIATION}$"#
//             )),
//             key,
//         ),
//         format_sample(col(formatcp!(r#"^StereospecificNumbers.*\.Array$"#)), key),
//         // Factors
//         format_mean(
//             col(formatcp!(r#"^{FACTORS}.*\.{MEAN}$"#)),
//             Key {
//                 percent: false,
//                 ..key
//             },
//         ),
//         format_standard_deviation(
//             col(formatcp!(r#"^{FACTORS}.*\.{STANDARD_DEVIATION}$"#)),
//             Key {
//                 percent: false,
//                 ..key
//             },
//         ),
//         format_sample(
//             col(formatcp!(r#"^{FACTORS}.*\.Array$"#)),
//             Key {
//                 percent: false,
//                 ..key
//             },
//         ),
//         // Calculation
//         format_sn13(
//             predicate.clone(),
//             format_mean(col(formatcp!("{STEREOSPECIFIC_NUMBERS123}.{MEAN}")), key),
//             format_mean(col(formatcp!("{STEREOSPECIFIC_NUMBERS2}.{MEAN}")), key),
//         )?
//         .alias(formatcp!("{STEREOSPECIFIC_NUMBERS13}.{CALCULATION}")),
//         format_ef(
//             predicate.clone(),
//             format_mean(col(formatcp!("{STEREOSPECIFIC_NUMBERS123}.{MEAN}")), key),
//             format_mean(col(formatcp!("{STEREOSPECIFIC_NUMBERS2}.{MEAN}")), key),
//         )?
//         .alias(formatcp!("{FACTORS}.{ENRICHMENT}.{CALCULATION}")),
//         format_sf(
//             predicate.clone(),
//             format_mean(col(formatcp!("{STEREOSPECIFIC_NUMBERS123}.{MEAN}")), key),
//             format_mean(col(formatcp!("{STEREOSPECIFIC_NUMBERS2}.{MEAN}")), key),
//             format_mean(
//                 col(formatcp!("{STEREOSPECIFIC_NUMBERS123}.{MEAN}"))
//                     .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
//                     .sum(),
//                 key,
//             ),
//             format_mean(
//                 col(formatcp!("{STEREOSPECIFIC_NUMBERS2}.{MEAN}"))
//                     .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
//                     .sum(),
//                 key,
//             ),
//         )?
//         .alias(formatcp!("{FACTORS}.{SELECTIVITY}.Calculation")),
//     ]);
//     // // Unique prefixes
//     // if key.prefix {
//     //     lazy_frame = lazy_frame.with_column(col(LABEL).map(
//     //         |column| {
//     //             let unique_prefixes = unique_prefixes(column.str()?);
//     //             Ok(Series::new(column.name().clone(), unique_prefixes).into_column())
//     //         },
//     //         |_, field| Ok(field.clone()),
//     //     ));
//     // }
//     // Properties
//     lazy_frame = lazy_frame.with_columns([
//         col(FATTY_ACID)
//             .fatty_acid()
//             .iodine_value()
//             .precision(key.precision, key.significant)
//             .alias(formatcp!("{PROPERTIES}.{IODINE_VALUE}")),
//         col(FATTY_ACID)
//             .fatty_acid()
//             .relative_atomic_mass(None)
//             .precision(key.precision, key.significant)
//             .alias(formatcp!("{PROPERTIES}.{RELATIVE_ATOMIC_MASS}")),
//     ]);
//     println!("Table 1: {}", lazy_frame.clone().collect().unwrap());
//     // Concat
//     lazy_frame = concat_lf_diagonal([lazy_frame, sum], Default::default())?;
//     Ok(lazy_frame)
// }

// Filter and sort by minor major
fn filter_and_sort(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    if key.threshold.filter {
        lazy_frame = lazy_frame.filter(col(THRESHOLD));
    } else if key.threshold.sort {
        lazy_frame = lazy_frame.sort_by_exprs(
            [col(THRESHOLD)],
            SortMultipleOptions::default()
                .with_maintain_order(true)
                .with_order_reversed(),
        );
    }
    lazy_frame
}

fn format(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    concat_lf_diagonal(
        [
            format_body(lazy_frame.clone(), key)?,
            format_sum(lazy_frame, key)?,
        ],
        UnionArgs::default(),
    )
}

fn format_body(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Stereospecific numbers
    lazy_frame = lazy_frame.with_columns(
        [
            STEREOSPECIFIC_NUMBERS123,
            STEREOSPECIFIC_NUMBERS2,
            STEREOSPECIFIC_NUMBERS13,
        ]
        .map(|name| {
            as_struct(vec![
                col(name).struct_().field_by_name(MEAN).format_mean(key),
                col(name)
                    .struct_()
                    .field_by_name(STANDARD_DEVIATION)
                    .format_standard_deviation(key),
                col(name).struct_().field_by_name(SAMPLE).format_sample(key),
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
                    expr.clone().struct_().field_by_name(MEAN).format_mean(key),
                    expr.clone()
                        .struct_()
                        .field_by_name(STANDARD_DEVIATION)
                        .format_standard_deviation(key),
                    expr.struct_().field_by_name(SAMPLE).format_sample(key),
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
        format_sn13(
            predicate.clone(),
            col(STEREOSPECIFIC_NUMBERS123).struct_().field_by_name(MEAN),
            col(STEREOSPECIFIC_NUMBERS2).struct_().field_by_name(MEAN),
        )?
        .alias(formatcp!("{STEREOSPECIFIC_NUMBERS13}.{CALCULATION}")),
        format_ef(
            predicate.clone(),
            col(STEREOSPECIFIC_NUMBERS123).struct_().field_by_name(MEAN),
            col(STEREOSPECIFIC_NUMBERS2).struct_().field_by_name(MEAN),
        )?
        .alias(formatcp!("{FACTORS}.{ENRICHMENT}.{CALCULATION}")),
        format_sf(
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

fn format_sum(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Stereospecific numbers
    Ok(lazy_frame.select(
        [
            STEREOSPECIFIC_NUMBERS123,
            STEREOSPECIFIC_NUMBERS2,
            STEREOSPECIFIC_NUMBERS13,
        ]
        .try_map(|name| -> PolarsResult<_> {
            let array = eval_arr(col(name).struct_().field_by_name(SAMPLE), |expr| {
                expr.filter(THRESHOLD).sum()
            })?;
            Ok(as_struct(vec![
                array.clone().arr().mean().alias(MEAN).format_mean(key),
                array
                    .clone()
                    .arr()
                    .std(key.ddof)
                    .alias(STANDARD_DEVIATION)
                    .format_standard_deviation(key),
                array.alias(SAMPLE).format_sample(key),
            ])
            .alias(name))
        })?
        .to_vec(),
    ))
}

/// Extension methods for [`Expr`]
trait Format {
    fn format_mean(self, key: Key) -> Expr;

    fn format_standard_deviation(self, key: Key) -> Expr;

    fn format_sample(self, key: Key) -> Expr;
}

impl Format for Expr {
    fn format_mean(self, key: Key) -> Expr {
        self.percent(key.percent)
            .precision(key.precision, key.significant)
    }

    fn format_standard_deviation(self, key: Key) -> Expr {
        self.percent(key.percent)
            .precision(key.precision + 1, key.significant)
    }

    fn format_sample(self, key: Key) -> Expr {
        self.arr().eval(
            element()
                .percent(key.percent)
                .precision(key.precision, key.significant),
            false,
        )
    }
}

// fn format_mean(expr: Expr, key: Key) -> Expr {
//     expr.percent(key.percent)
//         .precision(key.precision, key.significant)
// }

// fn format_standard_deviation(expr: Expr, key: Key) -> Expr {
//     expr.percent(key.percent)
//         .precision(key.precision + 1, key.significant)
// }

// fn format_sample(expr: Expr, key: Key) -> Expr {
//     expr.arr().eval(
//         element()
//             .percent(key.percent)
//             .precision(key.precision, key.significant),
//         false,
//     )
// }

fn format_sn13(predicate: Expr, sn123: Expr, sn2: Expr) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        predicate,
        format_str("(3 * {} - {}) / 2", [sn123, sn2])?,
        lit(NULL),
    ))
}

fn format_ef(predicate: Expr, sn123: Expr, sn2: Expr) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        predicate,
        format_str("1 / 3 *  ({} / {})", [sn2, sn123])?,
        lit(NULL),
    ))
}

fn format_sf(predicate: Expr, sn123: Expr, sn2: Expr, u123: Expr, u2: Expr) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        predicate,
        format_str("1 / 3 * ({} * {}) / ({} * {})", [sn2, u123, sn123, u2])?,
        lit(NULL),
    ))
}

// fn format(mut key: Key) -> PolarsResult<[Expr; 4]> {
//     let calculation = format_calculation(key)?;
//     if let Kind::EnrichmentFactor | Kind::SelectivityFactor = key.kind {
//         key.percent = false;
//     };
//     Ok([
//         format\.mean(expr(key).struct_().field_by_name(MEAN), key),
//         format_standard_deviation(expr(key).struct_().field_by_name(STANDARD_DEVIATION), key)?,
//         format_array(expr(key).struct_().field_by_name(SAMPLE), key)?,
//         calculation,
//     ])
// }

// fn format_calculation(key: Key) -> PolarsResult<Expr> {
//     let mean = |name| col(name).struct_().field_by_name(MEAN);
//     let standard_deviation = |name| col(name).struct_().field_by_name(STANDARD_DEVIATION);
//     let predicate = standard_deviation(STEREOSPECIFIC_NUMBERS2)
//         .is_null()
//         .or(standard_deviation(STEREOSPECIFIC_NUMBERS123).is_null());
//     Ok(match key.kind {
//         Kind::StereospecificNumbers13 => ternary_expr(
//             predicate,
//             format_str(
//                 "(3 * {} - {}) / 2",
//                 [
//                     format_float(mean(STEREOSPECIFIC_NUMBERS123), key),
//                     format_float(mean(STEREOSPECIFIC_NUMBERS2), key),
//                 ],
//             )?,
//             lit(NULL),
//         ),
//         Kind::EnrichmentFactor => ternary_expr(
//             predicate,
//             format_str(
//                 "{} / (3 * {})",
//                 [
//                     format_float(
//                         mean(STEREOSPECIFIC_NUMBERS2),
//                         Key {
//                             kind: Kind::StereospecificNumbers2,
//                             ..key
//                         },
//                     ),
//                     format_float(
//                         mean(STEREOSPECIFIC_NUMBERS123),
//                         Key {
//                             kind: Kind::StereospecificNumbers123,
//                             ..key
//                         },
//                     ),
//                 ],
//             )?,
//             lit(NULL),
//         ),
//         Kind::SelectivityFactor => ternary_expr(
//             predicate,
//             format_str(
//                 "({} * {}) / ({} * {})",
//                 [
//                     format_float(
//                         mean(STEREOSPECIFIC_NUMBERS2),
//                         Key {
//                             kind: Kind::StereospecificNumbers2,
//                             ..key
//                         },
//                     ),
//                     format_float(
//                         mean(STEREOSPECIFIC_NUMBERS123)
//                             .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
//                             .sum(),
//                         Key {
//                             kind: Kind::StereospecificNumbers123,
//                             ..key
//                         },
//                     ),
//                     format_float(
//                         mean(STEREOSPECIFIC_NUMBERS123),
//                         Key {
//                             kind: Kind::StereospecificNumbers123,
//                             ..key
//                         },
//                     ),
//                     format_float(
//                         mean(STEREOSPECIFIC_NUMBERS2)
//                             .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
//                             .sum(),
//                         Key {
//                             kind: Kind::StereospecificNumbers2,
//                             ..key
//                         },
//                     ),
//                 ],
//             )?,
//             lit(NULL),
//         ),
//         _ => lit(NULL),
//     }
//     .alias("Calculation"))
// }
