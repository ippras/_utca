use crate::{
    app::{panes::calculation::parameters::Parameters, presets::CHRISTIE},
    utils::{HashedDataFrame, HashedMetaDataFrame, hash_expr},
};
use egui::util::cache::{ComputerMut, FrameCache};
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
        let mut lazy_frame = match key.parameters.index {
            Some(index) => {
                let frame = &key.frames[index];
                // let mut lazy_frame = frame.data.data_frame.clone().lazy();
                compute(&frame.data.data_frame, key.parameters)?.select([
                    col(LABEL),
                    col(FATTY_ACID),
                    as_struct(vec![all().exclude_cols([LABEL, FATTY_ACID]).as_expr()])
                        .alias(frame.meta.format(".").to_string()),
                ])
            }
            None => {
                let compute = |frame: &HashedMetaDataFrame| -> PolarsResult<LazyFrame> {
                    Ok(compute(&frame.data.data_frame, key.parameters)?.select([
                        hash_expr(as_struct(vec![col(LABEL), col(FATTY_ACID)])),
                        col(LABEL),
                        col(FATTY_ACID),
                        as_struct(vec![all().exclude_cols([LABEL, FATTY_ACID]).as_expr()])
                            .alias(frame.meta.format(".").to_string()),
                    ]))
                };
                let mut lazy_frame = compute(&key.frames[0])?;
                for frame in &key.frames[1..] {
                    lazy_frame = lazy_frame.join(
                        compute(frame)?,
                        [col("Hash"), col(LABEL), col(FATTY_ACID)],
                        [col("Hash"), col(LABEL), col(FATTY_ACID)],
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                    );
                }
                lazy_frame = lazy_frame.drop(by_name(["Hash"], true));
                lazy_frame
            }
        };
        lazy_frame = mean_and_standard_deviations(lazy_frame, key.parameters.ddof)?;
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
    pub(crate) frames: &'a Vec<HashedMetaDataFrame>,
    pub(crate) parameters: &'a Parameters,
}

/// Calculation value
type Value = HashedDataFrame;

fn compute(data_frame: &DataFrame, parameters: &Parameters) -> PolarsResult<LazyFrame> {
    let mut lazy_frame = data_frame.clone().lazy();
    // Christie
    if parameters.christie {
        lazy_frame = christie(lazy_frame);
    }
    println!("compute 0: {}", lazy_frame.clone().collect().unwrap());
    let sn123 = experimental(STEREOSPECIFIC_NUMBERS123, parameters);
    // let sn2 = experimental(STEREOSPECIFIC_NUMBERS2, parameters);
    let sn2 = match data_frame[3].name().as_str() {
        STEREOSPECIFIC_NUMBERS2 => experimental(STEREOSPECIFIC_NUMBERS2, parameters),
        STEREOSPECIFIC_NUMBERS12_23 => {
            let sn_1223 = experimental(STEREOSPECIFIC_NUMBERS12_23, parameters);
            sn1223(sn123.clone(), sn_1223, parameters)
        }
        _ => lit(NULL),
    };
    let sn13 = sn13(sn123.clone(), sn2.clone(), parameters);
    // Stereospecific numbers
    lazy_frame = lazy_frame.with_columns([sn123, sn2, sn13]);
    // Factors
    lazy_frame = lazy_frame.with_column(factors());
    Ok(lazy_frame)
}

fn experimental(name: &str, parameters: &Parameters) -> Expr {
    let mut expr = col(name);
    if parameters.weighted {
        expr = col(name) / (col(name) * col(FATTY_ACID).fatty_acid().mass(None)).sum()
    };
    expr.normalize_if(parameters.normalize.experimental)
}

/// MAG2 = 4 * DAG1223 - 3 * TAG
fn sn1223(sn123: Expr, sn1223: Expr, parameters: &Parameters) -> Expr {
    (lit(4) * sn1223 - lit(3) * sn123)
        .clip_min_if(parameters.unsigned)
        .normalize_if(parameters.normalize.theoretical)
        .alias(STEREOSPECIFIC_NUMBERS2)
}

