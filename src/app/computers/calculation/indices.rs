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
    pub(crate) data_frame: Hashed<&'a DataFrame>,
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
    let Some(data_type) = data_frame.schema().get("Experimental") else {
        polars_bail!(SchemaMismatch: r#"The "Experimental" field was not found in the scheme"#);
    };
    let DataType::Struct(fields) = data_type else {
        polars_bail!(SchemaMismatch: r#"Invalid "Experimental" data type: expected `Struct`, got = `{data_type}`"#);
    };
    let Some(triacylglycerol) = fields
        .iter()
        .find(|field| field.name() == "Triacylglycerol")
    else {
        polars_bail!(SchemaMismatch: r#"The "Experimental.Triacylglycerol" field was not found in the scheme"#);
    };
    match triacylglycerol.dtype() {
        DataType::Float64 => {
            return Ok(1);
        }
        DataType::Struct(fields) => {
            let Some(values) = fields.iter().find(|field| field.name() == "Values") else {
                polars_bail!(SchemaMismatch: r#"The "Experimental.Triacylglycerol.Values" field was not found in the scheme"#);
            };
            let data_type = values.dtype();
            let &DataType::Array(box DataType::Float64, length) = data_type else {
                polars_bail!(SchemaMismatch: r#"Invalid "Experimental.Triacylglycerol.Values" data type: expected `Array(Float64)`, got = `{data_type}`"#);
            };
            return Ok(length as _);
        }
        data_type => {
            polars_bail!(SchemaMismatch: r#"Invalid "Experimental.Triacylglycerol" data type: expected [`Float64`, `Struct`], got = `{data_type}`"#);
        }
    }
}

mod one {
    use super::*;

    pub(super) fn compute(key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        let fatty_acid = col("FattyAcid").fatty_acid();
        let value = {
            col("Experimental")
                .struct_()
                .field_by_name("Triacylglycerol")
        };
        #[rustfmt::skip]
        let exprs = vec![
            fatty_acid.clone().monounsaturated(value.clone()),
            fatty_acid.clone().polyunsaturated(value.clone()),
            fatty_acid.clone().saturated(value.clone()),
            fatty_acid.clone().trans(value.clone()),
            fatty_acid.clone().unsaturated(value.clone(), None),
            fatty_acid.clone().unsaturated(value.clone(), NonZeroI8::new(-9)),
            fatty_acid.clone().unsaturated(value.clone(), NonZeroI8::new(-6)),
            fatty_acid.clone().unsaturated(value.clone(), NonZeroI8::new(-3)),
            fatty_acid.clone().unsaturated(value.clone(), NonZeroI8::new(9)),
            fatty_acid.clone().eicosapentaenoic_and_docosahexaenoic(value.clone()),
            fatty_acid.clone().fish_lipid_quality(value.clone()),
            fatty_acid.clone().health_promoting_index(value.clone()),
            fatty_acid.clone().hypocholesterolemic_to_hypercholesterolemic(value.clone()),
            fatty_acid.clone().index_of_atherogenicity(value.clone()),
            fatty_acid.clone().index_of_thrombogenicity(value.clone()),
            fatty_acid.clone().linoleic_to_alpha_linolenic(value.clone()),
            fatty_acid.clone().polyunsaturated_to_saturated(value.clone()),
            fatty_acid.clone().unsaturation_index(value.clone()),
        ];
        lazy_frame = lazy_frame.select(exprs);
        lazy_frame.collect()
    }
}

mod many {
    use super::*;

    macro_rules! index {
        ($f:ident, $fatty_acid:expr, $values:expr $(,$args:expr)*) => {{
            concat_arr(
                $values
                    .clone()
                    .map(|value| $fatty_acid.clone().$f(value $(,$args)*))
                    .collect(),
            )
        }};
    }

    pub(super) fn compute(key: Key, length: u64) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        let fatty_acid = col("FattyAcid").fatty_acid();
        let values = (0..length).map(|index| {
            col("Experimental")
                .struct_()
                .field_by_name("Triacylglycerol")
                .struct_()
                .field_by_name("Values")
                .arr()
                .get(index.into(), false)
        });
        #[rustfmt::skip]
        let exprs = vec![
            index!(monounsaturated, fatty_acid, values)?,
            index!(polyunsaturated, fatty_acid, values)?,
            index!(saturated, fatty_acid, values)?,
            index!(trans, fatty_acid, values)?,
            index!(unsaturated, fatty_acid, values, None)?,
            index!(unsaturated, fatty_acid, values, NonZeroI8::new(-9))?,
            index!(unsaturated, fatty_acid, values, NonZeroI8::new(-6))?,
            index!(unsaturated, fatty_acid, values, NonZeroI8::new(-3))?,
            index!(unsaturated, fatty_acid, values, NonZeroI8::new(9))?,
            index!(eicosapentaenoic_and_docosahexaenoic, fatty_acid, values)?,
            index!(fish_lipid_quality, fatty_acid, values)?,
            index!(health_promoting_index, fatty_acid, values)?,
            index!(hypocholesterolemic_to_hypercholesterolemic, fatty_acid, values)?,
            index!(index_of_atherogenicity, fatty_acid, values)?,
            index!(index_of_thrombogenicity, fatty_acid, values)?,
            index!(linoleic_to_alpha_linolenic, fatty_acid, values)?,
            index!(polyunsaturated_to_saturated, fatty_acid, values)?,
            index!(unsaturation_index, fatty_acid, values)?,
        ];
        lazy_frame = lazy_frame.select(exprs);
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
