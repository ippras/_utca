use crate::{app::panes::composition::settings::Settings, special::composition::*, utils::Hashed};
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
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        lazy_frame = filter(lazy_frame, key.settings);
        lazy_frame.collect()
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
    pub(crate) data_frame: &'a Hashed<DataFrame>,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash(state);
        self.settings.index.hash(state);
    }
}

/// Filtered composition value
type Value = DataFrame;

fn filter(lazy_frame: LazyFrame, settings: &Settings) -> LazyFrame {
    // println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());
    let mut predicate = lit(true);
    for (index, selection) in settings.special.selections.iter().enumerate() {
        // Key
        for (key, value) in &selection.filter.key {
            let expr = col("Keys").struct_().field_by_index(index as _);
            match selection.composition {
                MMC | NMC | TMC | UMC if value[0] => {
                    predicate = predicate.and(expr.neq(lit(LiteralValue::from(key.clone()))));
                }
                _ => {
                    let expr = expr.tag();
                    if value[0] {
                        predicate = predicate
                            .and(expr.clone().sn1().neq(lit(LiteralValue::from(key.clone()))));
                    }
                    if value[1] {
                        predicate = predicate
                            .and(expr.clone().sn2().neq(lit(LiteralValue::from(key.clone()))));
                    }
                    if value[2] {
                        predicate = predicate
                            .and(expr.clone().sn3().neq(lit(LiteralValue::from(key.clone()))));
                    }
                }
            }
        }
        // Value
        let mut expr = col("Values").arr().get(lit(index as u32), false);
        if settings.index.is_none() {
            expr = expr.struct_().field_by_name("Mean");
        }
        predicate = predicate.and(expr.gt_eq(lit(selection.filter.value)));
    }
    lazy_frame.filter(predicate)
}
