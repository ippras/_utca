use crate::{
    app::states::composition::{Discriminants, Method, Settings},
    utils::{HashedDataFrame, HashedMetaDataFrame},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use tracing::instrument;

/// Composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    pub(super) fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        // Чтобы обрабатывать универсально - даже при одно фрейме берем слайс.
        let frames = match key.index {
            Some(index) => &key.frames[index..=index],
            None => &key.frames[..],
        };
        let compute = |frame: &HashedMetaDataFrame, suffix: &str| -> PolarsResult<LazyFrame> {
            // Это вызов fn compute.
            Ok(compute(frame.data.data_frame.clone().lazy(), key)?.select([
                col(LABEL),
                col(TRIACYLGLYCEROL),
                col("Value").name().suffix(suffix),
            ]))
        };
        let mut lazy_frame = compute(&frames[0], "[0]")?;
        println!("spec 0: {}", lazy_frame.clone().collect().unwrap());
        for (index, frame) in frames[1..].iter().enumerate() {
            lazy_frame = lazy_frame.join(
                compute(frame, &index.to_string())?,
                [col(LABEL), col(TRIACYLGLYCEROL)],
                [col(LABEL), col(TRIACYLGLYCEROL)],
                JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
            );
        }
        println!("spec 1: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = lazy_frame.select(mean_and_standard_deviation(key.ddof)?);
        // println!("spec 2: {}", lazy_frame.clone().collect().unwrap());
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Composition key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frames: &'a [HashedMetaDataFrame],
    pub(crate) index: Option<usize>,
    pub(crate) ddof: u8,
    pub(crate) discriminants: &'a Discriminants,
    pub(crate) method: Method,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frames: &'a [HashedMetaDataFrame], settings: &'a Settings) -> Self {
        Self {
            frames,
            index: settings.index,
            ddof: settings.ddof,
            discriminants: &settings.discriminants,
            method: settings.method,
        }
    }
}

/// Composition value
type Value = HashedDataFrame;

// From:
// ┌────────────────┬────────────────────┬────────────────────┬───────────────────┬───────────────────┐
// │ Label          ┆ FattyAcid          ┆ StereospecificNumb ┆ StereospecificNum ┆ StereospecificNum │
// │ ---            ┆ ---                ┆ ers123             ┆ bers13            ┆ bers2             │
// │ str            ┆ struct[2]          ┆ ---                ┆ ---               ┆ ---               │
// │                ┆                    ┆ f64                ┆ f64               ┆ f64               │
// ╞════════════════╪════════════════════╪════════════════════╪═══════════════════╪═══════════════════╡
// To:
// ┌───────────────────────────────────────────┬───────────────────────────────────────────┬───────┐
// │ Label                                     ┆ Triacylglycerol                           ┆ Value │
// │ ---                                       ┆ ---                                       ┆ ---   │
// │ struct[3]                                 ┆ struct[3]                                 ┆ f64   │
// ╞═══════════════════════════════════════════╪═══════════════════════════════════════════╪═══════╡
fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    match key.method {
        Method::Gunstone => gunstone::compute(lazy_frame, key.discriminants),
        Method::MartinezForce => martinez_force::compute(lazy_frame),
        Method::VanderWal => vander_wal::compute(lazy_frame),
    }
}

fn mean_and_standard_deviation(ddof: u8) -> PolarsResult<[Expr; 3]> {
    let array = || concat_arr(vec![all().exclude_cols([LABEL, TRIACYLGLYCEROL]).as_expr()]);
    Ok([
        col(LABEL),
        col(TRIACYLGLYCEROL),
        as_struct(vec![
            array()?.arr().mean().alias("Mean"),
            array()?.arr().std(ddof).alias("StandardDeviation"),
            array()?.alias("Array"),
        ])
        .alias("Value"),
    ])
}

mod gunstone;
mod martinez_force;
mod vander_wal;
