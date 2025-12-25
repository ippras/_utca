use egui::{Context, Id};
use serde::{Deserialize, Serialize};

// const ID: Id = Id::from_hash(ahash::RandomState::with_seeds(1, 2, 3, 4).hash_one("Cache"));

/// Cache
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub(crate) struct Cache {
    pub(crate) fatty_acids: Vec<String>,
}

impl Cache {
    pub(crate) fn new() -> Self {
        Self {
            fatty_acids: Vec::new(),
        }
    }
}

impl Cache {
    pub(crate) fn load(ctx: &Context, id: Id) -> Self {
        ctx.data_mut(|data| {
            data.get_persisted_mut_or_insert_with(id, || Self::new())
                .clone()
        })
    }

    pub(crate) fn store(self, ctx: &Context, id: Id) {
        ctx.data_mut(|data| {
            data.insert_persisted(id, self);
        });
    }
}
