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
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        if !is_many(&key.data_frame)? {
            lazy_frame = lazy_frame.select(compute_one(
                col("FattyAcid").fatty_acid(),
                col("Experimental")
                    .struct_()
                    .field_by_name("Triacylglycerol")
                    .alias("Value"),
            ));
        } else {
            println!(
                "lazy_frame1!!!!!!!!!!!!!: {}",
                lazy_frame.clone().collect().unwrap()
            );
            // Repetitions
            let exprs = compute_many(
                col("FattyAcid").fatty_acid(),
                (0..3).map(|index| {
                    col("Experimental")
                        .struct_()
                        .field_by_name("Triacylglycerol")
                        .alias("Value")
                        .struct_()
                        .field_by_name("Values")
                        .list()
                        .get(index.into(), false)
                }),
                // col("Experimental")
                //     .struct_()
                //     .field_by_name("Triacylglycerol")
                //     .alias("Value")
                //     .struct_()
                //     .field_by_name("Values")
                //     .list()
                //     .eval(col("")),
            )?;
            lazy_frame = lazy_frame.select(exprs);
            println!(
                "lazy_frame2!!!!!!!!!!!!!: {}",
                lazy_frame.clone().collect().unwrap()
            );
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
        }
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
    pub(crate) data_frame: Hashed<&'a DataFrame>,
    pub(crate) ddof: u8,
}

/// Calculation indices value
type Value = DataFrame;

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

fn is_many(data_frame: &DataFrame) -> PolarsResult<bool> {
    // let Some(fatty_acid) = data_frame.schema().get(FATTY_ACID) else {
    //     polars_ensure!(fatty_acid == data_type!(FATTY_ACID), SchemaMismatch: r#"The "{FATTY_ACID}" field was not found in the scheme."#);
    //     // polars_bail!(SchemaMismatch: r#"The "{FATTY_ACID}" field was not found in the scheme."#);
    // };
    let Some(experimental) = data_frame.schema().get("Experimental") else {
        polars_bail!(SchemaMismatch: r#"The "Experimental" field was not found in the scheme."#);
    };
    let DataType::Struct(fields) = experimental else {
        polars_bail!(SchemaMismatch: r#"The "Experimental" field is not `Struct`."#);
    };
    let Some(triacylglycerol) = fields
        .iter()
        .find(|field| field.name() == "Triacylglycerol")
    else {
        polars_bail!(SchemaMismatch: r#"The "Experimental.Triacylglycerol" field was not found in the scheme."#);
    };
    match triacylglycerol.dtype() {
        DataType::Struct(_) => {
            return Ok(true);
        }
        DataType::Float64 => {
            return Ok(false);
        }
        other_type => {
            polars_bail!(SchemaMismatch: r#"The "Experimental.Triacylglycerol" field has other type: {other_type:?}"#);
        }
    }
}

fn compute_many(
    fatty_acid: FattyAcidExpr,
    values: impl Iterator<Item = Expr> + Clone,
) -> PolarsResult<Vec<Expr>> {
    Ok(vec![
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
        index!(
            hypocholesterolemic_to_hypercholesterolemic,
            fatty_acid,
            values
        )?,
        index!(index_of_atherogenicity, fatty_acid, values)?,
        index!(index_of_thrombogenicity, fatty_acid, values)?,
        index!(linoleic_to_alpha_linolenic, fatty_acid, values)?,
        index!(polyunsaturated_to_saturated, fatty_acid, values)?,
        index!(unsaturation_index, fatty_acid, values)?,
    ])
}

fn compute_one(fatty_acid: FattyAcidExpr, value: Expr) -> Vec<Expr> {
    vec![
        fatty_acid.clone().monounsaturated(value.clone()),
        fatty_acid.clone().polyunsaturated(value.clone()),
        fatty_acid.clone().saturated(value.clone()),
        fatty_acid.clone().trans(value.clone()),
        fatty_acid.clone().unsaturated(value.clone(), None),
        fatty_acid
            .clone()
            .unsaturated(value.clone(), NonZeroI8::new(-9)),
        fatty_acid
            .clone()
            .unsaturated(value.clone(), NonZeroI8::new(-6)),
        fatty_acid
            .clone()
            .unsaturated(value.clone(), NonZeroI8::new(-3)),
        fatty_acid
            .clone()
            .unsaturated(value.clone(), NonZeroI8::new(9)),
        fatty_acid
            .clone()
            .eicosapentaenoic_and_docosahexaenoic(value.clone()),
        fatty_acid.clone().fish_lipid_quality(value.clone()),
        fatty_acid.clone().health_promoting_index(value.clone()),
        fatty_acid
            .clone()
            .hypocholesterolemic_to_hypercholesterolemic(value.clone()),
        fatty_acid.clone().index_of_atherogenicity(value.clone()),
        fatty_acid.clone().index_of_thrombogenicity(value.clone()),
        fatty_acid
            .clone()
            .linoleic_to_alpha_linolenic(value.clone()),
        fatty_acid
            .clone()
            .polyunsaturated_to_saturated(value.clone()),
        fatty_acid.unsaturation_index(value),
    ]
}
