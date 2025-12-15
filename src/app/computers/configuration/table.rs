use crate::utils::HashedDataFrame;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;

/// Table computed
pub type Computed = FrameCache<Value, Computer>;

/// Table computer
#[derive(Default)]
pub struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = lazy_frame.select([
            col(LABEL),
            col(FATTY_ACID),
            col(STEREOSPECIFIC_NUMBERS123),
            nth(3).as_expr(),
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

/// Table key
#[derive(Clone, Copy, Debug, Hash)]
pub struct Key<'a> {
    pub frame: &'a HashedDataFrame,
    // pub kind: Kind,
    // pub percent: bool,
}

/// Table value
type Value = DataFrame;
