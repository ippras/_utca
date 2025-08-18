use crate::{
    app::panes::composition::settings::{Discriminants, Method},
    utils::{Hashed, hash},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_ext::expr::ExprExt as _;
use tracing::instrument;

/// Composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    pub(super) fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let lazy_frame = match key.index {
            Some(index) => {
                let frame = &key.frames[index];
                let mut lazy_frame = frame.data.clone().lazy();
                lazy_frame = compute(lazy_frame, key.settings())?;
                lazy_frame
            }
            None => {
                let compute = |frame: &MetaDataFrame| -> PolarsResult<LazyFrame> {
                    Ok(compute(frame.data.clone().lazy(), key.settings())?.select([
                        hash(as_struct(vec![col(LABEL), col(TRIACYLGLYCEROL)])),
                        col(LABEL),
                        col(TRIACYLGLYCEROL),
                        col("Value").alias(frame.meta.format(".").to_string()),
                    ]))
                };
                let mut lazy_frame = compute(&key.frames[0])?;
                for frame in &key.frames[1..] {
                    lazy_frame = lazy_frame.join(
                        compute(frame)?,
                        [col("Hash"), col(LABEL), col(TRIACYLGLYCEROL)],
                        [col("Hash"), col(LABEL), col(TRIACYLGLYCEROL)],
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                    );
                }
                lazy_frame = lazy_frame.drop(by_name(["Hash"], true));
                lazy_frame = lazy_frame.select(mean_and_standard_deviation(key.ddof)?);
                lazy_frame
            }
        };
        let mut data_frame = lazy_frame.collect()?;
        let hash = data_frame.hash_rows(None)?.xor_reduce().unwrap_or_default();
        Ok(Hashed {
            value: data_frame,
            hash,
        })
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
    pub(crate) frames: &'a Hashed<Vec<MetaDataFrame>>,
    pub(crate) index: Option<usize>,
    pub(crate) ddof: u8,
    pub(crate) method: Method,
    pub(crate) discriminants: &'a Discriminants,
}

#[derive(Clone, Copy, Debug, Hash, PartialEq)]
struct Settings<'a> {
    pub(crate) ddof: u8,
    pub(crate) discriminants: &'a Discriminants,
    pub(crate) method: Method,
}

impl Key<'_> {
    fn settings(&self) -> Settings<'_> {
        Settings {
            ddof: self.ddof,
            discriminants: self.discriminants,
            method: self.method,
        }
    }
}

/// Composition value
type Value = Hashed<DataFrame>;

fn compute(lazy_frame: LazyFrame, settings: Settings) -> PolarsResult<LazyFrame> {
    match settings.method {
        Method::Gunstone => gunstone(lazy_frame, settings.discriminants),
        Method::VanderWal => vander_wal(lazy_frame),
    }
}

// 0.0 + 0.048672 + 0.000623 + 0.950705 = 1.0
// let u = 1.0 - s;
// if s <= 2.0 / 3.0 {
//     Self {
//         s,
//         u,
//         s3: 0.0,
//         s2u: (3.0 * s / 2.0).powi(2),
//         su2: 3.0 * s * (3.0 * u - 1.0) / 2.0,
//         u3: ((3.0 * u - 1.0) / 2.0).powi(2),
//     }
// } else {
//     Self {
//         s,
//         u,
//         s3: 3.0 * s - 2.0,
//         s2u: 3.0 * u,
//         su2: 0.0,
//         u3: 0.0,
//     }
// }
// fn factor(&self, r#type: Tag<Saturation>) -> f64 {
//     match r#type.into() {
//         S3 => self.s3 / self.s.powi(3),                    // [SSS]
//         S2U => self.s2u / (self.s.powi(2) * self.u) / 3.0, // [SSU], [USS], [SUS]
//         SU2 => self.su2 / (self.s * self.u.powi(2)) / 3.0, // [SUU], [USU], [UUS]
//         U3 => self.u3 / self.u.powi(3),                    // [UUU]
//     }
// }

