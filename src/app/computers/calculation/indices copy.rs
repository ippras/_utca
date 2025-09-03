use crate::utils::Hashed;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use std::num::NonZeroI8;
use tracing::instrument;

/// Calculation indices computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation indices computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        match length(&key.data_frame)? {
            1 => one::compute(key),
            length => many::compute(key, length),
        }
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
    pub(crate) data_frame: &'a Hashed<DataFrame>,
    pub(crate) ddof: u8,
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
    let Some(triacylglycerol) = fields.iter().find(|field| field.name() == "Experimental") else {
        polars_bail!(SchemaMismatch: r#"The "{STEREOSPECIFIC_NUMBERS123}.Experimental" field was not found in the scheme"#);
    };
    match triacylglycerol.dtype() {
        DataType::Float64 => {
            return Ok(1);
        }
        DataType::Struct(fields) => {
            let Some(repetitions) = fields.iter().find(|field| field.name() == "Repetitions")
            else {
                polars_bail!(SchemaMismatch: r#"The "Experimental.STEREOSPECIFIC_NUMBERS123.Repetitions" field was not found in the scheme"#);
            };
            let data_type = repetitions.dtype();
            let &DataType::Array(box DataType::Float64, length) = data_type else {
                polars_bail!(SchemaMismatch: r#"Invalid "Experimental.STEREOSPECIFIC_NUMBERS123.Repetitions" data type: expected `Array(Float64)`, got = `{data_type}`"#);
            };
            return Ok(length as _);
        }
        data_type => {
            polars_bail!(SchemaMismatch: r#"Invalid "Experimental.STEREOSPECIFIC_NUMBERS123" data type: expected [`Float64`, `Struct`], got = `{data_type}`"#);
        }
    }
}

mod one {
    use super::*;

    pub(super) fn compute(key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        let fatty_acid = || col(FATTY_ACID).fatty_acid();
        let column = |value: Expr| {
            as_struct(vec![
                fatty_acid().monounsaturated(value.clone()),
                fatty_acid().polyunsaturated(value.clone()),
                fatty_acid().saturated(value.clone()),
                fatty_acid().trans(value.clone()),
                fatty_acid().unsaturated(value.clone(), None),
                fatty_acid().unsaturated(value.clone(), NonZeroI8::new(-9)),
                fatty_acid().unsaturated(value.clone(), NonZeroI8::new(-6)),
                fatty_acid().unsaturated(value.clone(), NonZeroI8::new(-3)),
                fatty_acid().unsaturated(value.clone(), NonZeroI8::new(9)),
                fatty_acid().eicosapentaenoic_and_docosahexaenoic(value.clone()),
                fatty_acid().fish_lipid_quality(value.clone()),
                fatty_acid().health_promoting_index(value.clone()),
                fatty_acid().hypocholesterolemic_to_hypercholesterolemic(value.clone()),
                fatty_acid().index_of_atherogenicity(value.clone()),
                fatty_acid().index_of_thrombogenicity(value.clone()),
                fatty_acid().linoleic_to_alpha_linolenic(value.clone()),
                fatty_acid().polyunsaturated_6_to_polyunsaturated_3(value.clone()),
                fatty_acid().polyunsaturated_to_saturated(value.clone()),
                fatty_acid().unsaturation_index(value.clone()),
            ])
        };
        let exprs = vec![
            column(
                col(STEREOSPECIFIC_NUMBERS123)
                    .struct_()
                    .field_by_name("Experimental"),
            )
            .alias(STEREOSPECIFIC_NUMBERS123),
            column(col(STEREOSPECIFIC_NUMBERS13).struct_().field_by_index(0))
                .alias(STEREOSPECIFIC_NUMBERS13),
            column(
                col(STEREOSPECIFIC_NUMBERS2)
                    .struct_()
                    .field_by_name("Experimental"),
            )
            .alias(STEREOSPECIFIC_NUMBERS2),
        ];
        lazy_frame = lazy_frame.select(exprs);
        lazy_frame.collect()
    }
}

mod many {
    use super::*;

    // macro_rules! index {
    //     ($f:ident, $fatty_acid:expr, $values:expr $(,$args:expr)*) => {{
    //         concat_arr(
    //             $values
    //                 .clone()
    //                 .map(|value| $fatty_acid.clone().$f(value $(,$args)*))
    //                 .collect(),
    //         )
    //     }};
    // }

    fn temp(values: impl Iterator<Item = Expr>, f: impl Fn(Expr) -> Expr) -> PolarsResult<Expr> {
        concat_arr(values.map(f).collect())
    }

    pub(super) fn compute(key: Key, length: u64) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        println!("lazy_frame0: {}", lazy_frame.clone().collect().unwrap());
        let fatty_acid = || col(FATTY_ACID).fatty_acid();
        let values = |stereospecific_number| {
            (0..length).map(move |index| {
                col("Calculated")
                    .struct_()
                    .field_by_name(stereospecific_number)
                    .struct_()
                    .field_by_name("Values")
                    .arr()
                    .get(index.into(), false)
            })
        };
        let column = |stereospecific_number| -> PolarsResult<Expr> {
            Ok(as_struct(vec![
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().monounsaturated(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().polyunsaturated(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().saturated(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().trans(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().unsaturated(value, None))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-9)))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-6)))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-3)))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(9)))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().eicosapentaenoic_and_docosahexaenoic(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().fish_lipid_quality(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().health_promoting_index(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| {
                            fatty_acid().hypocholesterolemic_to_hypercholesterolemic(value)
                        })
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().index_of_atherogenicity(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().index_of_thrombogenicity(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().linoleic_to_alpha_linolenic(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().polyunsaturated_6_to_polyunsaturated_3(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().polyunsaturated_to_saturated(value))
                        .collect(),
                )?,
                concat_arr(
                    values(stereospecific_number)
                        .map(|value| fatty_acid().unsaturation_index(value))
                        .collect(),
                )?,
            ]))
        };
        // #[rustfmt::skip]
        // let exprs = vec![
        //     index!(monounsaturated, fatty_acid, values)?,
        //     index!(polyunsaturated, fatty_acid, values)?,
        //     index!(saturated, fatty_acid, values)?,
        //     index!(trans, fatty_acid, values)?,
        //     index!(unsaturated, fatty_acid, values, None)?,
        //     index!(unsaturated, fatty_acid, values, NonZeroI8::new(-9))?,
        //     index!(unsaturated, fatty_acid, values, NonZeroI8::new(-6))?,
        //     index!(unsaturated, fatty_acid, values, NonZeroI8::new(-3))?,
        //     index!(unsaturated, fatty_acid, values, NonZeroI8::new(9))?,
        //     index!(eicosapentaenoic_and_docosahexaenoic, fatty_acid, values)?,
        //     index!(fish_lipid_quality, fatty_acid, values)?,
        //     index!(health_promoting_index, fatty_acid, values)?,
        //     index!(hypocholesterolemic_to_hypercholesterolemic, fatty_acid, values)?,
        //     index!(index_of_atherogenicity, fatty_acid, values)?,
        //     index!(index_of_thrombogenicity, fatty_acid, values)?,
        //     index!(linoleic_to_alpha_linolenic, fatty_acid, values)?,
        //     index!(polyunsaturated_6_to_polyunsaturated_3, fatty_acid, values)?,
        //     index!(polyunsaturated_to_saturated, fatty_acid, values)?,
        //     index!(unsaturation_index, fatty_acid, values)?,
        // ];
        println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());
        // Mean and standard deviation
        let exprs = lazy_frame
            .collect_schema()?
            .iter_names()
            .map(|name| {
                as_struct(vec![
                    col(name.as_str()).arr().mean().alias("Mean"),
                    col(name.as_str())
                        .arr()
                        .std(key.ddof)
                        .alias("StandardDeviation"),
                    col(name.as_str()).alias("Repetitions"),
                ])
                .alias(name.clone())
            })
            .collect::<Vec<_>>();
        lazy_frame = lazy_frame.select(exprs);
        lazy_frame.collect()
    }
}