/// 2 * DAG1(3) = 3 * TAG - MAG2 (стр. 116)
/// $x_{1 => i} = x_{3 => i} = x_{1|3 => i} / 2 = (3 * x_{1|2|3 => i} - x_{2 => i}) / 2$ (Sovová2008)
fn sn13(sn123: Expr, sn2: Expr, parameters: &Parameters) -> Expr {
    // // DAG1(3) = 3 * TAG - 2 * DAG1223
    // let sn12_23 = (lit(3) * sn123.clone() - lit(2) * sn12_23)
    //     .clip_min_if(parameters.unsigned)
    //     .normalize_if(parameters.normalize.theoretical)
    //     .alias(STEREOSPECIFIC_NUMBERS13);
    // DAG1(3) = (3 * TAG - MAG2) / 2
    // $x_{[i,,]} = x_{[,,i]} = (3 * x_{(i)} - x_{[,i,]}) / 2$
    ((lit(3) * sn123 - sn2) / lit(2))
        .clip_min_if(parameters.unsigned)
        .normalize_if(parameters.normalize.theoretical)
        .alias(STEREOSPECIFIC_NUMBERS13)
    // as_struct(vec![sn12_23, sn2]).alias(STEREOSPECIFIC_NUMBERS13)
}

fn factors() -> Expr {
    as_struct(vec![
        FattyAcidExpr::enrichment_factor(
            col(STEREOSPECIFIC_NUMBERS2),
            col(STEREOSPECIFIC_NUMBERS123),
        )
        .alias("Enrichment"),
        col(FATTY_ACID)
            .fatty_acid()
            .selectivity_factor(col(STEREOSPECIFIC_NUMBERS2), col(STEREOSPECIFIC_NUMBERS123))
            .alias("Selectivity"),
    ])
    .alias("Factors")
}

fn christie(lazy_frame: LazyFrame) -> LazyFrame {
    lazy_frame
        .with_column(hash_expr(col(FATTY_ACID)))
        .join(
            CHRISTIE.data.clone().lazy().select([
                hash_expr(col(FATTY_ACID)),
                col(FATTY_ACID),
                col("Christie"),
            ]),
            [col("Hash"), col(FATTY_ACID)],
            [col("Hash"), col(FATTY_ACID)],
            JoinArgs::new(JoinType::Left),
        )
        // col("Christie").fill_null(lit(1)),
        .drop(by_name(["Hash"], true))
}

// fn mean_and_standard_deviations(lazy_frame: LazyFrame, ddof: u8) -> PolarsResult<LazyFrame> {
//     Ok(lazy_frame.select([
//         col(LABEL),
//         col(FATTY_ACID),
//         as_struct(vec![
//             mean(&["Experimental", STEREOSPECIFIC_NUMBERS123], ddof)?,
//             mean(&["Experimental", STEREOSPECIFIC_NUMBERS12_23], ddof)?,
//             mean(&["Experimental", STEREOSPECIFIC_NUMBERS2], ddof)?,
//         ])
//         .alias("Experimental"),
//         as_struct(vec![
//             mean(&["Theoretical", STEREOSPECIFIC_NUMBERS123], ddof)?,
//             mean(&["Theoretical", STEREOSPECIFIC_NUMBERS12_23], ddof)?,
//             mean(&["Theoretical", STEREOSPECIFIC_NUMBERS2], ddof)?,
//             as_struct(vec![
//                 mean(
//                     &[
//                         "Theoretical",
//                         STEREOSPECIFIC_NUMBERS13,
//                         STEREOSPECIFIC_NUMBERS12_23,
//                     ],
//                     ddof,
//                 )?,
//                 mean(
//                     &[
//                         "Theoretical",
//                         STEREOSPECIFIC_NUMBERS13,
//                         STEREOSPECIFIC_NUMBERS2,
//                     ],
//                     ddof,
//                 )?,
//             ])
//             .alias(STEREOSPECIFIC_NUMBERS13),
//         ])
//         .alias("Theoretical"),
//         as_struct(vec![
//             mean(&["Factors", "Enrichment"], ddof)?,
//             mean(&["Factors", "Selectivity"], ddof)?,
//         ])
//         .alias("Factors"),
//     ]))
// }
fn mean_and_standard_deviations(lazy_frame: LazyFrame, ddof: u8) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame.select([
        col(LABEL),
        col(FATTY_ACID),
        mean(&[STEREOSPECIFIC_NUMBERS123], ddof)?,
        mean(&[STEREOSPECIFIC_NUMBERS2], ddof)?,
        mean(&[STEREOSPECIFIC_NUMBERS13], ddof)?,
        as_struct(vec![
            mean(&["Factors", "Enrichment"], ddof)?,
            mean(&["Factors", "Selectivity"], ddof)?,
        ])
        .alias("Factors"),
    ]))
}

