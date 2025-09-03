use crate::utils::Hashed;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::ExprIfExt as _;
use std::hash::{Hash, Hasher};

/// Display factor computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Display factor computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        lazy_frame = lazy_frame
            .with_column(
                col("Factors")
                    .struct_()
                    .field_by_name(match key.factor {
                        Factor::Enrichment => "Enrichment",
                        Factor::Selectivity => "Selectivity",
                    })
                    .struct_()
                    .field_by_names(["*"]),
            )
            .with_column(
                ternary_expr(
                    col("StandardDeviation").is_null(),
                    calculation(key)?,
                    lit(NULL),
                )
                .alias("Calculation"),
            )
            .select([
                col("Mean"),
                col("StandardDeviation"),
                col("Repetitions"),
                col("Calculation"),
            ]);
        let data_frame = lazy_frame.collect()?;
        Ok(data_frame)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Display factor key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a Hashed<DataFrame>,
    pub(crate) factor: Factor,
    pub(crate) percent: bool,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash.hash(state);
        self.factor.hash(state);
        self.percent.hash(state);
    }
}

/// Display factor value
type Value = DataFrame;

/// Factor
#[derive(Clone, Copy, Debug, Hash)]
pub enum Factor {
    Enrichment,
    Selectivity,
}

fn calculation(key: Key) -> PolarsResult<Expr> {
    match key.factor {
        Factor::Enrichment => format_str(
            "{} / {}",
            [
                col(STEREOSPECIFIC_NUMBERS2)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .percent_if(key.percent),
                col(STEREOSPECIFIC_NUMBERS123)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .percent_if(key.percent),
            ],
        ),
        Factor::Selectivity => format_str(
            "({} / {}) / ({} / {})",
            [
                col(STEREOSPECIFIC_NUMBERS2)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .percent_if(key.percent),
                col(STEREOSPECIFIC_NUMBERS123)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .percent_if(key.percent),
                col(STEREOSPECIFIC_NUMBERS2)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                    .sum()
                    .percent_if(key.percent),
                col(STEREOSPECIFIC_NUMBERS123)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                    .sum()
                    .percent_if(key.percent),
            ],
        ),
    }
}
