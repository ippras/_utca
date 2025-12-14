use crate::{
    app::states::calculation::{Indices, Settings},
    utils::{
        HashedDataFrame,
        polars::{format_sample, format_standard_deviation},
    },
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use std::num::NonZeroI8;
use tracing::instrument;

const STEREOSPECIFIC_NUMBERS: [&str; 3] = [
    STEREOSPECIFIC_NUMBERS123,
    STEREOSPECIFIC_NUMBERS13,
    STEREOSPECIFIC_NUMBERS2,
];

/// Calculation indices computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation indices computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        compute(key, length(&key.frame)?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation indices key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) indices: &'a Indices,
    pub(crate) precision: usize,
    pub(crate) save: bool,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            indices: &settings.indices,
            precision: settings.precision,
            save: settings.threshold.save,
            significant: settings.significant,
        }
    }
}

/// Calculation indices value
type Value = DataFrame;

fn length(data_frame: &DataFrame) -> PolarsResult<u64> {
    // FattyAcid
    let Some(data_type) = data_frame.schema().get(FATTY_ACID) else {
        polars_bail!(SchemaMismatch: "The `FATTY_ACID` field was not found in the scheme");
    };
    polars_ensure!(*data_type == data_type!(FATTY_ACID), SchemaMismatch: "Invalid `FATTY_ACID` data type: expected `FATTY_ACID`, got = `{data_type}`");
    // Value
    let Some(data_type) = data_frame.schema().get(STEREOSPECIFIC_NUMBERS123) else {
        polars_bail!(SchemaMismatch: r#"The "{STEREOSPECIFIC_NUMBERS123}" field was not found in the scheme"#);
    };
    let DataType::Struct(fields) = data_type else {
        polars_bail!(SchemaMismatch: r#"Invalid "{STEREOSPECIFIC_NUMBERS123}" data type: expected `Struct`, got = `{data_type}`"#);
    };
    let Some(array) = fields.iter().find(|field| field.name() == "Array") else {
        polars_bail!(SchemaMismatch: r#"The "STEREOSPECIFIC_NUMBERS123.Array" field was not found in the scheme"#);
    };
    let data_type = array.dtype();
    let &DataType::Array(box DataType::Float64, length) = data_type else {
        polars_bail!(SchemaMismatch: r#"Invalid "STEREOSPECIFIC_NUMBERS123.Array" data type: expected `Array(Float64)`, got = `{data_type}`"#);
    };
    return Ok(length as _);
}

fn compute(key: Key, length: u64) -> PolarsResult<Value> {
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
                .get(lit(index), false)
        })
    };
    let sample = |name: &str, f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        concat_arr(iter(col(name)).map(f).collect())
    };
    let stereospecific_numbers = |name: &str, f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            sample(name, f)?.arr().mean().alias("Mean"),
            sample(name, f)?
                .arr()
                .std(key.ddof)
                .alias("StandardDeviation"),
            sample(name, f)?.alias("Sample"),
        ])
        .alias(name))
        // Ok(as_struct(vec![
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .monounsaturated(value)
        //                     .alias("Monounsaturated")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .polyunsaturated(value)
        //                     .alias("Polyunsaturated")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .saturated(value)
        //                     .alias("Saturated")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| col(FATTY_ACID).fatty_acid().trans(value).alias("Trans"))
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .unsaturated(value, None)
        //                     .alias("Unsaturated")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .unsaturated(value, NonZeroI8::new(-9))
        //                     .alias("Unsaturated_9")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .unsaturated(value, NonZeroI8::new(-6))
        //                     .alias("Unsaturated_6")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .unsaturated(value, NonZeroI8::new(-3))
        //                     .alias("Unsaturated_3")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .unsaturated(value, NonZeroI8::new(9))
        //                     .alias("Unsaturated9")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .eicosapentaenoic_and_docosahexaenoic(value)
        //                     .alias("EicosapentaenoicAndDocosahexaenoic")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .fish_lipid_quality(value)
        //                     .alias("FishLipidQuality")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .health_promoting_index(value)
        //                     .alias("HealthPromotingIndex")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .hypocholesterolemic_to_hypercholesterolemic(value)
        //                     .alias("HypocholesterolemicToHypercholesterolemic")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .index_of_atherogenicity(value)
        //                     .alias("IndexOfAtherogenicity")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .index_of_thrombogenicity(value)
        //                     .alias("IndexOfThrombogenicity")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .linoleic_to_alpha_linolenic(value)
        //                     .alias("LinoleicToAlphaLinolenic")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .polyunsaturated_6_to_polyunsaturated_3(value)
        //                     .alias("Polyunsaturated_6ToPolyunsaturated_3")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .polyunsaturated_to_saturated(value)
        //                     .alias("PolyunsaturatedToSaturated")
        //             })
        //             .collect(),
        //     )?,
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 col(FATTY_ACID)
        //                     .fatty_acid()
        //                     .unsaturation_index(value)
        //                     .alias("UnsaturationIndex")
        //             })
        //             .collect(),
        //     )?,
        //     // P
        //     concat_arr(
        //         iter(expr.clone())
        //             .map(|value| {
        //                 (value
        //                     * col(FATTY_ACID)
        //                         .fatty_acid()
        //                         .iodine_value()
        //                         .alias("IodineValue"))
        //                 .sum()
        //                 .alias("IodineValue")
        //             })
        //             .collect(),
        //     )?,
        //     // concat_arr(
        //     //     values(expr.clone())
        //     //         .map(|value| {
        //     //             (lit(0.6683) * fatty_acid().unsaturation_index(value) + lit(0.250364))
        //     //                 .alias("IodineValue.Wang2012")
        //     //         })
        //     //         .collect(),
        //     // )?,
        //     // concat_arr(
        //     //     values(expr.clone())
        //     //         .map(|value| {
        //     //             (lit(-0.1209) * fatty_acid().unsaturation_index(value) + lit(0.650958))
        //     //                 .alias("CetaneNumber")
        //     //         })
        //     //         .collect(),
        //     // )?,
        //     // concat_arr(
        //     //     values(expr.clone())
        //     //         .map(|value| {
        //     //             (lit(-0.0384) * fatty_acid().degree_of_unsaturation(value) + lit(0.777))
        //     //                 .alias("OxidativeStability")
        //     //         })
        //     //         .collect(),
        //     // )?,
        //     // concat_arr(
        //     //     values(expr.clone())
        //     //         .map(|value| {
        //     //             (lit(1.7556) * fatty_acid().degree_of_unsaturation(value) + lit(-0.14772))
        //     //                 .alias("ColdFilterPluggingPoint")
        //     //         })
        //     //         .collect(),
        //     // )?,
        //     // concat_arr(
        //     //     values(expr.clone())
        //     //         .map(|value| {
        //     //             fatty_acid()
        //     //                 .long_chain_saturated_factor(value)
        //     //                 .alias("LongChainSaturatedFactor")
        //     //         })
        //     //         .collect(),
        //     // )?,
        // ]))
    };
    let property = |f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS123, f)?,
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS13, f)?,
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS2, f)?,
        ]))
    };
    lazy_frame = lazy_frame.select([
        property(monounsaturated)?.alias("Monounsaturated"),
        property(polyunsaturated)?.alias("Polyunsaturated"),
        property(saturated)?.alias("Saturated"),
        property(trans)?.alias("Trans"),
        property(|expr| unsaturated(expr, None))?.alias("Unsaturated"),
        property(|expr| unsaturated(expr, NonZeroI8::new(-9)))?.alias("Unsaturated-9"),
        property(|expr| unsaturated(expr, NonZeroI8::new(-6)))?.alias("Unsaturated-6"),
        property(|expr| unsaturated(expr, NonZeroI8::new(-3)))?.alias("Unsaturated-3"),
        property(|expr| unsaturated(expr, NonZeroI8::new(9)))?.alias("Unsaturated9"),
        property(eicosapentaenoic_and_docosahexaenoic)?.alias("EicosapentaenoicAndDocosahexaenoic"),
        property(fish_lipid_quality)?.alias("FishLipidQuality"),
        property(health_promoting_index)?.alias("HealthPromotingIndex"),
        property(hypocholesterolemic_to_hypercholesterolemic)?
            .alias("HypocholesterolemicToHypercholesterolemic"),
        property(index_of_atherogenicity)?.alias("IndexOfAtherogenicity"),
        property(index_of_thrombogenicity)?.alias("IndexOfThrombogenicity"),
        property(linoleic_to_alpha_linolenic)?.alias("LinoleicToAlphaLinolenic"),
        property(polyunsaturated_6_to_polyunsaturated_3)?
            .alias("Polyunsaturated-6ToPolyunsaturated-3"),
        property(polyunsaturated_to_saturated)?.alias("PolyunsaturatedToSaturated"),
        property(unsaturation_index)?.alias("UnsaturationIndex"),
        property(iodine_value)?.alias("IodineValue"),
    ]);
    // Format
    lazy_frame = lazy_frame
        .unnest(all(), Some(PlSmallStr::from_static(".")))
        .unnest(all(), Some(PlSmallStr::from_static(".")))
        .with_columns([
            col(r#"^.*Mean$"#)
                .precision(key.precision, key.significant)
                .cast(DataType::String),
            format_standard_deviation(
                col(r#"^.*StandardDeviation$"#).precision(key.precision, key.significant),
            )?,
            format_sample(
                col(r#"^.*Sample$"#).arr().eval(
                    element()
                        .precision(key.precision, key.significant)
                        .cast(DataType::String),
                    false,
                ),
            )?,
        ]);
    lazy_frame = lazy_frame.select(
        key.indices
            .iter_visible()
            .map(|name| {
                as_struct(
                    STEREOSPECIFIC_NUMBERS
                        .map(|stereospecific_numbers| {
                            as_struct(vec![
                                col(format!("{name}.{stereospecific_numbers}.Mean")).alias("Mean"),
                                col(format!("{name}.{stereospecific_numbers}.StandardDeviation"))
                                    .alias("StandardDeviation"),
                                col(format!("{name}.{stereospecific_numbers}.Sample"))
                                    .alias("Sample"),
                            ])
                            .alias(stereospecific_numbers)
                        })
                        .to_vec(),
                )
                .alias(name)
            })
            .collect::<Vec<_>>(),
    );
    println!("lazy_frame I 3: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame.collect()
}

fn monounsaturated(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().monounsaturated(expr)
}

fn polyunsaturated(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().polyunsaturated(expr)
}

fn saturated(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().saturated(expr)
}

fn trans(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().trans(expr)
}

fn unsaturated(expr: Expr, offset: Option<NonZeroI8>) -> Expr {
    col(FATTY_ACID).fatty_acid().unsaturated(expr, offset)
}

fn eicosapentaenoic_and_docosahexaenoic(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .eicosapentaenoic_and_docosahexaenoic(expr)
}

fn fish_lipid_quality(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().fish_lipid_quality(expr)
}

fn health_promoting_index(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().health_promoting_index(expr)
}

fn hypocholesterolemic_to_hypercholesterolemic(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .hypocholesterolemic_to_hypercholesterolemic(expr)
}

fn index_of_atherogenicity(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().index_of_atherogenicity(expr)
}

fn index_of_thrombogenicity(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().index_of_thrombogenicity(expr)
}

fn linoleic_to_alpha_linolenic(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .linoleic_to_alpha_linolenic(expr)
}

fn polyunsaturated_6_to_polyunsaturated_3(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .polyunsaturated_6_to_polyunsaturated_3(expr)
}

fn polyunsaturated_to_saturated(expr: Expr) -> Expr {
    col(FATTY_ACID)
        .fatty_acid()
        .polyunsaturated_to_saturated(expr)
}

fn unsaturation_index(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().unsaturation_index(expr)
}

fn iodine_value(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().iodine_value().sum()
}
