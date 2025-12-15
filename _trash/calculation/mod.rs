use crate::{
    app::states::calculation::settings::{Normalize, Settings, Threshold},
    assets::CHRISTIE,
    r#const::{
        ENRICHMENT, FACTOR, FACTORS, MASK, MEAN, SAMPLE, SELECTIVITY, STANDARD, STANDARD_DEVIATION,
        THRESHOLD,
    },
    utils::{HashedDataFrame, HashedMetaDataFrame},
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;

const SN123: Expr = Expr::Selector(Selector::Matches(PlSmallStr::from_static(formatcp!(
    r#"^{STEREOSPECIFIC_NUMBERS123}\[\d+\]$"#
))));

const SN2: Expr = Expr::Selector(Selector::Matches(PlSmallStr::from_static(formatcp!(
    r#"^{STEREOSPECIFIC_NUMBERS2}\[\d+\]$"#
))));

/// Calculation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frames[0]
            .data
            .data_frame
            .clone()
            .lazy()
            .select(exprs(0));
        for index in 1..key.frames.len() {
            lazy_frame = lazy_frame.join(
                key.frames[index]
                    .data
                    .data_frame
                    .clone()
                    .lazy()
                    .select(exprs(index)),
                [col(LABEL), col(FATTY_ACID)],
                [col(LABEL), col(FATTY_ACID)],
                JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
            );
        }
        lazy_frame = compute(lazy_frame, key)?;
        HashedDataFrame::new(lazy_frame.collect()?)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation key
#[derive(Clone, Copy, Debug, Hash)]
pub(crate) struct Key<'a> {
    pub(crate) frames: &'a [HashedMetaDataFrame],
    pub(crate) index: Option<usize>,
    pub(crate) christie: bool,
    pub(crate) ddof: u8,
    pub(crate) normalize_factors: bool,
    pub(crate) normalize: Normalize,
    pub(crate) standard: Option<&'a str>,
    pub(crate) threshold: &'a Threshold,
    pub(crate) unsigned: bool,
    pub(crate) weighted: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frames: &'a [HashedMetaDataFrame], settings: &'a Settings) -> Self {
        Self {
            frames,
            index: settings.index,
            christie: settings.christie,
            ddof: settings.ddof,
            normalize_factors: settings.normalize_factors,
            normalize: settings.normalize,
            standard: settings.standard.as_deref(),
            threshold: &settings.threshold,
            unsigned: settings.unsigned,
            weighted: settings.weighted,
        }
    }
}

/// Calculation value
type Value = HashedDataFrame;

fn exprs(index: usize) -> [Expr; 3] {
    [
        col(LABEL),
        col(FATTY_ACID),
        all()
            .exclude_cols([
                PlSmallStr::from_static(LABEL),
                PlSmallStr::from_static(FATTY_ACID),
            ])
            .as_expr()
            .name()
            .suffix(&format!("[{index}]")),
    ]
}

fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Christie
    lazy_frame = christie(lazy_frame, key);
    // Standard
    lazy_frame = standard(lazy_frame, key);
    // Normalize
    // Обнуляет значения стандарта при пасчете долей.
    lazy_frame = lazy_frame.with_columns([
        normalize_experimental(SN123.nullify(col(STANDARD).not()), key),
        normalize_experimental(SN2.nullify(col(STANDARD).not()), key),
    ]);
    // Threshold
    lazy_frame = threshold(lazy_frame, key)?;
    // Calculate
    lazy_frame = lazy_frame.with_columns([
        sn13(SN123, SN2, key),
        enrichment_factor(SN2, SN123, key),
        selectivity_factor(SN2, SN123, key),
    ]);
    lazy_frame = mean_and_standard_deviations(lazy_frame, key)?;
    Ok(lazy_frame)
}

/// Standard
fn standard(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    // Стандарт - true, все остальные - false.
    // `lit(standard)` - без `lit()` будет искать столбец `standard`
    lazy_frame = lazy_frame.with_column(
        match key.standard {
            Some(standard) => col(LABEL).eq(lit(standard)),
            None => lit(false),
        }
        .alias(STANDARD),
    );
    // Standard[i]
    // Отношения площадей к площади стандарта.
    lazy_frame.with_column(
        (SN123 / SN123.filter(col(STANDARD)).first())
            .name()
            .replace(STEREOSPECIFIC_NUMBERS123, STANDARD, true),
    )
}