fn mean(names: &[&str], ddof: u8) -> PolarsResult<Expr> {
    let array = || {
        concat_arr(vec![
            all()
                .exclude_cols([LABEL, FATTY_ACID])
                .as_expr()
                .destruct(names),
        ])
    };
    Ok(as_struct(vec![
        array()?.arr().mean().alias("Mean"),
        array()?.arr().std(ddof).alias("StandardDeviation"),
        array()?.alias("Array"),
    ])
    .alias(names[names.len() - 1]))
}

fn compute_sn(
    fatty_acid: Expr,
    sn123: Expr,
    sn12_23: Expr,
    sn2: Expr,
    parameters: &Parameters,
) -> [Expr; 4] {
    let experimental = |mut sn: Expr| {
        if parameters.weighted {
            sn = sn.clone() / (sn * fatty_acid.clone().fatty_acid().mass(None)).sum()
        };
        sn.normalize_if(parameters.normalize.experimental)
    };
    let sn123 = experimental(sn123);
    let sn12_23 = experimental(sn12_23);
    let sn2 = experimental(sn2);
    [
        compute_sn123(sn123.clone(), sn12_23.clone(), sn2.clone(), parameters),
        compute_sn12_23(sn123.clone(), sn12_23.clone(), sn2.clone(), parameters),
        compute_sn2(sn123.clone(), sn12_23.clone(), sn2.clone(), parameters),
        compute_sn13(sn123, sn12_23, sn2, parameters),
    ]
}

/// 3 * x{1,2,3: i} = 2 * x{1,3: i} + x{2: i}
/// 3 * x{1,2,3: i} = 4 * x{1,2: i; 2,3: i} - x{2: i}
/// 3 * TAG = 2 * DAG1(3) + MAG2
/// 3 * TAG = 4 * DAG1223 - MAG2
fn compute_sn123(sn123: Expr, sn12_23: Expr, sn2: Expr, parameters: &Parameters) -> Expr {
    let theoretical = ((lit(4) * sn12_23 - sn2) / lit(3))
        .clip_min_if(parameters.unsigned)
        .normalize_if(parameters.normalize.theoretical);
    as_struct(vec![
        sn123.alias("Experimental"),
        theoretical.alias("Theoretical"),
    ])
    .alias(STEREOSPECIFIC_NUMBERS123)
}

/// DAG1223 = (3 * TAG + MAG2) / 4
fn compute_sn12_23(sn123: Expr, sn12_23: Expr, sn2: Expr, parameters: &Parameters) -> Expr {
    let theoretical =
        ((lit(3) * sn123 + sn2) / lit(4)).normalize_if(parameters.normalize.theoretical);
    as_struct(vec![
        sn12_23.alias("Experimental"),
        theoretical.alias("Theoretical"),
    ])
    .alias(STEREOSPECIFIC_NUMBERS12_23)
}

/// MAG2 = 4 * DAG1223 - 3 * TAG
fn compute_sn2(sn123: Expr, sn12_23: Expr, sn2: Expr, parameters: &Parameters) -> Expr {
    let theoretical = (lit(4) * sn12_23 - lit(3) * sn123)
        .clip_min_if(parameters.unsigned)
        .normalize_if(parameters.normalize.theoretical);
    as_struct(vec![
        sn2.alias("Experimental"),
        theoretical.alias("Theoretical"),
    ])
    .alias(STEREOSPECIFIC_NUMBERS2)
}

// xi мольная доля i-й ЖК в ТГ
// x[i,,] мольная доля i-й ЖК в sn-1 положении ТГ
// x[,i,] мольная доля i-й ЖК в sn-2 положении ТГ
// x[,,i] мольная доля i-й ЖК в sn-3 положении ТГ
// x[i,j,k] мольная доля стереоизомера ТГ, содержащего ацил i-й ЖК в положении sn-1, ацил j-й ЖК в положении sn-2 и ацил k-й ЖК в положении sn-3

