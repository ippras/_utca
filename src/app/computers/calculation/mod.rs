use crate::{
    app::{
        panes::calculation::settings::{Fraction, From, Settings},
        presets::CHRISTIE,
    },
    utils::Hashed,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_ext::expr::{ExprExt as _, ExprIfExt as _};
use std::hash::{Hash, Hasher};

/// Calculation computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<DataFrame> {
        match key.settings.index {
            Some(index) => {
                let frame = &key.frames[index];
                let mut lazy_frame = frame.data.clone().lazy();
                lazy_frame = compute(lazy_frame, key.settings)?;
                lazy_frame.collect()
            }
            None => {
                let compute = |frame: &MetaDataFrame| -> PolarsResult<LazyFrame> {
                    Ok(compute(frame.data.clone().lazy(), key.settings)?.select([
                        as_struct(vec![col("Label"), col("FattyAcid")]).hash(),
                        col("Label"),
                        col("FattyAcid"),
                        as_struct(vec![
                            col("Experimental"),
                            col("Theoretical"),
                            col("Calculated"),
                            col("Factors"),
                        ])
                        .alias(frame.meta.format(".").to_string()),
                    ]))
                };
                let mut lazy_frame = compute(&key.frames[0])?;
                for frame in &key.frames[1..] {
                    lazy_frame = lazy_frame.join(
                        compute(frame)?,
                        [col("Hash"), col("Label"), col("FattyAcid")],
                        [col("Hash"), col("Label"), col("FattyAcid")],
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                    );
                }
                lazy_frame = lazy_frame.drop(by_name(["Hash"], true));
                lazy_frame = means(lazy_frame, key.settings)?;
                // Index
                lazy_frame = lazy_frame.with_row_index("Index", None);
                lazy_frame.collect()
            }
        }
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) frames: &'a Hashed<Vec<MetaDataFrame>>,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frames.hash(state);
        self.settings.index.hash(state);
        // self.settings.percent.hash(state);
        // self.settings.precision.hash(state);
        // self.settings.resizable.hash(state);
        // self.settings.sticky_columns.hash(state);
        // self.settings.truncate.hash(state);
        self.settings.fraction.hash(state);
        self.settings.from.hash(state);
        self.settings.normalize.hash(state);
        self.settings.unsigned.hash(state);
        self.settings.christie.hash(state);
        self.settings.factors.hash(state);
        self.settings.ddof.hash(state);
    }
}

/// Calculation value
type Value = DataFrame;

fn compute(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    // Christie
    if settings.christie {
        lazy_frame = christie(lazy_frame);
    }
    // Experimental
    lazy_frame = lazy_frame.with_column(
        ExperimentalExpr(as_struct(vec![
            col("Triacylglycerol"),
            col("Diacylglycerol1223"),
            col("Monoacylglycerol2"),
        ]))
        .compute(col("FattyAcid"), settings),
    );
    // Theoretical
    lazy_frame = lazy_frame.with_column(TheoreticalExpr(col("Experimental")).compute(settings));
    // Calculated
    lazy_frame = lazy_frame.with_column(
        as_struct(vec![
            col("Experimental").struct_().field_by_name("*"),
            col("Theoretical")
                .struct_()
                .field_by_name("Diacylglycerol13")
                .struct_()
                .field_by_name(match settings.from {
                    From::Dag1223 => "Diacylglycerol1223",
                    From::Mag2 => "Monoacylglycerol2",
                })
                .alias("Diacylglycerol13"),
        ])
        .alias("Calculated"),
    );
    // Factors
    lazy_frame = lazy_frame.with_column(FactorsExpr(col("Calculated")).compute(col("FattyAcid")));
    Ok(lazy_frame.select(vec![
        col("Label"),
        col("FattyAcid"),
        col("Experimental"),
        col("Theoretical"),
        col("Calculated"),
        col("Factors"),
    ]))
}

fn christie(lazy_frame: LazyFrame) -> LazyFrame {
    lazy_frame
        .with_column(col("FattyAcid").hash())
        .join(
            CHRISTIE.data.clone().lazy().select([
                col("FattyAcid").hash(),
                col("FattyAcid"),
                col("Christie"),
            ]),
            [col("Hash"), col("FattyAcid")],
            [col("Hash"), col("FattyAcid")],
            JoinArgs::new(JoinType::Left),
        )
        // col("Christie").fill_null(lit(1)),
        .drop(by_name(["Hash"], true))
}

fn means(lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame.select([
        col("Label"),
        col("FattyAcid"),
        as_struct(vec![
            mean(&["Experimental", "Triacylglycerol"], settings.ddof)?,
            mean(&["Experimental", "Diacylglycerol1223"], settings.ddof)?,
            mean(&["Experimental", "Monoacylglycerol2"], settings.ddof)?,
        ])
        .alias("Experimental"),
        as_struct(vec![
            mean(&["Theoretical", "Triacylglycerol"], settings.ddof)?,
            mean(&["Theoretical", "Diacylglycerol1223"], settings.ddof)?,
            mean(&["Theoretical", "Monoacylglycerol2"], settings.ddof)?,
            as_struct(vec![
                mean(
                    &["Theoretical", "Diacylglycerol13", "Diacylglycerol1223"],
                    settings.ddof,
                )?,
                mean(
                    &["Theoretical", "Diacylglycerol13", "Monoacylglycerol2"],
                    settings.ddof,
                )?,
            ])
            .alias("Diacylglycerol13"),
        ])
        .alias("Theoretical"),
        as_struct(vec![
            mean(&["Factors", "Enrichment"], settings.ddof)?,
            mean(&["Factors", "Selectivity"], settings.ddof)?,
        ])
        .alias("Factors"),
    ]))
}