fn gunstone(mut lazy_frame: LazyFrame, discriminants: &Discriminants) -> PolarsResult<LazyFrame> {
    println!("lazy_frame g0: {}", lazy_frame.clone().collect().unwrap());
    // lazy_frame = lazy_frame
    //     .clone()
    //     .select([as_struct(vec![
    //         col(LABEL),
    //         col(FATTY_ACID),
    //         col(STEREOSPECIFIC_NUMBER13).alias("Value"),
    //     ])
    lazy_frame = lazy_frame.select([
        col(LABEL),
        col(FATTY_ACID),
        col(STEREOSPECIFIC_NUMBER123).alias("Value"),
    ]);
    println!("lazy_frame g1: {}", lazy_frame.clone().collect().unwrap());
    let factor = gunstone_factor(lazy_frame.clone())?;
    println!("factor: {factor}");
    //
    println!("lazy_frame g2: {}", lazy_frame.clone().collect().unwrap());
    let discriminants = &discriminants.0;
    let discriminants = df! {
        LABEL => Series::from_iter(discriminants.keys().cloned()),
        "Factor1" => Series::from_iter(discriminants.values().map(|values| values[0])),
        "Factor2" => Series::from_iter(discriminants.values().map(|values| values[1])),
        "Factor3" => Series::from_iter(discriminants.values().map(|values| values[2])),
    }?;
    println!("Discriminants: {discriminants}");
    lazy_frame = lazy_frame
        .join(
            discriminants.lazy(),
            [col(LABEL)],
            [col(LABEL)],
            JoinArgs::new(JoinType::Left).with_coalesce(JoinCoalesce::CoalesceColumns),
        )
        .select([
            col(LABEL),
            col(FATTY_ACID),
            (col("Value") * col("Factor1")).alias("Value1"),
            (col("Value") * col("Factor2")).alias("Value2"),
            (col("Value") * col("Factor3")).alias("Value3"),
            // col("Value") * col("Factor"),
            // col("Value") * col("Factor").arr().get(lit(0), false),
            // col("Value") * col("Factor").arr().get(lit(1), false),
            // col("Value") * col("Factor").arr().get(lit(2), false),
        ]);
    println!("lazy_frame g25: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = gunstone_cartesian_product(lazy_frame)?;
    println!("lazy_frame g3: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = lazy_frame
        .with_column(
            col(FATTY_ACID)
                .triacylglycerol()
                .map_expr(|expr| expr.fatty_acid().is_unsaturated(None))
                .triacylglycerol()
                .sum()
                .alias("TMC"),
        )
        .join(
            factor.lazy(),
            [col("TMC")],
            [col("TMC")],
            JoinArgs::new(JoinType::Left).with_coalesce(JoinCoalesce::CoalesceColumns),
        )
        .select([
            col(LABEL),
            col(FATTY_ACID),
            (col("Value") * col("Factor")).normalize(),
        ]);
    println!("lazy_frame g4: {}", lazy_frame.clone().collect().unwrap());
    Ok(lazy_frame)
}

fn gunstone_factor(lazy_frame: LazyFrame) -> PolarsResult<DataFrame> {
    let data_frame = lazy_frame
        .clone()
        .select([
            col("Value")
                .nullify(col("FattyAcid").fatty_acid().is_saturated())
                .alias("S"),
            col("Value")
                .nullify(col("FattyAcid").fatty_acid().is_unsaturated(None))
                .alias("U"),
        ])
        .sum()
        .collect()?;
    let s = data_frame["S"].f64()?.first().unwrap();
    let u = data_frame["U"].f64()?.first().unwrap();
    // assert!(1.0 - u - s <= f64::EPSILON, "s + u != 1.0");
    // [SSS]
    let s3 = if s <= 2.0 / 3.0 { 0.0 } else { 3.0 * s - 2.0 } / s.powi(3);
    // [SSU], [USS], [SUS]
    let s2u = if s <= 2.0 / 3.0 {
        (3.0 * s / 2.0).powi(2)
    } else {
        3.0 * u
    } / (3.0 * s.powi(2) * u);
    // [SUU], [USU], [UUS]
    let su2 = if s <= 2.0 / 3.0 {
        3.0 * s * (3.0 * u - 1.0) / 2.0
    } else {
        0.0
    } / (3.0 * s * u.powi(2));
    // [UUU]
    let u3 = if s <= 2.0 / 3.0 {
        ((3.0 * u - 1.0) / 2.0).powi(2)
    } else {
        0.0
    } / u.powi(3);
    let factor = df! {
        "TMC" => Series::from_iter([0, 1, 2, 3]),
        "Factor" => Series::from_iter([s3, s2u, su2, u3]),
    }?;
    Ok(factor)
}

fn gunstone_cartesian_product(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
        .clone()
        .select([as_struct(vec![
            col("Label"),
            col("FattyAcid"),
            col("Value1").alias("Value"),
        ])
        .alias("StereospecificNumber1")])
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col("Label"),
                col("FattyAcid"),
                col("Value2").alias("Value"),
            ])
            .alias("StereospecificNumber2")]),
            None,
        )
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col("Label"),
                col("FattyAcid"),
                col("Value3").alias("Value"),
            ])
            .alias("StereospecificNumber3")]),
            None,
        );
    // Restruct
    lazy_frame = lazy_frame.select([
        as_struct(vec![
            col("StereospecificNumber1")
                .struct_()
                .field_by_name("Label")
                .alias("StereospecificNumber1"),
            col("StereospecificNumber2")
                .struct_()
                .field_by_name("Label")
                .alias("StereospecificNumber2"),
            col("StereospecificNumber3")
                .struct_()
                .field_by_name("Label")
                .alias("StereospecificNumber3"),
        ])
        .alias("Label"),
        as_struct(vec![
            col("StereospecificNumber1")
                .struct_()
                .field_by_name("FattyAcid")
                .alias("StereospecificNumber1"),
            col("StereospecificNumber2")
                .struct_()
                .field_by_name("FattyAcid")
                .alias("StereospecificNumber2"),
            col("StereospecificNumber3")
                .struct_()
                .field_by_name("FattyAcid")
                .alias("StereospecificNumber3"),
        ])
        .alias("FattyAcid"),
        col("StereospecificNumber1")
            .struct_()
            .field_by_name("Value")
            * col("StereospecificNumber2")
                .struct_()
                .field_by_name("Value")
            * col("StereospecificNumber3")
                .struct_()
                .field_by_name("Value"),
    ]);
    Ok(lazy_frame)
}

