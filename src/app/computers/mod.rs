#[derive(Clone, Copy, Debug)]
enum Mode {
    One,
    Many(u64),
}

pub(super) mod calculation;
pub(super) mod composition;
pub(super) mod configuration;
