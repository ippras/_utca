use crate::{
    app::panes::composition::settings::{ECN_MONO, MASS_MONO, Settings, TYPE_MONO, UNSATURATION_MONO},
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use std::hash::{Hash, Hasher};

/// Filtered composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Filtered composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.data_frame.clone().lazy();
        lazy_frame = filter(lazy_frame, key.settings);
        let mut data_frame = lazy_frame.collect()?;
        let hash = data_frame.hash_rows(None)?.xor_reduce().unwrap_or_default();
        Ok(HashedDataFrame {
            data_frame,
            hash,
        })
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Filtered composition key
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a HashedDataFrame,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash(state);
        self.settings.index.hash(state);
    }
}

/// Filtered composition value
type Value = HashedDataFrame;

fn filter(lazy_frame: LazyFrame, settings: &Settings) -> LazyFrame {
    let mut predicate = lit(true);
    for (index, selection) in settings.special.selections.iter().enumerate() {
        // Key
        for (key, value) in &selection.filter.key {
            let expr = col("Keys").struct_().field_by_index(index as _);
            match selection.composition {
                MASS_MONO | ECN_MONO | TYPE_MONO | UNSATURATION_MONO if value[0] => {
                    predicate = predicate.and(expr.neq(lit(LiteralValue::from(key.clone()))));
                }
                _ => {
                    let expr = expr.triacylglycerol();
                    if value[0] {
                        predicate = predicate.and(
                            expr.clone()
                                .stereospecific_number1()
                                .neq(lit(LiteralValue::from(key.clone()))),
                        );
                    }
                    if value[1] {
                        predicate = predicate.and(
                            expr.clone()
                                .stereospecific_number2()
                                .neq(lit(LiteralValue::from(key.clone()))),
                        );
                    }
                    if value[2] {
                        predicate = predicate.and(
                            expr.clone()
                                .stereospecific_number3()
                                .neq(lit(LiteralValue::from(key.clone()))),
                        );
                    }
                }
            }
        }
        // Value
        let expr = col("Values")
            .arr()
            .get(lit(index as u32), false)
            .struct_()
            .field_by_name("Mean");
        predicate = predicate.and(expr.gt_eq(lit(selection.filter.value)));
    }
    lazy_frame.filter(predicate)
}
