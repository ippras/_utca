use crate::utils::Hashed;
use egui::util::cache::{ComputerMut, FrameCache};
use polars::prelude::*;
use polars_ext::prelude::*;ExprIfExt as _;
use std::hash::{Hash, Hasher};

/// Display computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Display computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        lazy_frame = lazy_frame
            .with_column(key.expr.clone().struct_().field_by_names(["*"]))
            .select([
                col("Mean").percent(key.percent),
                col("StandardDeviation").percent(key.percent),
                col("Array").percent(key.percent),
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

/// Display key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a Hashed<DataFrame>,
    pub(crate) expr: &'a Expr,
    pub(crate) percent: bool,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash.hash(state);
        self.expr.hash(state);
        self.percent.hash(state);
    }
}

/// Display value
type Value = DataFrame;

#[derive(Clone, Copy, Debug, Hash)]
pub enum Display {
    EnrichmentFactor,
    SelectivityFactor,
}
