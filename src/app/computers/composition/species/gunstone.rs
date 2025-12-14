use super::Discriminants;
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
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

pub(super) fn compute(
    mut lazy_frame: LazyFrame,
    discriminants: &Discriminants,
) -> PolarsResult<LazyFrame> {
    println!("lazy_frame g0: {}", lazy_frame.clone().collect().unwrap());
    // lazy_frame = lazy_frame
    //     .clone()
    //     .select([as_struct(vec![
    //         col(LABEL),
    //         col(FATTY_ACID),
    //         col(STEREOSPECIFIC_NUMBERS13).alias("Value"),
    //     ])
    lazy_frame = lazy_frame.select([
        col(LABEL),
        col(FATTY_ACID),
        col(STEREOSPECIFIC_NUMBERS123).alias("Value"),
    ]);
    println!("lazy_frame g1: {}", lazy_frame.clone().collect().unwrap());
    let factors = factors(lazy_frame.clone())?;
    println!("factor: {}", factors.clone().collect().unwrap());
    // println!("lazy_frame g2: {}", lazy_frame.clone().collect().unwrap());
    let discriminants = &discriminants.0;
    let discriminants = df! {
        LABEL => Series::from_iter(discriminants.keys().cloned()),
        "Factor1" => Series::from_iter(discriminants.values().map(|values| values[0])),
        "Factor2" => Series::from_iter(discriminants.values().map(|values| values[1])),
        "Factor3" => Series::from_iter(discriminants.values().map(|values| values[2])),
    }?;
    println!("Discriminants: {discriminants}");
    // lazy_frame = lazy_frame
    //     .join(
    //         discriminants.lazy(),
    //         [col(LABEL)],
    //         [col(LABEL)],
    //         JoinArgs::new(JoinType::Left).with_coalesce(JoinCoalesce::CoalesceColumns),
    //     )
    //     .select([
    //         col(LABEL),
    //         col(FATTY_ACID),
    //         (col("Value") * col("Factor1")).alias("Value1"),
    //         (col("Value") * col("Factor2")).alias("Value2"),
    //         (col("Value") * col("Factor3")).alias("Value3"),
    //     ]);
    lazy_frame = lazy_frame.select([
        col(LABEL),
        col(FATTY_ACID),
        col("Value").alias("Value1"),
        col("Value").alias("Value2"),
        col("Value").alias("Value3"),
    ]);
    println!("lazy_frame g2: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = cartesian_product(lazy_frame)?;
    println!("lazy_frame g3: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = lazy_frame
        .with_column(
            col(TRIACYLGLYCEROL)
                .triacylglycerol()
                .map_expr(|expr| expr.fatty_acid().is_unsaturated(None))
                .triacylglycerol()
                .sum()
                .alias("MTC"),
        )
        .join(
            factors,
            [col("MTC")],
            [col("MTC")],
            JoinArgs::new(JoinType::Left).with_coalesce(JoinCoalesce::CoalesceColumns),
        )
        .select([
            col(LABEL),
            col(TRIACYLGLYCEROL),
            (col("Value") * col("Factor")).normalize(true),
        ]);
    println!("lazy_frame g4: {}", lazy_frame.clone().collect().unwrap());
    Ok(lazy_frame)
}

// 0.055395+0.944605=1
// 0.0+0.006904+0.152377+0.840718=0.999999
fn factors(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    // lazy_frame = lazy_frame
    //     .clone()
    //     .select([
    //         col("Value")
    //             .nullify(col(FATTY_ACID).fatty_acid().is_saturated())
    //             .alias("S"),
    //         col("Value")
    //             .nullify(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
    //             .alias("U"),
    //     ])
    //     .sum();
    // println!("lazy_frame gx1: {}", lazy_frame.clone().collect().unwrap());
    let predicate = col("S").lt_eq(lit(2.0 / 3.0));
    // S, U
    lazy_frame = lazy_frame.select([
        col("Value")
            .nullify(col(FATTY_ACID).fatty_acid().is_saturated())
            .sum()
            .alias("S"),
        col("Value")
            .nullify(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
            .sum()
            .alias("U"),
    ]);
    println!("lazy_frame gx1: {}", lazy_frame.clone().collect().unwrap());
    // [S/3;S/3;S/3], [S/3;S/3;U/3], [S/3;U/3;U/3], [U/3;U/3;U/3]
    lazy_frame = lazy_frame.select([
        ternary_expr(predicate.clone(), lit(0.0), lit(1) - lit(3) * col("U")).alias("S_3"),
        ternary_expr(
            predicate.clone(),
            (lit(3.0 / 2.0) * col("S")).pow(2),
            lit(3) * col("U"),
        )
        .alias("S_2U"),
        ternary_expr(
            predicate.clone(),
            lit(3.0 / 2.0) * col("S") * (lit(3) * col("U") - lit(1)),
            lit(0),
        )
        .alias("SU_2"),
        ternary_expr(
            predicate,
            (lit(1) - lit(3.0 / 2.0) * col("S")).pow(2),
            lit(0.0),
        )
        .alias("U_3"),
    ]);
    println!("lazy_frame gx2: {}", lazy_frame.clone().collect().unwrap());
    // Unpivot
    lazy_frame = lazy_frame
        .unpivot(UnpivotArgsDSL {
            on: by_name(["S_3", "S_2U", "SU_2", "U_3"], true),
            index: empty(),
            variable_name: None,
            value_name: Some(PlSmallStr::from_static("Factor")),
        })
        .with_row_index("MTC", None)
        .select([col("MTC"), col("Factor")]);
    println!("lazy_frame gx3: {}", lazy_frame.clone().collect().unwrap());
    // let s = data_frame["S"].f64()?.first().unwrap();
    // let u = data_frame["U"].f64()?.first().unwrap();
    // // assert!(1.0 - u - s <= f64::EPSILON, "s + u != 1.0");
    // // [SSS]
    // let s3 = if s <= 2.0 / 3.0 { 0.0 } else { 3.0 * s - 2.0 } / s.powi(3);
    // // [SSU], [USS], [SUS]
    // let s2u = if s <= 2.0 / 3.0 {
    //     (3.0 * s / 2.0).powi(2)
    // } else {
    //     3.0 * u
    // } / (3.0 * s.powi(2) * u);
    // // [SUU], [USU], [UUS]
    // let su2 = if s <= 2.0 / 3.0 {
    //     3.0 * s * (3.0 * u - 1.0) / 2.0
    // } else {
    //     0.0
    // } / (3.0 * s * u.powi(2));
    // // [UUU]
    // let u3 = if s <= 2.0 / 3.0 {
    //     ((3.0 * u - 1.0) / 2.0).powi(2)
    // } else {
    //     0.0
    // } / u.powi(3);
    // let factor = df! {
    //     "TMC" => Series::from_iter([0, 1, 2, 3]),
    //     "Factor" => Series::from_iter([s3, s2u, su2, u3]),
    // }?;
    Ok(lazy_frame)
}

fn cartesian_product(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    // Cartesian product (TAG from FA)
    lazy_frame = lazy_frame
        .clone()
        .select([as_struct(vec![
            col(LABEL),
            col(FATTY_ACID),
            col("Value1").alias("Value"),
        ])
        .alias(STEREOSPECIFIC_NUMBERS1)])
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col(LABEL),
                col(FATTY_ACID),
                col("Value2").alias("Value"),
            ])
            .alias(STEREOSPECIFIC_NUMBERS2)]),
            None,
        )
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col(LABEL),
                col(FATTY_ACID),
                col("Value3").alias("Value"),
            ])
            .alias(STEREOSPECIFIC_NUMBERS3)]),
            None,
        );
    // Restruct
    let label = |name| col(name).struct_().field_by_name(LABEL).alias(name);
    let fatty_acid = |name| col(name).struct_().field_by_name(FATTY_ACID).alias(name);
    let value = |name| col(name).struct_().field_by_name("Value");
    lazy_frame = lazy_frame.select([
        as_struct(vec![
            label(STEREOSPECIFIC_NUMBERS1),
            label(STEREOSPECIFIC_NUMBERS2),
            label(STEREOSPECIFIC_NUMBERS3),
        ])
        .alias(LABEL),
        as_struct(vec![
            fatty_acid(STEREOSPECIFIC_NUMBERS1),
            fatty_acid(STEREOSPECIFIC_NUMBERS2),
            fatty_acid(STEREOSPECIFIC_NUMBERS3),
        ])
        .alias(TRIACYLGLYCEROL),
        value(STEREOSPECIFIC_NUMBERS1)
            * value(STEREOSPECIFIC_NUMBERS2)
            * value(STEREOSPECIFIC_NUMBERS3),
    ]);
    Ok(lazy_frame)
}

// s3: 0.0,
// s2u: (3.0 * s / 2.0).powi(2),
// su2: 3.0 * s * (3.0 * u - 1.0) / 2.0,
// u3: ((3.0 * u - 1.0) / 2.0).powi(2),
// fn gunstone1(series: &Series) -> PolarsResult<Series> {
//     let triacylglycerol = series.struct_()?.field_by_name("Triacylglycerol")?.f64()?;
//     let fatty_acid = series.struct_()?.field_by_name(FATTY_ACID)?.fa();
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
