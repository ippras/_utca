use crate::{
    app::states::composition::{Discriminants, Method, Settings},
    r#const::VALUE,
    utils::{HashedDataFrame, HashedMetaDataFrame},
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use tracing::instrument;

/// Starts with `VALUE`
const VALUE_: &str = formatcp!(r#"^{VALUE}.*$"#);

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
        let compute = |frame: &HashedMetaDataFrame| {
            // Это вызов fn compute.
            compute(frame.data.data_frame.clone().lazy(), key)
        };
        let mut lazy_frame = indexed(compute(&frames[0])?, 0);
        // println!("spec 0: {}", lazy_frame.clone().collect().unwrap());
        for index in 1..frames.len() {
            lazy_frame = lazy_frame.join(
                indexed(compute(&key.frames[index])?, index),
                [col(LABEL), col(TRIACYLGLYCEROL)],
                [col(LABEL), col(TRIACYLGLYCEROL)],
                JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
            );
        }
        lazy_frame = lazy_frame.select([
            col(LABEL),
            col(TRIACYLGLYCEROL),
            concat_arr(vec![col(VALUE_)])?.alias(VALUE),
        ]);
        // println!("spec 1: {}", lazy_frame.clone().collect().unwrap());
        // Mean, standard deviation, sample
        // lazy_frame = mean_and_standard_deviation(lazy_frame, key)?;
        println!("S 2: {}", lazy_frame.clone().collect().unwrap());
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
///
/// | Label     | Triacylglycerol | Value         |
/// | --------- | --------------- | ------------- |
/// | struct[3] | struct[3]       | array[f64, n] |
type Value = HashedDataFrame;

fn indexed(lazy_frame: LazyFrame, index: usize) -> LazyFrame {
    lazy_frame.select([
        col(LABEL),
        col(TRIACYLGLYCEROL),
        col(VALUE).name().suffix(&format!("[{index}]")),
    ])
}

// From:
//
// | Label | FattyAcid | StereospecificNumbers123 | StereospecificNumbers13 | StereospecificNumbers2 |
// | ----- | --------- | ------------------------ | ----------------------- | ---------------------- |
// | str   | struct[2] | f64                      | f64                     | f64                    |
//
// To:
//
// | Label     | Triacylglycerol | Value |
// | --------- | --------------- | ----- |
// | struct[3] | struct[3]       | f64   |
fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    match key.method {
        Method::Gunstone => gunstone::compute(lazy_frame, key.discriminants),
        Method::MartinezForce => martinez_force::compute(lazy_frame),
        Method::VanderWal => vander_wal::compute(lazy_frame),
    }
}

// fn mean_and_standard_deviation(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
//     let array = concat_arr(vec![col(VALUE_)])?;
//     Ok(lazy_frame.with_column(
//         as_struct(vec![
//             array.clone().arr().mean().alias(MEAN),
//             array.clone().arr().std(key.ddof).alias(STANDARD_DEVIATION),
//             array.alias(SAMPLE),
//         ])
//         .alias(VALUE),
//     ))
// }

mod gunstone;
mod martinez_force;
mod vander_wal;