/// 2 * DAG1(3) = 3 * TAG - MAG2 (стр. 116)
/// $x_{[i,,]} = x_{[,,i]} = (3 * x_{(i)} - x_{[,i,]}) / 2$ (Sovová2008)
fn compute_sn13(sn123: Expr, sn12_23: Expr, sn2: Expr, parameters: &Parameters) -> Expr {
    // DAG1(3) = 3 * TAG - 2 * DAG1223
    let sn12_23 = (lit(3) * sn123.clone() - lit(2) * sn12_23)
        .clip_min_if(parameters.unsigned)
        .normalize_if(parameters.normalize.theoretical)
        .alias(STEREOSPECIFIC_NUMBERS12_23);
    // DAG1(3) = (3 * TAG - MAG2) / 2
    // $x_{[i,,]} = x_{[,,i]} = (3 * x_{(i)} - x_{[,i,]}) / 2$
    let sn2 = ((lit(3) * sn123 - sn2) / lit(2))
        .clip_min_if(parameters.unsigned)
        .normalize_if(parameters.normalize.theoretical)
        .alias(STEREOSPECIFIC_NUMBERS2);
    as_struct(vec![sn12_23, sn2]).alias(STEREOSPECIFIC_NUMBERS13)
}

fn _factors(fatty_acid: Expr, sn123: Expr, sn2: Expr) -> Expr {
    let sn123 = sn123
        .clone()
        .struct_()
        .field_by_index(0)
        .fill_nan(sn123.struct_().field_by_index(1));
    let sn2 = sn2
        .clone()
        .struct_()
        .field_by_index(0)
        .fill_nan(sn2.struct_().field_by_index(1));
    as_struct(vec![
        FattyAcidExpr::enrichment_factor(sn2.clone(), sn123.clone()).alias("Enrichment"),
        fatty_acid
            .fatty_acid()
            .selectivity_factor(sn2, sn123)
            .alias("Selectivity"),
    ])
    .alias("Factors")
}

// fn experimental(
//     fatty_acid: Expr,
//     sn123: Expr,
//     sn12_23: Expr,
//     sn2: Expr,
//     parameters: &Parameters,
// ) -> Expr {
//     // // col(name) / (col(name) * col("FA").fa().mass() / lit(10)).sum()
//     let compute = |mut expr: Expr| {
//         // S / ∑(S * M)
//         if parameters.weighted {
//             expr = expr.clone() / (expr * fatty_acid.clone().fatty_acid().mass(None)).sum()
//         };
//         expr.normalize_if(parameters.normalize.experimental)
//     };
//     as_struct(vec![compute(sn123), compute(sn12_23), compute(sn2)]).alias("Experimental")
// }

// fn theoretical(sn123: Expr, sn12_23: Expr, sn2: Expr, parameters: &Parameters) -> Expr {
//     // 3 * TAG =  2 * DAG13 + MAG2
//     let tag = || (lit(4) * sn12_23.clone() - sn2.clone()) / lit(3);
//     // DAG1223 = (3 * TAG + MAG2) / 4
//     let dag1223 = || (lit(3) * sn123.clone() + sn2.clone()) / lit(4);
//     // MAG2 = 4 * DAG1223 - 3 * TAG
//     let mag2 = || lit(4) * sn12_23.clone() - lit(3) * sn123.clone();
//     // 2 * DAG13 = 3 * TAG - MAG2 (стр. 116)
//     let sn13 = || {
//         // DAG13 = (3 * TAG - MAG2) / 2
//         let sn2 = || (lit(3) * sn123.clone() - sn2.clone()) / lit(2);
//         // DAG13 = 3 * TAG - 2 * DAG1223
//         let sn12_23 = || lit(3) * sn123.clone() - lit(2) * sn12_23.clone();
//         let sn12_23 = sn12_23()
//             .clip_min_if(parameters.unsigned)
//             .normalize_if(parameters.normalize.theoretical)
//             .alias(STEREOSPECIFIC_NUMBERS12_23);
//         let sn2 = sn2()
//             .clip_min_if(parameters.unsigned)
//             .normalize_if(parameters.normalize.theoretical)
//             .alias(STEREOSPECIFIC_NUMBERS2);
//         match parameters.from {
//             From::Sn12_23 => as_struct(vec![sn12_23, sn2]),
//             From::Sn2 => as_struct(vec![sn2, sn12_23]),
//         }
//     };
//     as_struct(vec![
//         tag()
//             .clip_min_if(parameters.unsigned)
//             .normalize_if(parameters.normalize.theoretical)
//             .alias(STEREOSPECIFIC_NUMBERS123),
//         dag1223()
//             .normalize_if(parameters.normalize.theoretical)
//             .alias(STEREOSPECIFIC_NUMBERS12_23),
//         mag2()
//             .clip_min_if(parameters.unsigned)
//             .normalize_if(parameters.normalize.theoretical)
//             .alias(STEREOSPECIFIC_NUMBERS2),
//         sn13().alias(STEREOSPECIFIC_NUMBERS13),
//     ])
//     .alias("Theoretical")
// }

