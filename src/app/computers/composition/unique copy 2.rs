use crate::{
    app::panes::composition::settings::Selection,
    special::composition::{MMC, MSC, NMC, NSC, SMC, SPC, SSC, TMC, TPC, TSC, UMC, USC},
    utils::Hashed,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::polars::ExprExt;
use polars::prelude::*;
use std::{
    collections::VecDeque,
    hash::{Hash, Hasher},
};

/// Unique composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Unique composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let lazy_frame = key.data_frame.value.clone().lazy();
        let mut exprs = Vec::new();
        // println!(
        //     "lazy_frame unique x: {}",
        //     lazy_frame.clone().collect().unwrap()
        // );
        for (index, selection) in key.selections.iter().enumerate() {
            match selection.composition {
                MMC | NMC | SMC | TMC | UMC => {
                    println!(
                        "lazy_frame y: {}",
                        lazy_frame
                            .clone()
                            .select([col("Keys")
                                .struct_()
                                .field_by_index(index as _)
                                .unique()
                                .sort(Default::default())])
                            .collect()
                            .unwrap()
                    );
                    exprs.push(
                        col("Keys")
                            .struct_()
                            .field_by_index(index as _)
                            .unique()
                            .sort(Default::default()),
                    )
                }
                MSC | NSC | SPC | SSC | TPC | TSC | USC => exprs.push(
                    col("Keys")
                        .tag()
                        .sn1()
                        .struct_()
                        .field_by_index(index as _)
                        .unique()
                        .sort(Default::default()),
                ),
            }
        }
        // println!(
        //     "lazy_frame y: {exprs:?} {}",
        //     lazy_frame.clone().select(exprs.clone()).collect().unwrap()
        // );
        lazy_frame.select(exprs).collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Unique composition key
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a Hashed<DataFrame>,
    pub(crate) selections: &'a VecDeque<Selection>,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash(state);
        self.selections.hash(state);
    }
}

/// Unique composition value
type Value = DataFrame;
