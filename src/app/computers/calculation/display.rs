use crate::{app::panes::calculation::parameters::From, utils::Hashed};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::ExprIfExt as _;
use std::hash::{Hash, Hasher};

/// Display calculation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Display calculation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        println!("Display 0: {}", lazy_frame.clone().collect().unwrap());
        // let exprs = match key.kind {
        //     Kind::StereospecificNumbers123 => col(STEREOSPECIFIC_NUMBERS123)
        //         .struct_()
        //         .field_by_name("Experimental")
        //         .struct_()
        //         .field_by_name("*")
        //         .percent_if(key.percent),
        //     Kind::StereospecificNumbers12_23 => col(STEREOSPECIFIC_NUMBERS12_23)
        //         .struct_()
        //         .field_by_name("Experimental")
        //         .struct_()
        //         .field_by_name("*")
        //         .percent_if(key.percent),
        //     Kind::StereospecificNumbers13(from) => col(STEREOSPECIFIC_NUMBERS13)
        //         .struct_()
        //         .field_by_name(match from {
        //             From::StereospecificNumbers12_23 => STEREOSPECIFIC_NUMBERS12_23,
        //             From::StereospecificNumbers2 => STEREOSPECIFIC_NUMBERS2,
        //         })
        //         .struct_()
        //         .field_by_name("*")
        //         .percent_if(key.percent),
        //     Kind::StereospecificNumbers2 => col(STEREOSPECIFIC_NUMBERS2)
        //         .struct_()
        //         .field_by_name("Experimental")
        //         .struct_()
        //         .field_by_name("*")
        //         .percent_if(key.percent),
        //     Kind::EnrichmentFactor => col("Factors")
        //         .struct_()
        //         .field_by_name("Enrichment")
        //         .struct_()
        //         .field_by_name("*"),
        //     Kind::SelectivityFactor => col("Factors")
        //         .struct_()
        //         .field_by_name("Selectivity")
        //         .struct_()
        //         .field_by_name("*"),
        // };
        lazy_frame = lazy_frame.select([
            experimental(key),
            alternative(key).alias("Alternative"),
            theoretical(key).alias("Theoretical"),
            calculation(key)?.alias("Calculation"),
        ]);
        println!("Display 1: {}", lazy_frame.clone().collect().unwrap());
        let data_frame = lazy_frame.collect()?;
        Ok(data_frame)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Display calculation key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a Hashed<DataFrame>,
    pub(crate) kind: Kind,
    pub(crate) percent: bool,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash.hash(state);
        self.kind.hash(state);
        self.percent.hash(state);
    }
}

/// Display calculation value
type Value = DataFrame;

// Display kind
#[derive(Clone, Copy, Debug, Hash)]
pub enum Kind {
    StereospecificNumbers123,
    StereospecificNumbers12_23,
    StereospecificNumbers13(From),
    StereospecificNumbers2,
    EnrichmentFactor,
    SelectivityFactor,
}

fn experimental(key: Key) -> Expr {
    match key.kind {
        Kind::StereospecificNumbers123 => col(STEREOSPECIFIC_NUMBERS123)
            .struct_()
            .field_by_name("Experimental")
            .struct_()
            .field_by_name("*")
            .percent_if(key.percent),
        Kind::StereospecificNumbers12_23 => col(STEREOSPECIFIC_NUMBERS12_23)
            .struct_()
            .field_by_name("Experimental")
            .struct_()
            .field_by_name("*")
            .percent_if(key.percent),
        Kind::StereospecificNumbers13(from) => col(STEREOSPECIFIC_NUMBERS13)
            .struct_()
            .field_by_name(match from {
                From::StereospecificNumbers12_23 => STEREOSPECIFIC_NUMBERS12_23,
                From::StereospecificNumbers2 => STEREOSPECIFIC_NUMBERS2,
            })
            .struct_()
            .field_by_name("*")
            .percent_if(key.percent),
        Kind::StereospecificNumbers2 => col(STEREOSPECIFIC_NUMBERS2)
            .struct_()
            .field_by_name("Experimental")
            .struct_()
            .field_by_name("*")
            .percent_if(key.percent),
        Kind::EnrichmentFactor => col("Factors")
            .struct_()
            .field_by_name("Enrichment")
            .struct_()
            .field_by_name("*"),
        Kind::SelectivityFactor => col("Factors")
            .struct_()
            .field_by_name("Selectivity")
            .struct_()
            .field_by_name("*"),
    }
}

fn alternative(key: Key) -> Expr {
    match key.kind {
        Kind::StereospecificNumbers13(from) => col(STEREOSPECIFIC_NUMBERS13)
            .struct_()
            .field_by_name(match from {
                From::StereospecificNumbers12_23 => STEREOSPECIFIC_NUMBERS2,
                From::StereospecificNumbers2 => STEREOSPECIFIC_NUMBERS12_23,
            })
            .struct_()
            .field_by_name("Mean")
            .percent_if(key.percent),
        _ => lit(NULL),
    }
}

fn theoretical(key: Key) -> Expr {
    match key.kind {
        Kind::StereospecificNumbers123 => col(STEREOSPECIFIC_NUMBERS123)
            .struct_()
            .field_by_name("Theoretical")
            .struct_()
            .field_by_name("Mean")
            .percent_if(key.percent),
        Kind::StereospecificNumbers12_23 => col(STEREOSPECIFIC_NUMBERS12_23)
            .struct_()
            .field_by_name("Theoretical")
            .struct_()
            .field_by_name("Mean")
            .percent_if(key.percent),
        Kind::StereospecificNumbers2 => col(STEREOSPECIFIC_NUMBERS2)
            .struct_()
            .field_by_name("Theoretical")
            .struct_()
            .field_by_name("Mean")
            .percent_if(key.percent),
        _ => lit(NULL),
    }
}

fn calculation(key: Key) -> PolarsResult<Expr> {
    let mean = |name| {
        col(name)
            .struct_()
            .field_by_name("Experimental")
            .struct_()
            .field_by_name("Mean")
    };
    let standard_deviation = |name| {
        col(name)
            .struct_()
            .field_by_name("Experimental")
            .struct_()
            .field_by_name("StandardDeviation")
    };
    let predicate = standard_deviation(STEREOSPECIFIC_NUMBERS2)
        .is_null()
        .or(standard_deviation(STEREOSPECIFIC_NUMBERS123).is_null());
    Ok(match key.kind {
        Kind::EnrichmentFactor => ternary_expr(
            predicate,
            format_str(
                "{} / (3 * {})",
                [
                    mean(STEREOSPECIFIC_NUMBERS2).percent_if(key.percent),
                    mean(STEREOSPECIFIC_NUMBERS123).percent_if(key.percent),
                ],
            )?,
            lit(NULL),
        ),
        Kind::SelectivityFactor => ternary_expr(
            predicate,
            format_str(
                "({} * {}) / ({} * {})",
                [
                    mean(STEREOSPECIFIC_NUMBERS2).percent_if(key.percent),
                    mean(STEREOSPECIFIC_NUMBERS123)
                        .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                        .sum()
                        .percent_if(key.percent),
                    mean(STEREOSPECIFIC_NUMBERS123).percent_if(key.percent),
                    mean(STEREOSPECIFIC_NUMBERS2)
                        .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                        .sum()
                        .percent_if(key.percent),
                ],
            )?,
            lit(NULL),
        ),
        _ => lit(NULL),
    })
}
