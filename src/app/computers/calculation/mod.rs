use crate::{
    app::states::calculation::{Normalize, Settings},
    presets::CHRISTIE,
    utils::{HashedDataFrame, HashedMetaDataFrame},
};
use egui::{
    emath::OrderedFloat,
    util::cache::{ComputerMut, FrameCache},
};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::{ExprExt as _, ExprIfExt as _};

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
    pub(crate) row_filter: OrderedFloat<f64>,
    pub(crate) standard: Option<&'a str>,
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
            row_filter: OrderedFloat(settings.table.row_filter),
            standard: settings.standard.as_deref(),
            unsigned: settings.unsigned,
            weighted: settings.weighted,
        }
    }

    pub(crate) fn with_index(self, index: Option<usize>) -> Self {
        Self { index, ..self }
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
    if key.christie {
        lazy_frame = christie(lazy_frame);
    }
    let sn123 = col(format!(r#"^{STEREOSPECIFIC_NUMBERS123}\[\d+\]$"#));
    let sn2 = col(format!(r#"^{STEREOSPECIFIC_NUMBERS2}\[\d+\]$"#));
    // Standard
    if let Some(standard) = key.standard {
        // lazy_frame = lazy_frame.filter(col(LABEL).neq(lit(standard)));
        // lazy_frame = lazy_frame.with_column(col(LABEL).neq(lit(standard)).alias("Filter"));
        lazy_frame = lazy_frame.with_column(
            ternary_expr(col(LABEL).neq(lit(standard)), lit(true), lit(NULL)).alias("Filter"),
        );
        println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());
    }
    // Normalize
    lazy_frame = lazy_frame.with_columns([
        ternary_expr(
            col("Filter"),
            experimental(sn123.clone().nullify(col("Filter")), key),
            experimental(sn123.clone(), key),
        ),
        ternary_expr(
            col("Filter"),
            experimental(sn2.clone().nullify(col("Filter")), key),
            experimental(sn2.clone(), key),
        ),
    ]);
    // Filter
    // lazy_frame = lazy_frame.filter(any_horizontal([sn123.clone().gt_eq(key.row_filter.0)])?.or(
    //     any_horizontal([sn2.clone().fill_nan(lit(0)).gt_eq(key.row_filter.0)])?,
    // ));
    lazy_frame = lazy_frame.with_column(col("Filter").and(
        any_horizontal([sn123.clone().gt_eq(key.row_filter.0)])?.or(any_horizontal([
            sn2.clone().fill_nan(lit(0)).gt_eq(key.row_filter.0),
        ])?),
    ));
    println!("lazy_frame 1: {}", lazy_frame.clone().collect().unwrap());
    // Calculate
    lazy_frame = lazy_frame.with_columns([
        sn13(sn123.clone(), sn2.clone(), key),
        enrichment_factors(sn2.clone(), sn123.clone(), key),
        selectivity_factors(sn2.clone(), sn123.clone(), key),
    ]);
    // Mean and standard deviation
    match key.index {
        Some(index) => {
            let sn123 = col(format!("{STEREOSPECIFIC_NUMBERS123}[{index}]"));
            let sn2 = col(format!("{STEREOSPECIFIC_NUMBERS2}[{index}]"));
            let sn13 = col(format!("{STEREOSPECIFIC_NUMBERS13}[{index}]"));
            let enrichment_factor = col(format!("EnrichmentFactor[{index}]"));
            let selectivity_factor = col(format!("SelectivityFactor[{index}]"));
            lazy_frame = lazy_frame.select([
                col(LABEL),
                col(FATTY_ACID),
                mean_and_standard_deviations(sn123, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS123),
                mean_and_standard_deviations(sn2, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS2),
                mean_and_standard_deviations(sn13, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS13),
                as_struct(vec![
                    mean_and_standard_deviations(enrichment_factor, key.ddof)?.alias("Enrichment"),
                    mean_and_standard_deviations(selectivity_factor, key.ddof)?
                        .alias("Selectivity"),
                ])
                .alias("Factors"),
                col("Filter"),
            ]);
        }
        None => {
            let sn13 = col(format!(r#"^{STEREOSPECIFIC_NUMBERS13}\[\d+\]$"#));
            let enrichment_factors = col(format!(r#"^EnrichmentFactor\[\d+\]$"#));
            let selectivity_factors = col(format!(r#"^SelectivityFactor\[\d+\]$"#));
            lazy_frame = lazy_frame.select([
                col(LABEL),
                col(FATTY_ACID),
                mean_and_standard_deviations(sn123, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS123),
                mean_and_standard_deviations(sn2, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS2),
                mean_and_standard_deviations(sn13, key.ddof)?.alias(STEREOSPECIFIC_NUMBERS13),
                as_struct(vec![
                    mean_and_standard_deviations(enrichment_factors, key.ddof)?.alias("Enrichment"),
                    mean_and_standard_deviations(selectivity_factors, key.ddof)?
                        .alias("Selectivity"),
                ])
                .alias("Factors"),
                col("Filter"),
            ]);
        }
    }
    Ok(lazy_frame)
}

// Christie factors
fn christie(mut lazy_frame: LazyFrame) -> LazyFrame {
    lazy_frame = lazy_frame
        .join(
            CHRISTIE.data.data_frame.clone().lazy(),
            [col(FATTY_ACID)],
            [col(FATTY_ACID)],
            JoinArgs::new(JoinType::Left).with_coalesce(JoinCoalesce::CoalesceColumns),
        )
        .with_columns([dtype_col(&DataType::Float64)
            .as_selector()
            .exclude_cols(["Factor"])
            .as_expr()
            * col("Factor").fill_null(lit(1.0))])
        .drop(cols(["Factor"]));
    lazy_frame
}

fn experimental(mut expr: Expr, key: Key) -> Expr {
    if key.weighted {
        expr = expr * col(FATTY_ACID).fatty_acid().relative_atomic_mass(None);
    }
    expr.clone() / expr.sum()
}

/// 2 * DAG1(3) = 3 * TAG - MAG2 (стр. 116)
/// $x_{1:i} = x_{3:i} = x_{1:i | 3:i} / 2 = (3 * x_{1:i | 2:i | 3:i} - x_{2:i}) / 2$ (Sovová2008)
fn sn13(sn123: Expr, sn2: Expr, key: Key) -> Expr {
    ((sn123 * lit(3) - sn2) / lit(2))
        .clip_min_if(key.unsigned)
        .normalize()
        .name()
        .replace(STEREOSPECIFIC_NUMBERS123, STEREOSPECIFIC_NUMBERS13, true)
}

fn enrichment_factors(sn2: Expr, sn123: Expr, key: Key) -> Expr {
    let mut enrichment_factor = FattyAcidExpr::enrichment_factor(sn2.clone(), sn123.clone());
    if key.normalize_factors {
        enrichment_factor = enrichment_factor / lit(3);
    }
    enrichment_factor
        .name()
        .replace(STEREOSPECIFIC_NUMBERS2, "EnrichmentFactor", true)
}

fn selectivity_factors(sn2: Expr, sn123: Expr, key: Key) -> Expr {
    let mut selectivity_factor = col(FATTY_ACID).fatty_acid().selectivity_factor(sn2, sn123);
    if key.normalize_factors {
        selectivity_factor = selectivity_factor / lit(3);
    }
    selectivity_factor
        .name()
        .replace(STEREOSPECIFIC_NUMBERS2, "SelectivityFactor", true)
}

fn mean_and_standard_deviations(expr: Expr, ddof: u8) -> PolarsResult<Expr> {
    let array = concat_arr(vec![expr])?;
    Ok(as_struct(vec![
        array.clone().arr().mean().alias("Mean"),
        array.clone().arr().std(ddof).alias("StandardDeviation"),
        array.alias("Array"),
    ]))
}

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
pub(crate) mod display;
pub(crate) mod indices;

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
