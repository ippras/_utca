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
use polars_ext::expr::ExprExt;
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
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter minor
        if key.filter {
            lazy_frame = lazy_frame.filter(col("Filter"));
        }
        // Compute
        lazy_frame = compute(lazy_frame, key)?;
        // Format
        lazy_frame = format(lazy_frame, key)?;
        lazy_frame.collect()
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
    pub(crate) filter: bool,
    pub(crate) indices: &'a Indices,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            filter: settings.threshold.filter,
            indices: &settings.indices,
            precision: settings.precision,
            significant: settings.significant,
        }
    }
}

/// Calculation indices value
type Value = DataFrame;

fn length(data_frame: &DataFrame) -> PolarsResult<usize> {
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
    return Ok(length);
}

fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let length = length(&key.frame)?;
    let iter = |expr: Expr| {
        (0..length).map(move |index| {
            expr.clone()
                .struct_()
                .field_by_name("Array")
                .arr()
                .get(lit(index as IdxSize), false)
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
    };
    let property = |f: fn(Expr) -> Expr| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS123, f)?,
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS13, f)?,
            stereospecific_numbers(STEREOSPECIFIC_NUMBERS2, f)?,
        ]))
    };
    Ok(lazy_frame.select([
        property(conjugated)?.alias("Conjugated"),
        property(monounsaturated)?.alias("Monounsaturated"),
        property(polyunsaturated)?.alias("Polyunsaturated"),
        property(saturated)?.alias("Saturated"),
        property(trans)?.alias("Trans"),
        property(|expr| unsaturated(expr, None))?.alias("Unsaturated"),
        property(|expr| unsaturated(expr, NonZeroI8::new(-9)))?.alias("Unsaturated-9"),
        property(|expr| unsaturated(expr, NonZeroI8::new(-6)))?.alias("Unsaturated-6"),
        property(|expr| unsaturated(expr, NonZeroI8::new(-3)))?.alias("Unsaturated-3"),
        property(|expr| unsaturated(expr, NonZeroI8::new(9)))?.alias("Unsaturated9"),
        //
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
    ]))
}

fn format(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let stereospecific_numbers = |expr: Expr| -> PolarsResult<Expr> {
        Ok(expr.clone().struct_().with_fields(vec![
            expr.clone()
                .struct_()
                .field_by_name("Mean")
                .precision(key.precision, key.significant)
                .cast(DataType::String),
            format_standard_deviation(
                expr.clone()
                    .struct_()
                    .field_by_name("StandardDeviation")
                    .precision(key.precision, key.significant),
            )?,
            format_sample(
                expr.struct_().field_by_name("Sample").arr().eval(
                    element()
                        .precision(key.precision, key.significant)
                        .cast(DataType::String),
                    false,
                ),
            )?,
        ]))
    };
    let property = |name: &str| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            stereospecific_numbers(col(name).struct_().field_by_name(STEREOSPECIFIC_NUMBERS123))?,
            stereospecific_numbers(col(name).struct_().field_by_name(STEREOSPECIFIC_NUMBERS13))?,
            stereospecific_numbers(col(name).struct_().field_by_name(STEREOSPECIFIC_NUMBERS2))?,
        ])
        .alias(name))
    };
    Ok(lazy_frame.select([
        property("Conjugated")?,
        property("Monounsaturated")?,
        property("Polyunsaturated")?,
        property("Saturated")?,
        property("Trans")?,
        property("Unsaturated-3")?,
        property("Unsaturated-6")?,
        property("Unsaturated-9")?,
        property("Unsaturated")?,
        property("Unsaturated9")?,
        //
        property("EicosapentaenoicAndDocosahexaenoic")?,
        property("FishLipidQuality")?,
        property("HealthPromotingIndex")?,
        property("HypocholesterolemicToHypercholesterolemic")?,
        property("IndexOfAtherogenicity")?,
        property("IndexOfThrombogenicity")?,
        property("LinoleicToAlphaLinolenic")?,
        property("Polyunsaturated-6ToPolyunsaturated-3")?,
        property("PolyunsaturatedToSaturated")?,
        property("UnsaturationIndex")?,
        property("IodineValue")?,
    ]))
}

fn conjugated(expr: Expr) -> Expr {
    col(FATTY_ACID).fatty_acid().conjugated(expr)
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
    (expr * col(FATTY_ACID).fatty_acid().iodine_value()).sum()
}

pub(crate) mod biodiesel;
