use crate::utils::Hashed;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use std::num::NonZeroI8;
use tracing::instrument;

/// Composition indices computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition indices computer
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

/// Composition indices key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: Hashed<&'a DataFrame>,
    pub(crate) ddof: u8,
}

/// Composition indices value
type Value = DataFrame;

fn length(data_frame: &DataFrame) -> PolarsResult<u64> {
    // Triacylglycerol
    let Some(data_type) = data_frame.schema().get(TRIACYLGLYCEROL) else {
        polars_bail!(SchemaMismatch: "The `TRIACYLGLYCEROL` field was not found in the scheme");
    };
    polars_ensure!(*data_type == data_type!(TRIACYLGLYCEROL), SchemaMismatch: "Invalid `TRIACYLGLYCEROL` data type: expected `TRIACYLGLYCEROL`, got = `{data_type}`");
    // Value
    let Some(data_type) = data_frame.schema().get("Value") else {
        polars_bail!(SchemaMismatch: r#"The "Value" field was not found in the scheme"#);
    };
    match data_type {
        DataType::Float64 => {
            return Ok(1);
        }
        DataType::Struct(fields) => {
            let Some(repetitions) = fields.iter().find(|field| field.name() == "Repetitions")
            else {
                polars_bail!(SchemaMismatch: r#"The "Value.Repetitions" field was not found in the scheme"#);
            };
            let data_type = repetitions.dtype();
            let &DataType::Array(box DataType::Float64, length) = data_type else {
                polars_bail!(SchemaMismatch: r#"Invalid "Value.Repetitions" data type: expected `Array(Float64)`, got = `{data_type}`"#);
            };
            return Ok(length as _);
        }
        data_type => {
            polars_bail!(SchemaMismatch: r#"Invalid "Value" data type: expected [`Float64`, `Struct`], got = `{data_type}`"#);
        }
    }
}

mod one {
    use super::*;

    macro_rules! index {
        ($f:ident, $triacylglycerol:ident, $value:ident $(,$args:expr)*) => {
            |name| $triacylglycerol.clone().struct_().field_by_name(name).fatty_acid().$f($value.clone() $(,$args)*)
        };
    }

    pub(super) fn compute(key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        let triacylglycerol = col("Triacylglycerol");
        let value = col("Value");
        #[rustfmt::skip]
        let exprs = [
            stereospecific_numbers(index!(monounsaturated, triacylglycerol, value))?.alias("Monounsaturated"),
            stereospecific_numbers(index!(polyunsaturated, triacylglycerol, value))?.alias("Polyunsaturated"),
            stereospecific_numbers(index!(saturated, triacylglycerol, value))?.alias("Saturated"),
            stereospecific_numbers(index!(trans, triacylglycerol, value))?.alias("Trans"),
            stereospecific_numbers(index!(unsaturated, triacylglycerol, value, None))?.alias("Unsaturated"),
            stereospecific_numbers(index!(unsaturated, triacylglycerol, value, NonZeroI8::new(-9)))?.alias("Unsaturated-9"),
            stereospecific_numbers(index!(unsaturated, triacylglycerol, value, NonZeroI8::new(-6)))?.alias("Unsaturated-6"),
            stereospecific_numbers(index!(unsaturated, triacylglycerol, value, NonZeroI8::new(-3)))?.alias("Unsaturated-3"),
            stereospecific_numbers(index!(unsaturated, triacylglycerol, value, NonZeroI8::new(9)))?.alias("Unsaturated9"),
            stereospecific_numbers(index!(eicosapentaenoic_and_docosahexaenoic, triacylglycerol, value))?.alias("EicosapentaenoicAndDocosahexaenoic"),
            stereospecific_numbers(index!(fish_lipid_quality, triacylglycerol, value))?.alias("FishLipidQuality"),
            stereospecific_numbers(index!(health_promoting_index, triacylglycerol, value))?.alias("HealthPromotingIndex"),
            stereospecific_numbers(index!(hypocholesterolemic_to_hypercholesterolemic, triacylglycerol, value))?.alias("HypocholesterolemicToHypercholesterolemic"),
            stereospecific_numbers(index!(index_of_atherogenicity, triacylglycerol, value))?.alias("IndexOfAtherogenicity"),
            stereospecific_numbers(index!(index_of_thrombogenicity, triacylglycerol, value))?.alias("IndexOfThrombogenicity"),
            stereospecific_numbers(index!(linoleic_to_alpha_linolenic, triacylglycerol, value))?.alias("LinoleicToAlphaLinolenic"),
            stereospecific_numbers(index!(polyunsaturated_to_saturated, triacylglycerol, value))?.alias("PolyunsaturatedToSaturated"),
            stereospecific_numbers(index!(unsaturation_index, triacylglycerol, value))?.alias("UnsaturationIndex"),
        ];
        lazy_frame = lazy_frame.clone().select(exprs);
        lazy_frame.collect()
    }

    fn stereospecific_numbers(index: impl Fn(&str) -> Expr) -> PolarsResult<Expr> {
        concat_arr(vec![
            index("StereospecificNumber1"),
            index("StereospecificNumber2"),
            index("StereospecificNumber3"),
        ])
    }
}

mod many {
    use super::*;

    macro_rules! repetitions {
        ($f:ident, $triacylglycerol:ident, $value:ident, $length:ident $(,$args:expr)*) => {
            |name| concat_arr((0..$length).map(|index| {
                $triacylglycerol.clone().struct_().field_by_name(name).fatty_acid().$f($value(index) $(,$args)*)
            }).collect::<Vec<_>>())
        };
    }

    pub(super) fn compute(key: Key, length: u64) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        let triacylglycerol = col("Triacylglycerol");
        let value = |index| {
            col("Value")
                .struct_()
                .field_by_name("Repetitions")
                .arr()
                .get(lit(index), false)
        };
        #[rustfmt::skip]
        let exprs = [
            stereospecific_numbers(repetitions!(monounsaturated, triacylglycerol, value, length), key.ddof)?.alias("Monounsaturated"),
            stereospecific_numbers(repetitions!(polyunsaturated, triacylglycerol, value, length), key.ddof)?.alias("Polyunsaturated"),
            stereospecific_numbers(repetitions!(saturated, triacylglycerol, value, length), key.ddof)?.alias("Saturated"),
            stereospecific_numbers(repetitions!(trans, triacylglycerol, value, length), key.ddof)?.alias("Trans"),
            stereospecific_numbers(repetitions!(unsaturated, triacylglycerol, value, length, None), key.ddof)?.alias("Unsaturated"),
            stereospecific_numbers(repetitions!(unsaturated, triacylglycerol, value, length, NonZeroI8::new(-9)), key.ddof)?.alias("Unsaturated-9"),
            stereospecific_numbers(repetitions!(unsaturated, triacylglycerol, value, length, NonZeroI8::new(-6)), key.ddof)?.alias("Unsaturated-6"),
            stereospecific_numbers(repetitions!(unsaturated, triacylglycerol, value, length, NonZeroI8::new(-3)), key.ddof)?.alias("Unsaturated-3"),
            stereospecific_numbers(repetitions!(unsaturated, triacylglycerol, value, length, NonZeroI8::new(9)), key.ddof)?.alias("Unsaturated9"),
            stereospecific_numbers(repetitions!(eicosapentaenoic_and_docosahexaenoic, triacylglycerol, value, length), key.ddof)?.alias("EicosapentaenoicAndDocosahexaenoic"),
            stereospecific_numbers(repetitions!(fish_lipid_quality, triacylglycerol, value, length), key.ddof)?.alias("FishLipidQuality"),
            stereospecific_numbers(repetitions!(health_promoting_index, triacylglycerol, value, length), key.ddof)?.alias("HealthPromotingIndex"),
            stereospecific_numbers(repetitions!(hypocholesterolemic_to_hypercholesterolemic, triacylglycerol, value, length), key.ddof)?.alias("HypocholesterolemicToHypercholesterolemic"),
            stereospecific_numbers(repetitions!(index_of_atherogenicity, triacylglycerol, value, length), key.ddof)?.alias("IndexOfAtherogenicity"),
            stereospecific_numbers(repetitions!(index_of_thrombogenicity, triacylglycerol, value, length), key.ddof)?.alias("IndexOfThrombogenicity"),
            stereospecific_numbers(repetitions!(linoleic_to_alpha_linolenic, triacylglycerol, value, length), key.ddof)?.alias("LinoleicToAlphaLinolenic"),
            stereospecific_numbers(repetitions!(polyunsaturated_to_saturated, triacylglycerol, value, length), key.ddof)?.alias("PolyunsaturatedToSaturated"),
            stereospecific_numbers(repetitions!(unsaturation_index, triacylglycerol, value, length), key.ddof)?.alias("UnsaturationIndex"),
        ];
        lazy_frame = lazy_frame.clone().select(exprs);
        lazy_frame.collect()
    }

    fn stereospecific_numbers(
        repetitions: impl Fn(&str) -> PolarsResult<Expr>,
        ddof: u8,
    ) -> PolarsResult<Expr> {
        let index = |name| -> PolarsResult<_> {
            Ok(as_struct(vec![
                repetitions(name)?.arr().mean().alias("Mean"),
                repetitions(name)?
                    .arr()
                    .std(ddof)
                    .alias("StandardDeviation"),
                repetitions(name)?.alias("Repetitions"),
            ]))
        };
        concat_arr(vec![
            index("StereospecificNumber1")?,
            index("StereospecificNumber2")?,
            index("StereospecificNumber3")?,
        ])
    }
}
