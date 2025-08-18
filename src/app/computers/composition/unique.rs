use crate::{
    app::panes::composition::settings::{
        MMC, MSC, NMC, NSC, SMC, SPC, SSC, TMC, TPC, TSC, UMC, USC,
    },
    utils::Hashed,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use metadata::MetaDataFrame;
use polars::prelude::*;
use std::{
    collections::{BTreeSet, HashSet},
    hash::{Hash, Hasher},
};

/// Unique composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Unique composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut hashes = HashSet::new();
        let mut labels = Vec::new();
        for frame in &key.frames[..] {
            for label in frame.data["Label"].str()? {
                if hashes.insert(label) {
                    labels.push(label.unwrap_or_default().to_owned());
                }
            }
        }
        Ok(labels)
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
    pub(crate) frames: &'a Hashed<Vec<MetaDataFrame>>,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frames.hash(state);
    }
}

/// Unique composition value
type Value = Vec<String>;
