use lipid::triacylglycerol::Stereospecificity;
use serde::{Deserialize, Serialize};

pub const MC: Composition = Composition {
    kind: Kind::Mass,
    stereospecificity: None,
    agregation: false,
};
pub const PMC: Composition = Composition {
    kind: Kind::Mass,
    stereospecificity: Some(Stereospecificity::Positional),
    agregation: false,
};
pub const SMC: Composition = Composition {
    kind: Kind::Mass,
    stereospecificity: Some(Stereospecificity::Stereo),
    agregation: false,
};
pub const EC: Composition = Composition {
    kind: Kind::Ecn,
    stereospecificity: None,
    agregation: false,
};
pub const PEC: Composition = Composition {
    kind: Kind::Ecn,
    stereospecificity: Some(Stereospecificity::Positional),
    agregation: false,
};
pub const SEC: Composition = Composition {
    kind: Kind::Ecn,
    stereospecificity: Some(Stereospecificity::Stereo),
    agregation: false,
};
pub const SC: Composition = Composition {
    kind: Kind::Species,
    stereospecificity: None,
    agregation: false,
};
pub const PSC: Composition = Composition {
    kind: Kind::Species,
    stereospecificity: Some(Stereospecificity::Positional),
    agregation: false,
};
pub const SSC: Composition = Composition {
    kind: Kind::Species,
    stereospecificity: Some(Stereospecificity::Stereo),
    agregation: false,
};
pub const TC: Composition = Composition {
    kind: Kind::Type,
    stereospecificity: None,
    agregation: false,
};
pub const PTC: Composition = Composition {
    kind: Kind::Type,
    stereospecificity: Some(Stereospecificity::Positional),
    agregation: false,
};
pub const STC: Composition = Composition {
    kind: Kind::Type,
    stereospecificity: Some(Stereospecificity::Stereo),
    agregation: false,
};
pub const UC: Composition = Composition {
    kind: Kind::Unsaturation,
    stereospecificity: None,
    agregation: false,
};
pub const PUC: Composition = Composition {
    kind: Kind::Unsaturation,
    stereospecificity: Some(Stereospecificity::Positional),
    agregation: false,
};
pub const SUC: Composition = Composition {
    kind: Kind::Unsaturation,
    stereospecificity: Some(Stereospecificity::Stereo),
    agregation: false,
};

pub const UPCA: Union = Union {
    kind: Kind::Unsaturation,
    stereospecificity: Some(Stereospecificity::Stereo),
    agregation: false,
};

/// Union
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Union {
    pub kind: Kind,
    pub stereospecificity: Option<Stereospecificity>,
    pub agregation: bool,
}

/// Composition
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Composition {
    pub kind: Kind,
    pub agregation: bool,
    pub stereospecificity: Option<Stereospecificity>,
}

impl Composition {
    pub const fn new() -> Self {
        Self {
            kind: Kind::Species,
            agregation: false,
            stereospecificity: Some(Stereospecificity::Positional),
        }
    }
}

impl Default for Composition {
    fn default() -> Self {
        Self::new()
    }
}

/// Composition kind
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Kind {
    Ecn,
    Mass,
    Species,
    Type,
    Unsaturation,
}
