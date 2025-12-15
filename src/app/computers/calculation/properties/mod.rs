use crate::{
    app::states::calculation::settings::{Indices, Settings},
    r#const::*,
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use std::num::NonZeroI8;
use tracing::instrument;

/// Calculation properties computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation properties computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // Filter threshold
        if key.threshold_filter {
            lazy_frame = lazy_frame.filter(col(THRESHOLD));
        }
        // Compute
        lazy_frame = compute(lazy_frame, key)?;
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation properties key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) indices: &'a Indices,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) threshold_filter: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            indices: &settings.indices,
            precision: settings.precision,
            significant: settings.significant,
            threshold_filter: settings.threshold.filter,
        }
    }
}

/// Calculation properties value
type Value = DataFrame;

fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let mut exprs = Vec::with_capacity(4);
    // Names
    exprs.push(lit(Series::from_iter(
        key.indices
            .iter()
            .filter_map(|index| index.visible.then_some(index.name.as_str())),
    )
    .with_name(PlSmallStr::from_static(NAME))));
    // Stereospecific numbers
    for name in [
        STEREOSPECIFIC_NUMBERS123,
        STEREOSPECIFIC_NUMBERS13,
        STEREOSPECIFIC_NUMBERS2,
    ] {
        let expr = concat_arr(
            key.indices
                .iter()
                .filter(|index| index.visible)
                .map(|index| {
                    let array =
                        eval_arr(
                            col(name).struct_().field_by_name(SAMPLE),
                            |expr| match &*index.name {
                                "Saturated" => saturated(expr),
                                "Monounsaturated" => monounsaturated(expr),
                                "Polyunsaturated" => polyunsaturated(expr),
                                "Unsaturated" => unsaturated(expr, None),
                                "Unsaturated-9" => unsaturated(expr, NonZeroI8::new(-9)),
                                "Unsaturated-6" => unsaturated(expr, NonZeroI8::new(-6)),
                                "Unsaturated-3" => unsaturated(expr, NonZeroI8::new(-3)),
                                "Unsaturated9" => unsaturated(expr, NonZeroI8::new(9)),
                                "Trans" => trans(expr),
                                "Conjugated" => conjugated(expr),
                                "EicosapentaenoicAndDocosahexaenoic" => {
                                    eicosapentaenoic_and_docosahexaenoic(expr)
                                }
                                "FishLipidQuality" => fish_lipid_quality(expr),
                                "HealthPromotingIndex" => health_promoting_index(expr),
                                "HypocholesterolemicToHypercholesterolemic" => {
                                    hypocholesterolemic_to_hypercholesterolemic(expr)
                                }
                                "IndexOfAtherogenicity" => index_of_atherogenicity(expr),
                                "IndexOfThrombogenicity" => index_of_thrombogenicity(expr),
                                "LinoleicToAlphaLinolenic" => linoleic_to_alpha_linolenic(expr),
                                "Polyunsaturated-6ToPolyunsaturated-3" => {
                                    polyunsaturated_6_to_polyunsaturated_3(expr)
                                }
                                "PolyunsaturatedToSaturated" => polyunsaturated_to_saturated(expr),
                                "UnsaturationIndex" => unsaturation_index(expr),
                                "IodineValue" => iodine_value(expr),
                                _ => unreachable!(),
                            },
                        )?;
                    Ok(as_struct(vec![
                        array
                            .clone()
                            .arr()
                            .mean()
                            .precision(key.precision, key.significant)
                            .alias(MEAN),
                        array
                            .clone()
                            .arr()
                            .std(key.ddof)
                            .precision(key.precision + 1, key.significant)
                            .alias(STANDARD_DEVIATION),
                        array
                            .arr()
                            .eval(element().precision(key.precision, key.significant), false)
                            .alias(SAMPLE),
                    ]))
                })
                .collect::<PolarsResult<_>>()?,
        )?
        .explode()
        .alias(name);
        exprs.push(expr);
    }
    Ok(lazy_frame.select(exprs))
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