/// Threshold
fn threshold(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Стандарт - true, все остальные - автоматически или вручную.
    Ok(lazy_frame.with_column(
        if key.threshold.is_auto {
            col(STANDARD)
                .or(any_horizontal([SN123.gt_eq(key.threshold.auto.0)])?)
                .or(any_horizontal([SN2.gt_eq(key.threshold.auto.0)])?)
        } else {
            lit(Series::from_iter(&key.threshold.manual))
        }
        .alias(THRESHOLD),
    ))
}

/// Christie factors
fn christie(lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    if key.christie {
        lazy_frame
            .join(
                CHRISTIE.data.data_frame.clone().lazy(),
                [col(FATTY_ACID)],
                [col(FATTY_ACID)],
                JoinArgs::new(JoinType::Left).with_coalesce(JoinCoalesce::CoalesceColumns),
            )
            .with_columns([dtype_col(&DataType::Float64)
                .as_selector()
                .exclude_cols([FACTOR])
                .as_expr()
                * col(FACTOR).fill_null(lit(1.0))])
            .drop(cols([FACTOR]))
    } else {
        lazy_frame
    }
}

/// Normalize experimental data
fn normalize_experimental(mut expr: Expr, key: Key) -> Expr {
    if key.weighted {
        expr = expr * col(FATTY_ACID).fatty_acid().relative_atomic_mass(None);
    }
    expr.normalize(true)
}

/// 2 * DAG1(3) = 3 * TAG - MAG2 (стр. 116)
/// $x_{1:i} = x_{3:i} = x_{1:i | 3:i} / 2 = (3 * x_{1:i | 2:i | 3:i} - x_{2:i}) / 2$ (Sovová2008)
fn sn13(sn123: Expr, sn2: Expr, key: Key) -> Expr {
    let expr = ((sn123 * lit(3) - sn2) / lit(2)).clip_min_if(key.unsigned);
    (expr.clone() / expr.sum()).name().replace(
        STEREOSPECIFIC_NUMBERS123,
        STEREOSPECIFIC_NUMBERS13,
        true,
    )
}

fn enrichment_factor(sn2: Expr, sn123: Expr, key: Key) -> Expr {
    let mut enrichment_factor = FattyAcidExpr::enrichment_factor(sn2.clone(), sn123.clone());
    if key.normalize_factors {
        enrichment_factor = enrichment_factor / lit(3);
    }
    enrichment_factor.name().replace(
        STEREOSPECIFIC_NUMBERS2,
        formatcp!("{FACTORS}.{ENRICHMENT}"),
        true,
    )
}

fn selectivity_factor(sn2: Expr, sn123: Expr, key: Key) -> Expr {
    let mut selectivity_factor = col(FATTY_ACID).fatty_acid().selectivity_factor(sn2, sn123);
    if key.normalize_factors {
        selectivity_factor = selectivity_factor / lit(3);
    }
    selectivity_factor.name().replace(
        STEREOSPECIFIC_NUMBERS2,
        formatcp!("{FACTORS}.{SELECTIVITY}"),
        true,
    )
}