// 1,3-sn 2-sn 1,2,3-sn
// PSC:
// [abc] = 2*[a_{13}]*[_b2]*[c_{13}]
// [aab] = 2*[a_{13}]*[a_2]*[b13]
// [aba] = [a13]^2*[b2]
// `2*[a_{13}]` - потому что зеркальные ([abc]=[cba], [aab]=[baa]).
// SSC: [abc] = [a_{13}]*[b_2]*[c_{13}]
fn vander_wal(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    // Cartesian product (TAG from FA)
    lazy_frame = lazy_frame
        .clone()
        .select([as_struct(vec![
            col(LABEL),
            col(FATTY_ACID),
            col(STEREOSPECIFIC_NUMBER13).alias("Value"),
        ])
        .alias(STEREOSPECIFIC_NUMBER1)])
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col(LABEL),
                col(FATTY_ACID),
                col(STEREOSPECIFIC_NUMBER2).alias("Value"),
            ])
            .alias(STEREOSPECIFIC_NUMBER2)]),
            None,
        )
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col(LABEL),
                col(FATTY_ACID),
                col(STEREOSPECIFIC_NUMBER13).alias("Value"),
            ])
            .alias(STEREOSPECIFIC_NUMBER3)]),
            None,
        );
    // Restruct
    let label = |name| col(name).struct_().field_by_name(LABEL).alias(name);
    let fatty_acid = |name| col(name).struct_().field_by_name(FATTY_ACID).alias(name);
    let value = |name| col(name).struct_().field_by_name("Value");
    lazy_frame = lazy_frame.select([
        as_struct(vec![
            label(STEREOSPECIFIC_NUMBER1),
            label(STEREOSPECIFIC_NUMBER2),
            label(STEREOSPECIFIC_NUMBER3),
        ])
        .alias(LABEL),
        as_struct(vec![
            fatty_acid(STEREOSPECIFIC_NUMBER1),
            fatty_acid(STEREOSPECIFIC_NUMBER2),
            fatty_acid(STEREOSPECIFIC_NUMBER3),
        ])
        .alias(TRIACYLGLYCEROL),
        value(STEREOSPECIFIC_NUMBER1)
            * value(STEREOSPECIFIC_NUMBER2)
            * value(STEREOSPECIFIC_NUMBER3),
    ]);
    Ok(lazy_frame)
}