fn mean(names: &[&str], ddof: u8) -> PolarsResult<Expr> {
    let array = || {
        concat_arr(vec![
            all()
                .exclude_cols(["Label", "FattyAcid"])
                .as_expr()
                .destruct(names),
        ])
    };
    Ok(as_struct(vec![
        array()?.arr().mean().alias("Mean"),
        array()?.arr().std(ddof).alias("StandardDeviation"),
        array()?.alias("Values"),
    ])
    .alias(names[names.len() - 1]))
}

trait Tag {
    fn tag(self) -> Expr;
}

trait Dag1223 {
    fn dag1223(self) -> Expr;
}

trait Mag2 {
    fn mag2(self) -> Expr;
}

/// Experimental
#[derive(Clone, Debug)]
struct ExperimentalExpr(Expr);

// TODO
// if Fraction then: UserWarning: groups may be out of bounds; more groups than elements in a series is only possible in dynamic group_by
impl ExperimentalExpr {
    fn compute(self, fatty_acid: Expr, settings: &Settings) -> Expr {
        // // col(name) / (col(name) * col("FA").fa().mass() / lit(10)).sum()
        let experimental = |mut expr: Expr| {
            // S / ∑(S * M)
            if let Fraction::Fraction = settings.fraction {
                expr = expr.clone() / (expr * fatty_acid.clone().fatty_acid().mass(None)).sum()
            };
            expr.normalize_if(settings.normalize.experimental)
        };
        as_struct(vec![
            experimental(self.clone().tag()),
            experimental(self.clone().dag1223()),
            experimental(self.clone().mag2()),
        ])
        .alias("Experimental")
    }
}

impl Tag for ExperimentalExpr {
    fn tag(self) -> Expr {
        self.0.struct_().field_by_name("Triacylglycerol")
    }
}

impl Dag1223 for ExperimentalExpr {
    fn dag1223(self) -> Expr {
        self.0.struct_().field_by_name("Diacylglycerol1223")
    }
}

impl Mag2 for ExperimentalExpr {
    fn mag2(self) -> Expr {
        self.0.struct_().field_by_name("Monoacylglycerol2")
    }
}

/// Theoretical
#[derive(Clone, Debug)]
struct TheoreticalExpr(Expr);

impl TheoreticalExpr {
    fn compute(self, settings: &Settings) -> Expr {
        // 3 * TAG =  2 * DAG13 + MAG2
        let tag = || (lit(4) * self.clone().dag1223() - self.clone().mag2()) / lit(3);
        // DAG1223 = (3 * TAG + MAG2) / 4
        let dag1223 = || (lit(3) * self.clone().tag() + self.clone().mag2()) / lit(4);
        // MAG2 = 4 * DAG1223 - 3 * TAG
        let mag2 = || lit(4) * self.clone().dag1223() - lit(3) * self.clone().tag();
        // 2 * DAG13 = 3 * TAG - MAG2 (стр. 116)
        let dag13 = || {
            // DAG13 = (3 * TAG - MAG2) / 2
            let mag2 = || (lit(3) * self.clone().tag() - self.clone().mag2()) / lit(2);
            // DAG13 = 3 * TAG - 2 * DAG1223
            let dag1223 = || lit(3) * self.clone().tag() - lit(2) * self.clone().dag1223();
            as_struct(vec![
                dag1223()
                    .clip_min_if(settings.unsigned)
                    .normalize_if(settings.normalize.theoretical)
                    .alias("Diacylglycerol1223"),
                mag2()
                    .clip_min_if(settings.unsigned)
                    .normalize_if(settings.normalize.theoretical)
                    .alias("Monoacylglycerol2"),
            ])
        };
        as_struct(vec![
            tag()
                .clip_min_if(settings.unsigned)
                .normalize_if(settings.normalize.theoretical)
                .alias("Triacylglycerol"),
            dag1223()
                .normalize_if(settings.normalize.theoretical)
                .alias("Diacylglycerol1223"),
            mag2()
                .clip_min_if(settings.unsigned)
                .normalize_if(settings.normalize.theoretical)
                .alias("Monoacylglycerol2"),
            dag13().alias("Diacylglycerol13"),
        ])
        .alias("Theoretical")
    }
}

impl Tag for TheoreticalExpr {
    fn tag(self) -> Expr {
        self.0.struct_().field_by_name("Triacylglycerol")
    }
}

impl Dag1223 for TheoreticalExpr {
    fn dag1223(self) -> Expr {
        self.0.struct_().field_by_name("Diacylglycerol1223")
    }
}

impl Mag2 for TheoreticalExpr {
    fn mag2(self) -> Expr {
        self.0.struct_().field_by_name("Monoacylglycerol2")
    }
}

/// Factors
#[derive(Clone, Debug)]
struct FactorsExpr(Expr);

impl FactorsExpr {
    fn compute(self, fatty_acid: Expr) -> Expr {
        as_struct(vec![
            FattyAcidExpr::enrichment_factor(self.clone().mag2(), self.clone().tag())
                .alias("Enrichment"),
            fatty_acid
                .fatty_acid()
                .selectivity_factor(self.clone().mag2(), self.tag())
                .alias("Selectivity"),
        ])
        .alias("Factors")
    }
}

impl Tag for FactorsExpr {
    fn tag(self) -> Expr {
        self.0.struct_().field_by_name("Triacylglycerol")
    }
}

impl Mag2 for FactorsExpr {
    fn mag2(self) -> Expr {
        self.0.struct_().field_by_name("Monoacylglycerol2")
    }
}

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
