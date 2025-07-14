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
        match &key.data_frame["Value"].dtype() {
            DataType::Float64 => f64::compute(key),
            DataType::Struct(_) => r#struct::compute(key),
            _ => {
                polars_bail!(SchemaMismatch: "cannot compute composition indices, data types don't match");
            }
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

mod f64 {
    use super::*;

    macro_rules! index {
        ($f:ident, $triacylglycerol:ident, $value:ident $(,$args:expr)*) => {
            |name| $triacylglycerol().struct_().field_by_name(name).fatty_acid().$f($value() $(,$args)*)
        };
    }

    pub(super) fn compute(key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        let triacylglycerol = || col("Triacylglycerol");
        let value = || col("Value");
        #[rustfmt::skip]
        let exprs = [
            tri(index!(monounsaturated, triacylglycerol, value))?.alias("Monounsaturated"),
            tri(index!(polyunsaturated, triacylglycerol, value))?.alias("Polyunsaturated"),
            tri(index!(saturated, triacylglycerol, value))?.alias("Saturated"),
            tri(index!(trans, triacylglycerol, value))?.alias("Trans"),
            tri(index!(unsaturated, triacylglycerol, value, None))?.alias("Unsaturated"),
            tri(index!(unsaturated, triacylglycerol, value, NonZeroI8::new(-9)))?.alias("Unsaturated-9"),
            tri(index!(unsaturated, triacylglycerol, value, NonZeroI8::new(-6)))?.alias("Unsaturated-6"),
            tri(index!(unsaturated, triacylglycerol, value, NonZeroI8::new(-3)))?.alias("Unsaturated-3"),
            tri(index!(unsaturated, triacylglycerol, value, NonZeroI8::new(9)))?.alias("Unsaturated9"),
            tri(index!(eicosapentaenoic_and_docosahexaenoic, triacylglycerol, value))?.alias("EicosapentaenoicAndDocosahexaenoic"),
            tri(index!(fish_lipid_quality, triacylglycerol, value))?.alias("FishLipidQuality"),
            tri(index!(health_promoting_index, triacylglycerol, value))?.alias("HealthPromotingIndex"),
            tri(index!(hypocholesterolemic_to_hypercholesterolemic, triacylglycerol, value))?.alias("HypocholesterolemicToHypercholesterolemic"),
            tri(index!(index_of_atherogenicity, triacylglycerol, value))?.alias("IndexOfAtherogenicity"),
            tri(index!(index_of_thrombogenicity, triacylglycerol, value))?.alias("IndexOfThrombogenicity"),
            tri(index!(linoleic_to_alpha_linolenic, triacylglycerol, value))?.alias("LinoleicToAlphaLinolenic"),
            tri(index!(polyunsaturated_to_saturated, triacylglycerol, value))?.alias("PolyunsaturatedToSaturated"),
            tri(index!(unsaturation_index, triacylglycerol, value))?.alias("UnsaturationIndex"),
        ];
        lazy_frame = lazy_frame.clone().select(exprs);
        lazy_frame.collect()
    }

    fn tri(index: impl Fn(&str) -> Expr) -> PolarsResult<Expr> {
        concat_arr(vec![
            index("StereospecificNumber1"),
            index("StereospecificNumber2"),
            index("StereospecificNumber3"),
        ])
    }
}

mod r#struct {
    use super::*;

    macro_rules! repetitions {
        ($f:ident, $triacylglycerol:ident, $value:ident $(,$args:expr)*) => {
            |name| concat_list((0..3).map(|index| {
                $triacylglycerol().struct_().field_by_name(name).fatty_acid().$f($value(index) $(,$args)*)
            }).collect::<Vec<_>>())
        };
    }

    pub(super) fn compute(key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        let triacylglycerol = || col("Triacylglycerol");
        let value = |index| {
            col("Value")
                .struct_()
                .field_by_name("Repetitions")
                .list()
                .get(lit(index), false)
        };
        #[rustfmt::skip]
        let exprs = [
            tri(repetitions!(monounsaturated, triacylglycerol, value), key.ddof)?.alias("Monounsaturated"),
            tri(repetitions!(polyunsaturated, triacylglycerol, value), key.ddof)?.alias("Polyunsaturated"),
            tri(repetitions!(saturated, triacylglycerol, value), key.ddof)?.alias("Saturated"),
            tri(repetitions!(trans, triacylglycerol, value), key.ddof)?.alias("Trans"),
            tri(repetitions!(unsaturated, triacylglycerol, value, None), key.ddof)?.alias("Unsaturated"),
            tri(repetitions!(unsaturated, triacylglycerol, value, NonZeroI8::new(-9)), key.ddof)?.alias("Unsaturated-9"),
            tri(repetitions!(unsaturated, triacylglycerol, value, NonZeroI8::new(-6)), key.ddof)?.alias("Unsaturated-6"),
            tri(repetitions!(unsaturated, triacylglycerol, value, NonZeroI8::new(-3)), key.ddof)?.alias("Unsaturated-3"),
            tri(repetitions!(unsaturated, triacylglycerol, value, NonZeroI8::new(9)), key.ddof)?.alias("Unsaturated9"),
            tri(repetitions!(eicosapentaenoic_and_docosahexaenoic, triacylglycerol, value), key.ddof)?.alias("EicosapentaenoicAndDocosahexaenoic"),
            tri(repetitions!(fish_lipid_quality, triacylglycerol, value), key.ddof)?.alias("FishLipidQuality"),
            tri(repetitions!(health_promoting_index, triacylglycerol, value), key.ddof)?.alias("HealthPromotingIndex"),
            tri(repetitions!(hypocholesterolemic_to_hypercholesterolemic, triacylglycerol, value), key.ddof)?.alias("HypocholesterolemicToHypercholesterolemic"),
            tri(repetitions!(index_of_atherogenicity, triacylglycerol, value), key.ddof)?.alias("IndexOfAtherogenicity"),
            tri(repetitions!(index_of_thrombogenicity, triacylglycerol, value), key.ddof)?.alias("IndexOfThrombogenicity"),
            tri(repetitions!(linoleic_to_alpha_linolenic, triacylglycerol, value), key.ddof)?.alias("LinoleicToAlphaLinolenic"),
            tri(repetitions!(polyunsaturated_to_saturated, triacylglycerol, value), key.ddof)?.alias("PolyunsaturatedToSaturated"),
            tri(repetitions!(unsaturation_index, triacylglycerol, value), key.ddof)?.alias("UnsaturationIndex"),
        ];
        lazy_frame = lazy_frame.clone().select(exprs);
        lazy_frame.collect()
    }

    fn tri(repetitions: impl Fn(&str) -> PolarsResult<Expr>, ddof: u8) -> PolarsResult<Expr> {
        let index = |name| -> PolarsResult<_> {
            Ok(as_struct(vec![
                repetitions(name)?.list().mean().alias("Mean"),
                repetitions(name)?
                    .list()
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
