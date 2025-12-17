use crate::{
    app::states::calculation::settings::{Correlation, Settings, StereospecificNumbers, Threshold},
    r#const::{SAMPLE, THRESHOLD},
    utils::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use tracing::instrument;

const CORRELATION: &str = "Correlation";
const LABEL1: &str = formatcp!("{LABEL}[1]");
const LABEL2: &str = formatcp!("{LABEL}[2]");
const SAMPLE1: &str = formatcp!("{SAMPLE}[1]");
const SAMPLE2: &str = formatcp!("{SAMPLE}[2]");

/// Calculation correlation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation correlation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = filter_and_sort(lazy_frame, key);
        lazy_frame = compute(lazy_frame, key)?;
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation correlation key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) chaddock: bool,
    pub(crate) correlation: Correlation,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
    pub(crate) stereospecific_numbers: StereospecificNumbers,
    pub(crate) threshold: &'a Threshold,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            chaddock: settings.chaddock,
            correlation: settings.correlation,
            precision: settings.precision,
            significant: settings.significant,
            stereospecific_numbers: settings.stereospecific_numbers,
            threshold: &settings.threshold,
        }
    }
}

/// Calculation correlation value
type Value = DataFrame;

// Filter and sort threshold (major, minor)
fn filter_and_sort(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    if key.threshold.filter {
        lazy_frame.filter(col(THRESHOLD))
    } else if key.threshold.sort {
        lazy_frame.sort_by_exprs(
            [col(THRESHOLD)],
            SortMultipleOptions::default()
                .with_maintain_order(true)
                .with_order_reversed(),
        )
    } else {
        lazy_frame
    }
}

fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Labels
    // Нужны отфильтрованные и отсортированные метки.
    let labels = lazy_frame.clone().select([col(LABEL)]).collect()?;
    // Select
    lazy_frame = lazy_frame.select([
        col(LABEL),
        col(key.stereospecific_numbers.to_string())
            .struct_()
            .field_by_name(SAMPLE),
    ]);
    // Cross join
    // Установить maintain_order
    lazy_frame = lazy_frame
        .clone()
        .select([
            col(LABEL).name().suffix("[1]"),
            col(SAMPLE).name().suffix("[1]"),
        ])
        .join(
            lazy_frame.select([
                col(LABEL).name().suffix("[2]"),
                col(SAMPLE).name().suffix("[2]"),
            ]),
            vec![],
            vec![],
            JoinArgs {
                how: JoinType::Cross,
                maintain_order: MaintainOrderJoin::LeftRight,
                ..Default::default()
            },
        )
        .explode(cols([SAMPLE1, SAMPLE2]));
    // Correlation and format
    lazy_frame =
        lazy_frame
            .group_by_stable([col(LABEL1), col(LABEL2)])
            .agg([match key.correlation {
                Correlation::Pearson => pearson_corr(col(SAMPLE1), col(SAMPLE2)),
                Correlation::SpearmanRank => spearman_rank_corr(col(SAMPLE1), col(SAMPLE2), false),
            }
            .precision(key.precision, key.significant)
            .alias(CORRELATION)]);
    // Pivot
    lazy_frame = lazy_frame.pivot(
        by_name([LABEL2], true),
        Arc::new(labels),
        by_name([LABEL1], true),
        by_name([CORRELATION], true),
        element().item(true),
        true,
        PlSmallStr::EMPTY,
    );
    // Переименовываем `LABEL[1]` в `LABEL`
    Ok(lazy_frame.rename([LABEL1], [LABEL], true))
}
