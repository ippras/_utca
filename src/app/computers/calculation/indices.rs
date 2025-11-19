use crate::{
    app::states::calculation::{Indices, Settings},
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::ExprExt;
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
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            indices: &settings.indices,
            precision: settings.precision,
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
    let fatty_acid = || col(FATTY_ACID).fatty_acid();
    let values = |expr: Expr| {
        (0..length).map(move |index| {
            expr.clone()
                .struct_()
                .field_by_name("Array")
                .arr()
                .get(lit(index), false)
        })
    };
    let stereospecific_numbers = |expr: Expr| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().monounsaturated(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().polyunsaturated(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().saturated(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().trans(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().unsaturated(value, None))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-9)))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-6)))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-3)))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(9)))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().eicosapentaenoic_and_docosahexaenoic(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().fish_lipid_quality(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().health_promoting_index(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().hypocholesterolemic_to_hypercholesterolemic(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().index_of_atherogenicity(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().index_of_thrombogenicity(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().linoleic_to_alpha_linolenic(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().polyunsaturated_6_to_polyunsaturated_3(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().polyunsaturated_to_saturated(value))
                    .collect(),
            )?,
            concat_arr(
                values(expr.clone())
                    .map(|value| fatty_acid().unsaturation_index(value))
                    .collect(),
            )?,
        ]))
    };
    lazy_frame = lazy_frame.select([
        stereospecific_numbers(col(STEREOSPECIFIC_NUMBERS123))?.alias(STEREOSPECIFIC_NUMBERS123),
        stereospecific_numbers(col(STEREOSPECIFIC_NUMBERS13))?.alias(STEREOSPECIFIC_NUMBERS13),
        stereospecific_numbers(col(STEREOSPECIFIC_NUMBERS2))?.alias(STEREOSPECIFIC_NUMBERS2),
    ]);
    // Mean and standard deviation
    let exprs = STEREOSPECIFIC_NUMBERS
        .into_iter()
        .map(|stereospecific_numbers| {
            as_struct(
                key.indices
                    .iter_visible()
                    .map(|name| {
                        as_struct(vec![
                            col(stereospecific_numbers)
                                .struct_()
                                .field_by_name(name)
                                .clone()
                                .arr()
                                .mean()
                                .alias("Mean"),
                            col(stereospecific_numbers)
                                .struct_()
                                .field_by_name(name)
                                .clone()
                                .arr()
                                .std(key.ddof)
                                .alias("StandardDeviation"),
                            col(stereospecific_numbers)
                                .struct_()
                                .field_by_name(name)
                                .alias("Array"),
                        ])
                        .alias(name)
                    })
                    .collect(),
            )
            .alias(stereospecific_numbers)
        })
        .collect::<Vec<_>>();
    lazy_frame = lazy_frame.select(exprs);
    // Format
    lazy_frame = lazy_frame
        .unnest(all(), Some(PlSmallStr::from_static("_")))
        .unnest(all(), Some(PlSmallStr::from_static("_")))
        .with_columns([
            col(r#"^.*_Mean$"#).precision(key.precision, key.significant),
            col(r#"^.*_StandardDeviation$"#).precision(key.precision, key.significant),
            col(r#"^.*_Array$"#)
                .arr()
                .eval(element().precision(key.precision, key.significant), false),
        ]);
    let exprs = STEREOSPECIFIC_NUMBERS.map(|stereospecific_number| {
        as_struct(
            key.indices
                .iter_visible()
                .map(|name| {
                    as_struct(vec![
                        col(format!("{stereospecific_number}_{name}_Mean")).alias("Mean"),
                        col(format!("{stereospecific_number}_{name}_StandardDeviation"))
                            .alias("StandardDeviation"),
                        col(format!("{stereospecific_number}_{name}_Array")).alias("Array"),
                    ])
                    .alias(name)
                })
                .collect(),
        )
        .alias(stereospecific_number)
    });
    lazy_frame = lazy_frame.select(exprs);
    lazy_frame.collect()
}
