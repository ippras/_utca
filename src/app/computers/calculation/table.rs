use crate::{app::states::calculation::settings::Settings, utils::HashedDataFrame};
use egui::util::cache::{ComputerMut, FrameCache};
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
        Field::new(PlSmallStr::from_static("Filter"), DataType::Boolean),
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
        let lazy_frame = format(key)?;
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
    pub(crate) filter: bool,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) sort: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &Settings) -> Self {
        Self {
            frame,
            filter: settings.threshold.filter,
            percent: settings.percent,
            precision: settings.precision,
            significant: settings.significant,
            sort: settings.sort_by_minor_major,
        }
    }
}

/// Table calculation value
type Value = DataFrame;

fn schema(data_frame: &DataFrame) -> PolarsResult<()> {
    let schema = data_frame.schema();
    let _cast = schema.matches_schema(&SCHEMA)?;
    // let length = schema
    //     .array_lengths_recursive()?
    //     .into_iter()
    //     .all_equal_value()
    //     .map_err(|lengths| polars_err!(SchemaMismatch: "Invalid array lengths: expected all equal, got = {lengths:?}"))?;
    Ok(())
}

fn format(key: Key) -> PolarsResult<LazyFrame> {
    let mut lazy_frame = key.frame.data_frame.clone().lazy();
    // println!("Display 0: {}", lazy_frame.clone().collect().unwrap());
    // Unnest
    lazy_frame = lazy_frame
        .unnest(
            cols([
                STEREOSPECIFIC_NUMBERS123,
                STEREOSPECIFIC_NUMBERS2,
                STEREOSPECIFIC_NUMBERS13,
                "Factors",
            ]),
            Some(PlSmallStr::from_static(".")),
        )
        .unnest(
            cols(["Factors.Enrichment", "Factors.Selectivity"]),
            Some(PlSmallStr::from_static(".")),
        );
    // Format sum
    let sum = lazy_frame.clone().select([
        // Фильтрует и считает сумму.
        format_float(
            col(r#"^StereospecificNumbers.*\.Mean$"#)
                .filter(col("Filter"))
                .sum(),
            key,
        ),
        // Не фильтрует и считает стандартное отклонение суммы.
        ternary_expr(
            col(r#"^StereospecificNumbers.*\.StandardDeviation$"#)
                .is_not_null()
                .any(true),
            format_standard_deviation(
                col(r#"^StereospecificNumbers.*\.StandardDeviation$"#)
                    .pow(2)
                    .sum()
                    .sqrt(),
                key,
            )?,
            lit(NULL),
        ),
    ]);
    // Filter minor
    if key.filter {
        // true or null (standard)
        lazy_frame = lazy_frame.filter(col("Filter").or(col("Filter").is_null()));
    } else if key.sort {
        lazy_frame = lazy_frame.sort_by_exprs(
            [col("Filter")],
            SortMultipleOptions::default()
                .with_maintain_order(true)
                .with_order_reversed(),
        );
    }
    // Format
    let predicate = col("StereospecificNumbers123.StandardDeviation")
        .is_null()
        .or(col("StereospecificNumbers2.StandardDeviation").is_null());
    lazy_frame = lazy_frame.with_columns([
        // Stereospecific numbers
        format_float(col(r#"^StereospecificNumbers.*\.Mean$"#), key),
        format_standard_deviation(col(r#"^StereospecificNumbers.*\.StandardDeviation$"#), key)?,
        format_array(col(r#"^StereospecificNumbers.*\.Array$"#), key)?,
        // Factors
        format_float(
            col(r#"^Factors.*\.Mean$"#),
            Key {
                percent: false,
                ..key
            },
        ),
        format_standard_deviation(
            col(r#"^Factors.*\.StandardDeviation$"#),
            Key {
                percent: false,
                ..key
            },
        )?,
        format_array(
            col(r#"^Factors.*\.Array$"#),
            Key {
                percent: false,
                ..key
            },
        )?,
        // Calculation
        format_sn13(
            predicate.clone(),
            format_float(col("StereospecificNumbers123.Mean"), key),
            format_float(col("StereospecificNumbers2.Mean"), key),
        )?
        .alias("StereospecificNumbers13.Calculation"),
        format_ef(
            predicate.clone(),
            format_float(col("StereospecificNumbers123.Mean"), key),
            format_float(col("StereospecificNumbers2.Mean"), key),
        )?
        .alias("Factors.Enrichment.Calculation"),
        format_sf(
            predicate.clone(),
            format_float(col("StereospecificNumbers123.Mean"), key),
            format_float(col("StereospecificNumbers2.Mean"), key),
            format_float(
                col("StereospecificNumbers123.Mean")
                    .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                    .sum(),
                key,
            ),
            format_float(
                col("StereospecificNumbers2.Mean")
                    .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                    .sum(),
                key,
            ),
        )?
        .alias("Factors.Selectivity.Calculation"),
    ]);
    // // Unique prefixes
    // if key.prefix {
    //     lazy_frame = lazy_frame.with_column(col(LABEL).map(
    //         |column| {
    //             let unique_prefixes = unique_prefixes(column.str()?);
    //             Ok(Series::new(column.name().clone(), unique_prefixes).into_column())
    //         },
    //         |_, field| Ok(field.clone()),
    //     ));
    // }
    // Properties
    lazy_frame = lazy_frame.with_columns([
        format_float(
            col(FATTY_ACID).fatty_acid().iodine_value(),
            Key {
                percent: false,
                ..key
            },
        )
        .alias("Properties.IodineValue"),
        format_float(
            col(FATTY_ACID).fatty_acid().relative_atomic_mass(None),
            Key {
                percent: false,
                ..key
            },
        )
        .alias("Properties.RelativeAtomicMass"),
    ]);
    // Concat
    lazy_frame = concat_lf_diagonal([lazy_frame, sum], Default::default())?;
    Ok(lazy_frame)
}

fn format_float(expr: Expr, key: Key) -> Expr {
    float(expr, key).cast(DataType::String)
}

fn format_standard_deviation(expr: Expr, key: Key) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        expr.clone().is_not_null(),
        format_str("±{}", [float(expr, key)])?,
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
                .eval(format_float(element(), key), false)
                .arr()
                .join(lit(", "), false)],
        )?,
        lit(NULL),
    ))
}

fn float(expr: Expr, key: Key) -> Expr {
    expr.percent_if(key.percent)
        .precision(key.precision, key.significant)
}

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

// fn format_sum(key: Key, length: usize) -> PolarsResult<[Expr; 3]> {
//     let array = concat_arr(
//         (0..length)
//             .map(|index| {
//                 expr(key)
//                     .struct_()
//                     .field_by_name("Array")
//                     .arr()
//                     .get(lit(index as IdxSize), false)
//                     .sum()
//             })
//             .collect(),
//     )?;
//     Ok([
//         format\.mean(expr(key).struct_().field_by_name("Mean").sum(), key),
//         format_standard_deviation(array.clone().arr().std(key.ddof), key)?,
//         format_array(array, key)?,
//     ])
// }

// fn format(mut key: Key) -> PolarsResult<[Expr; 4]> {
//     let calculation = format_calculation(key)?;
//     if let Kind::EnrichmentFactor | Kind::SelectivityFactor = key.kind {
//         key.percent = false;
//     };
//     Ok([
//         format\.mean(expr(key).struct_().field_by_name("Mean"), key),
//         format_standard_deviation(expr(key).struct_().field_by_name("StandardDeviation"), key)?,
//         format_array(expr(key).struct_().field_by_name("Array"), key)?,
//         calculation,
//     ])
// }

// fn format_calculation(key: Key) -> PolarsResult<Expr> {
//     let mean = |name| col(name).struct_().field_by_name("Mean");
//     let standard_deviation = |name| col(name).struct_().field_by_name("StandardDeviation");
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