// fn old_factors(fatty_acid: Expr, sn123: Expr, sn2: Expr) -> Expr {
//     as_struct(vec![
//         FattyAcidExpr::enrichment_factor(sn2.clone(), sn123.clone()).alias("Enrichment"),
//         fatty_acid
//             .fatty_acid()
//             .selectivity_factor(sn2, sn123)
//             .alias("Selectivity"),
//     ])
//     .alias("Factors")
// }

// fn single(mut lazy_frame: LazyFrame, column: &str, key: Key) -> PolarsResult<LazyFrame> {
//     let fraction = match key.settings.fraction {
//         Fraction::AsIs => as_is,
//         Fraction::ToMole => to_mole,
//         Fraction::ToMass => to_mass,
//         Fraction::Fraction => fraction,
//     };
//     lazy_frame = lazy_frame.with_column(
//         as_struct(vec![
//             fraction([column, "TAG"])
//                 .fill_null(lit(0.0))
//                 .christie(key.settings.christie.apply)
//                 .normalize_if(key.settings.normalize.experimental),
//             fraction([column, "DAG1223"])
//                 .fill_null(lit(0.0))
//                 .christie(key.settings.christie.apply)
//                 .normalize_if(key.settings.normalize.experimental),
//             fraction([column, "MAG2"])
//                 .fill_null(lit(0.0))
//                 .christie(key.settings.christie.apply)
//                 .normalize_if(key.settings.normalize.experimental),
//         ])
//         .alias("Experimental"),
//     );
//     // Theoretical
//     // lazy_frame = lazy_frame.with_column(
//     //     as_struct(vec![
//     //         col("Experimental")
//     //             .experimental()
//     //             .tag123(key.settings)
//     //             .alias("TAG"),
//     //         col("Experimental")
//     //             .experimental()
//     //             .dag1223(key.settings)
//     //             .alias("DAG1223"),
//     //         col("Experimental")
//     //             .experimental()
//     //             .mag2(key.settings)
//     //             .alias("MAG2"),
//     //         as_struct(vec![
//     //             col("Experimental")
//     //                 .experimental()
//     //                 .dag13_from_dag1223(key.settings)
//     //                 .alias("DAG1223"),
//     //             col("Experimental")
//     //                 .experimental()
//     //                 .dag13_from_mag2(key.settings)
//     //                 .alias("MAG2"),
//     //         ])
//     //         .alias("DAG13"),
//     //     ])
//     //     .alias("Theoretical"),
//     // );
//     // Calculated
//     // lazy_frame = lazy_frame.with_column(
//     //     as_struct(vec![
//     //         col("Experimental")
//     //             .struct_()
//     //             .field_by_names(["TAG", "DAG1223", "MAG2"]),
//     //         col("Theoretical")
//     //             .struct_()
//     //             .field_by_name("DAG13")
//     //             .struct_()
//     //             .field_by_name(match key.settings.from {
//     //                 From::Dag1223 => "DAG1223",
//     //                 From::Mag2 => "MAG2",
//     //             })
//     //             .alias("DAG13"),
//     //     ])
//     //     .alias("Calculated"),
//     // );
//     // Enrichment factor
//     lazy_frame = lazy_frame.with_column(
//         as_struct(vec![
//             col("Calculated").calculated().ef("MAG2").alias("MAG2"),
//             col("Calculated").calculated().ef("DAG13").alias("DAG13"),
//         ])
//         .alias("EF"),
//     );
//     // Selectivity factor
//     lazy_frame = lazy_frame.with_column(
//         as_struct(vec![
//             col("Calculated").calculated().sf("MAG2").alias("MAG2"),
//             col("Calculated").calculated().sf("DAG13").alias("DAG13"),
//         ])
//         .alias("SF"),
//     );
//     println!("lazy_frame 8: {}", lazy_frame.clone().collect().unwrap());
//     lazy_frame = lazy_frame.with_column(
//         as_struct(vec![
//             col("Experimental"),
//             col("Theoretical"),
//             col("Calculated"),
//             as_struct(vec![col("EF"), col("SF")]).alias("Factors"),
//         ])
//         .alias(column),
//     );
//     println!("lazy_frame 9: {}", lazy_frame.clone().collect().unwrap());
//     Ok(lazy_frame)
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

pub(super) mod display;
pub(super) mod indices;

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
