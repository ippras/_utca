use egui::Color32;

#[derive(Clone, Copy, Debug)]
pub(crate) enum Sign<T> {
    Negative(T),
    Zero,
    Positive(T),
}

impl Sign<f64> {
    pub(crate) fn chaddock(&self) -> Sign<Chaddock> {
        match *self {
            Sign::Negative(value) => Sign::Negative(Chaddock::from(value)),
            Sign::Zero => Sign::Zero,
            Sign::Positive(value) => Sign::Positive(Chaddock::from(value)),
        }
    }
}

impl Sign<f64> {
    pub(crate) fn color(&self, source: Color32) -> Color32 {
        match *self {
            Sign::Negative(value) => source.lerp_to_gamma(Color32::RED, value as _),
            Sign::Zero => source,
            Sign::Positive(value) => source.lerp_to_gamma(Color32::BLUE, value as _),
        }
    }
}

impl Sign<Chaddock> {
    pub(crate) fn color(&self, source: Color32) -> Color32 {
        match self {
            Sign::Negative(Chaddock::VeryStrong) => source.lerp_to_gamma(Color32::RED, 1.0),
            Sign::Negative(Chaddock::Strong) => source.lerp_to_gamma(Color32::RED, 0.8),
            Sign::Negative(Chaddock::Moderate) => source.lerp_to_gamma(Color32::RED, 0.6),
            Sign::Negative(Chaddock::Weak) => source.lerp_to_gamma(Color32::RED, 0.4),
            Sign::Negative(Chaddock::VeryWeak) => source.lerp_to_gamma(Color32::RED, 0.2),
            Sign::Zero => source,
            Sign::Positive(Chaddock::VeryWeak) => source.lerp_to_gamma(Color32::BLUE, 0.2),
            Sign::Positive(Chaddock::Weak) => source.lerp_to_gamma(Color32::BLUE, 0.4),
            Sign::Positive(Chaddock::Moderate) => source.lerp_to_gamma(Color32::BLUE, 0.6),
            Sign::Positive(Chaddock::Strong) => source.lerp_to_gamma(Color32::BLUE, 0.8),
            Sign::Positive(Chaddock::VeryStrong) => source.lerp_to_gamma(Color32::BLUE, 1.0),
        }
    }
}

impl From<f64> for Sign<f64> {
    fn from(value: f64) -> Self {
        if value < 0.0 {
            Sign::Negative(value.abs())
        } else if value > 0.0 {
            Sign::Positive(value.abs())
        } else {
            Sign::Zero
        }
    }
}

/// Chaddock
#[derive(Clone, Copy, Debug)]
pub(crate) enum Chaddock {
    VeryStrong,
    Strong,
    Moderate,
    Weak,
    VeryWeak,
}

impl From<f64> for Chaddock {
    fn from(value: f64) -> Self {
        match value.abs() {
            0.0..0.3 => Self::VeryWeak,
            0.3..0.5 => Self::Weak,
            0.5..0.7 => Self::Moderate,
            0.7..0.9 => Self::Strong,
            0.9.. => Self::VeryStrong,
            _ => unreachable!(),
        }
    }
}