fn mean_and_standard_deviation(ddof: u8) -> PolarsResult<[Expr; 3]> {
    let array = || concat_arr(vec![all().exclude_cols([LABEL, TRIACYLGLYCEROL]).as_expr()]);
    Ok([
        col(LABEL),
        col(TRIACYLGLYCEROL),
        as_struct(vec![
            array()?.arr().mean().alias("Mean"),
            array()?.arr().std(ddof).alias("StandardDeviation"),
            array()?.alias("Repetitions"),
        ])
        .alias("Value"),
    ])
}

// s3: 0.0,
// s2u: (3.0 * s / 2.0).powi(2),
// su2: 3.0 * s * (3.0 * u - 1.0) / 2.0,
// u3: ((3.0 * u - 1.0) / 2.0).powi(2),
// fn gunstone1(series: &Series) -> PolarsResult<Series> {
//     let triacylglycerol = series.struct_()?.field_by_name("Triacylglycerol")?.f64()?;
//     let fatty_acid = series.struct_()?.field_by_name("FattyAcid")?.fa();
//     let Some(s) = triacylglycerol.filter(fatty_acid).sum() else {
//         polars_bail!(NoData: "Triacylglycerol");
//     };
//     let u = 1.0 - s;
//     let condition = s <= 2.0 / 3.0;
//     let s3 = if condition { 0.0 } else { 3.0 * s - 2.0 };
//     let s2u = if condition {
//         (3.0 * s / 2.0).powi(2)
//     } else {
//         3.0 * u
//     };
//     let su2 = if condition {
//         3.0 * s * (3.0 * u - 1.0) / 2.0
//     } else {
//         0.0
//     };
//     let u3 = if condition {
//         ((3.0 * u - 1.0) / 2.0).powi(2)
//     } else {
//         0.0
//     };
//     Ok(StructChunked::from_series(
//         PlSmallStr::EMPTY,
//         series.len(),
//         [
//             Scalar::new(DataType::Float64, s.into()).into_series("S".into()),
//             Scalar::new(DataType::Float64, u.into()).into_series("U".into()),
//             Scalar::new(DataType::Float64, s3.into()).into_series("S3".into()),
//             Scalar::new(DataType::Float64, s2u.into()).into_series("S2U".into()),
//             Scalar::new(DataType::Float64, su2.into()).into_series("SU2".into()),
//             Scalar::new(DataType::Float64, u3.into()).into_series("U3".into()),
//         ]
//         .iter(),
//     )?
//     .into_series())
// }