// Mean and standard deviation
fn mean_and_standard_deviations(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    Ok(match key.index {
        Some(index) => {
            let sn123 = col(format!("{STEREOSPECIFIC_NUMBERS123}[{index}]"));
            let sn2 = col(format!("{STEREOSPECIFIC_NUMBERS2}[{index}]"));
            let sn13 = col(format!("{STEREOSPECIFIC_NUMBERS13}[{index}]"));
            let enrichment_factor = col(format!("{FACTORS}.{ENRICHMENT}[{index}]"));
            let selectivity_factor = col(format!("{FACTORS}.{SELECTIVITY}[{index}]"));
            lazy_frame.select([
                col(LABEL),
                col(FATTY_ACID),
                mean_and_standard_deviation(sn123, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS123),
                mean_and_standard_deviation(sn2, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS2),
                mean_and_standard_deviation(sn13, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS13),
                as_struct(vec![
                    mean_and_standard_deviation(enrichment_factor, key.ddof)?.alias(ENRICHMENT),
                    mean_and_standard_deviation(selectivity_factor, key.ddof)?.alias(SELECTIVITY),
                ])
                .alias(FACTORS),
                as_struct(vec![
                    concat_arr(vec![col(format!("{STANDARD}[{index}]"))])?.alias(FACTOR),
                    col(STANDARD).alias(MASK),
                ])
                .alias(STANDARD),
                col(THRESHOLD),
            ])
        }
        None => {
            let sn13 = col(formatcp!(r#"^{STEREOSPECIFIC_NUMBERS13}\[\d+\]$"#));
            let enrichment_factors = col(formatcp!(r#"^{FACTORS}.{ENRICHMENT}\[\d+\]$"#));
            let selectivity_factors = col(formatcp!(r#"^{FACTORS}.{SELECTIVITY}\[\d+\]$"#));
            lazy_frame.select([
                col(LABEL),
                col(FATTY_ACID),
                mean_and_standard_deviation(SN123, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS123),
                mean_and_standard_deviation(SN2, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS2),
                mean_and_standard_deviation(sn13, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS13),
                as_struct(vec![
                    mean_and_standard_deviation(enrichment_factors, key.ddof)?.alias(ENRICHMENT),
                    mean_and_standard_deviation(selectivity_factors, key.ddof)?.alias(SELECTIVITY),
                ])
                .alias(FACTORS),
                as_struct(vec![
                    concat_arr(vec![col(formatcp!(r#"^{STANDARD}\[\d+\]$"#))])?.alias(FACTOR),
                    col(STANDARD).alias(MASK),
                ])
                .alias(STANDARD),
                col(THRESHOLD),
            ])
        }
    })
}

fn mean_and_standard_deviation(expr: Expr, ddof: u8) -> PolarsResult<Expr> {
    let sample = concat_arr(vec![expr])?;
    Ok(as_struct(vec![
        sample.clone().arr().mean().alias(MEAN),
        sample.clone().arr().std(ddof).alias(STANDARD_DEVIATION),
        sample.alias(SAMPLE),
    ]))
}

// /// Extension methods for [`Expr`]
// trait ExprExt {
//     fn mean_and_standard_deviation(self, ddof: u8) -> PolarsResult<Expr>;
// }

// impl ExprExt for Expr {
//     fn mean_and_standard_deviation(self, ddof: u8) -> PolarsResult<Expr> {
//         let sample = concat_arr(vec![self])?;
//         Ok(as_struct(vec![
//             sample.clone().arr().mean().alias(MEAN),
//             sample.clone().arr().std(ddof).alias(STANDARD_DEVIATION),
//             sample.alias(SAMPLE),
//         ]))
//     }
// }

// // n = m / M
// fn to_mole(names: [&str; 2]) -> Expr {
//     destruct(names) / col("FA").fa().mass()
// }

// // m = n * M
// fn to_mass(names: [&str; 2]) -> Expr {
//     destruct(names) * col("FA").fa().mass()
// }

// // Pchelkin fraction
// fn fraction(names: [&str; 2]) -> Expr {
//     // col(name) / (col(name) * col("FA").fa().mass() / lit(10)).sum()
//     destruct(names) / to_mass(names).sum()
// }

pub(crate) mod correlations;
pub(crate) mod sum;
pub(crate) mod table;

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test() -> PolarsResult<()> {
        let data_frame = df! {
            "A" => &[
                0208042.,
                0302117.,
                2420978.,
                0085359.,
                0195625.,
                2545783.,
                0031482.,
                4819586.,
                0012823.,
            ],
            "B" => &[
                0042194.,
                0145011.,
                0599666.,
                0025799.,
                0074037.,
                0595393.,
                0007738.,
                1158289.,
                0005070.,
            ],
            "M" => &[
                294.462,
                270.442,
                292.446,
                322.414,
                298.494,
                296.478,
                326.546,
                294.462,
                292.446,
            ],
        }?;
        let lazy_frame = data_frame.lazy().with_columns([
            (col("A") / (col("A") * col("M")).sum())
                .round(6, RoundMode::HalfToEven)
                .alias("_N___GLC_Peak_Area__Free_1,2-DAGs"),
            (col("B") / (col("B") * col("M")).sum())
                .round(6, RoundMode::HalfToEven)
                .alias("_N___GLC_Peak_Area__Total_TAGs"),
        ]);
        let data_frame = lazy_frame.collect()?;
        assert_eq!(
            data_frame["_N___GLC_Peak_Area__Free_1,2-DAGs"],
            Series::from_iter([
                0.000067, 0.000097, 0.000775, 0.000027, 0.000063, 0.000815, 0.000010, 0.001542,
                0.000004,
            ])
            .into_column(),
        );
        // [
        //     0.000067, 0.000097, 0.000775, 0.000027, 0.000063, 0.000815, 0.000010, 0.001542,
        //     0.000004,
        // ]
        Ok(())
    }
}