// impl Composer {
//     fn gunstone(&mut self, key: Key) -> Tree<Meta, Data> {
//         let Key { context } = key;
//         let tags123 = &context
//             .state
//             .entry()
//             .data
//             .calculated
//             .tags123
//             .experimental
//             .normalized;
//         let tags1 = discriminated(&context, Sn::One);
//         let tags2 = discriminated(&context, Sn::Two);
//         let tags3 = discriminated(&context, Sn::Three);
//         let s = zip(tags123, &context.state.entry().meta.formulas)
//             .filter_map(|(value, formula)| match formula.saturation() {
//                 Saturated => Some(value),
//                 Unsaturated => None,
//             })
//             .sum();
//         let gunstone = Gunstone::new(s);
//         let ungrouped = repeat(0..context.state.entry().len())
//             .take(3)
//             .multi_cartesian_product()
//             .map(|indices| {
//                 let tag = Tag([indices[0], indices[1], indices[2]])
//                     .compose(context.settings.composition.tree.leafs.stereospecificity);
//                 let value = gunstone.factor(context.r#type(tag))
//                     * tags1[indices[0]]
//                     * tags2[indices[1]]
//                     * tags3[indices[2]];
//                 (tag, value.into())
//             })
//             .into_grouping_map()
//             .sum();
//         Tree::from(ungrouped.group_by_key(key))
//     }
//     // 1,3-sn 2-sn 1,2,3-sn
//     // [abc] = 2*[a13]*[b2]*[c13]
//     // [aab] = 2*[a13]*[a2]*[b13]
//     // [aba] = [a13]^2*[b2]
//     // [abc] = [a13]*[b2]*[c13]
//     // `2*[a13]` - потому что зеркальные ([abc]=[cba], [aab]=[baa]).
//     fn vander_wal(&mut self, key: Key) -> Tree<Meta, Data> {
//         let Key { context } = key;
//         let dags13 = &context
//             .state
//             .entry()
//             .data
//             .calculated
//             .dags13
//             .value(context.settings.calculation.from)
//             .normalized;
//         let mags2 = &context
//             .state
//             .entry()
//             .data
//             .calculated
//             .mags2
//             .value()
//             .normalized;
//         let ungrouped = repeat(0..context.state.entry().len())
//             .take(3)
//             .multi_cartesian_product()
//             .map(|indices| {
//                 let tag = Tag([indices[0], indices[1], indices[2]])
//                     .compose(context.settings.composition.tree.leafs.stereospecificity);
//                 let value = dags13[indices[0]] * mags2[indices[1]] * dags13[indices[2]];
//                 (tag, value.into())
//             })
//             .into_grouping_map()
//             .sum();
//         Tree::from(ungrouped.group_by_key(key))
//     }
// }

/// Gunstone
struct Gunstone {
    s: f64,
    u: f64,
    s3: f64,
    s2u: f64,
    su2: f64,
    u3: f64,
}

impl Gunstone {
    fn new(s: f64) -> Self {
        let u = 1.0 - s;
        if s <= 2.0 / 3.0 {
            Self {
                s,
                u,
                s3: 0.0,
                s2u: (3.0 * s / 2.0).powi(2),
                su2: 3.0 * s * (3.0 * u - 1.0) / 2.0,
                u3: ((3.0 * u - 1.0) / 2.0).powi(2),
            }
        } else {
            Self {
                s,
                u,
                s3: 3.0 * s - 2.0,
                s2u: 3.0 * u,
                su2: 0.0,
                u3: 0.0,
            }
        }
    }

    // fn factor(&self, r#type: Tag<Saturation>) -> f64 {
    //     match r#type.into() {
    //         S3 => self.s3 / self.s.powi(3),                    // [SSS]
    //         S2U => self.s2u / (self.s.powi(2) * self.u) / 3.0, // [SSU], [USS], [SUS]
    //         SU2 => self.su2 / (self.s * self.u.powi(2)) / 3.0, // [SUU], [USU], [UUS]
    //         U3 => self.u3 / self.u.powi(3),                    // [UUU]
    //     }
    // }
}

// fn discriminated(context: &Context, sn: Sn) -> Vec<f64> {
//     context
//         .state
//         .entry()
//         .data
//         .calculated
//         .tags123
//         .experimental
//         .normalized
//         .iter()
//         .enumerate()
//         .map(move |(index, &value)| {
//             let discrimination = &context.settings.composition.discrimination;
//             match sn {
//                 Sn::One => discrimination.get(&index),
//                 Sn::Two => discrimination.get(&index),
//                 Sn::Three => discrimination.get(&index),
//             }
//             .map_or(value, |&f| f * value)
//         })
//         .normalized()
// }
