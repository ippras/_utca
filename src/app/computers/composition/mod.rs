use crate::{
    app::panes::composition::settings::{Filter, Method, Order, Selection, Settings, Sort},
    special::composition::{MMC, MSC, NMC, NSC, SMC, SPC, SSC, TMC, TPC, TSC, UMC, USC},
    utils::Hashed,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_ext::{
    prelude::ExprExt as _,
    series::{column, round},
};
use std::{
    convert::identity,
    hash::{Hash, Hasher},
    process::exit,
};
use tracing::instrument;

/// Composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        // warn!("index: {:?}", key.settings.index);
        let mut settings = key.settings.clone();
        if settings.special.selections.is_empty() {
            settings.special.selections.push_back(Selection {
                composition: SSC,
                filter: Filter::new(),
            });
        }
        let settings = &settings;
        let mut lazy_frame = match settings.index {
            Some(index) => {
                let frame = &key.frames[index];
                let mut lazy_frame = frame.data.clone().lazy();
                lazy_frame = compute(lazy_frame, settings)?;
                lazy_frame
            }
            None => {
                let compute = |frame: &MetaDataFrame| -> PolarsResult<LazyFrame> {
                    Ok(compute(frame.data.clone().lazy(), settings)?.select([
                        col("Keys").hash(),
                        col("Keys"),
                        col("Values").alias(frame.meta.title()),
                    ]))
                };
                let mut lazy_frame = compute(&key.frames[0])?;
                for frame in &key.frames[1..] {
                    lazy_frame = lazy_frame.join(
                        compute(frame)?,
                        [col("Hash"), col("Keys")],
                        [col("Hash"), col("Keys")],
                        JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
                    );
                }
                lazy_frame = lazy_frame.drop([col("Hash")]);
                lazy_frame = meta(lazy_frame, settings)?;
                lazy_frame
            }
        };
        // // Filter
        // lazy_frame = filter(lazy_frame, settings);
        // Sort
        lazy_frame = sort(lazy_frame, settings);
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Composition key
#[derive(Clone, Copy, Debug, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frames: &'a Hashed<Vec<MetaDataFrame>>,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frames.hash(state);
        self.settings.index.hash(state);
        self.settings.special.hash(state);
    }
}

/// Composition value
type Value = DataFrame;

fn compute(lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    match &settings.special.method {
        Method::Gunstone => gunstone(lazy_frame, settings),
        Method::VanderWal => vander_wal(lazy_frame, settings),
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
fn gunstone(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    println!("lazy_frame g0: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = lazy_frame.select([
        // col("Index"),
        col("Label"),
        col("FattyAcid"),
        col("Calculated")
            .struct_()
            .field_by_names(["Triacylglycerol"])
            .alias("Value"),
    ]);
    println!("lazy_frame g1: {}", lazy_frame.clone().collect().unwrap());

    let factor = gunstone_factor(lazy_frame.clone())?;
    println!("factor: {factor}");
    //
    println!("lazy_frame g2: {}", lazy_frame.clone().collect().unwrap());
    let discriminants = &settings.special.discriminants.0;
    let discriminants = df! {
        "Label" => Series::from_iter(discriminants.keys().cloned()),
        "Factor1" => Series::from_iter(discriminants.values().map(|values| values[0])),
        "Factor2" => Series::from_iter(discriminants.values().map(|values| values[1])),
        "Factor3" => Series::from_iter(discriminants.values().map(|values| values[2])),
    }?;
    println!("Discriminants: {discriminants}");
    lazy_frame = lazy_frame
        .join(
            discriminants.lazy(),
            [col("Label")],
            [col("Label")],
            JoinArgs::new(JoinType::Left).with_coalesce(JoinCoalesce::CoalesceColumns),
        )
        .select([
            col("Label"),
            col("FattyAcid"),
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
            col("FattyAcid")
                .tag()
                .map(|expr| expr.fa().is_unsaturated())
                .tag()
                .sum()
                .alias("TMC"),
        )
        .join(
            factor.lazy(),
            [col("TMC")],
            [col("TMC")],
            JoinArgs::new(JoinType::Left).with_coalesce(JoinCoalesce::CoalesceColumns),
        )
        .select([col("Label"), col("FattyAcid"), col("Value") * col("Factor")]);
    println!("lazy_frame g4: {}", lazy_frame.clone().collect().unwrap());

    // Compose
    lazy_frame = compose(lazy_frame, settings)?;
    Ok(lazy_frame)
}

fn gunstone_factor(lazy_frame: LazyFrame) -> PolarsResult<DataFrame> {
    let data_frame = lazy_frame
        .clone()
        .select([
            col("Value")
                .nullify(col("FattyAcid").fa().is_saturated())
                .alias("S"),
            col("Value")
                .nullify(col("FattyAcid").fa().is_unsaturated())
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
fn vander_wal(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame.select([
        col("Index"),
        col("Label"),
        col("FattyAcid"),
        col("Calculated")
            .struct_()
            .field_by_names(["Diacylglycerol13", "Monoacylglycerol2"]),
    ]);
    // Cartesian product (TAG from FA)
    lazy_frame = cartesian_product(lazy_frame)?;
    // Compose
    lazy_frame = compose(lazy_frame, settings)?;
    Ok(lazy_frame)
}

fn cartesian_product(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
        .clone()
        .select([as_struct(vec![
            col("Label"),
            col("FattyAcid"),
            col("Diacylglycerol13").alias("Value"),
        ])
        .alias("StereospecificNumber1")])
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col("Label"),
                col("FattyAcid"),
                col("Monoacylglycerol2").alias("Value"),
            ])
            .alias("StereospecificNumber2")]),
            None,
        )
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col("Label"),
                col("FattyAcid"),
                col("Diacylglycerol13").alias("Value"),
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

fn compose(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    // Composition
    for (index, selection) in settings.special.selections.iter().enumerate() {
        lazy_frame = lazy_frame.with_column(
            match selection.composition {
                MMC => col("FattyAcid")
                    .tag()
                    .mass(Some(lit(settings.special.adduct)))
                    .map(
                        column(round(settings.special.round_mass)),
                        GetOutput::same_type(),
                    )
                    .alias("MMC"),
                MSC => col("FattyAcid")
                    .tag()
                    .map(|expr| expr.fa().mass(None).round(settings.special.round_mass))
                    .alias("MSC"),
                NMC => col("FattyAcid").tag().ecn().alias("NMC"),
                NSC => col("FattyAcid")
                    .tag()
                    .map(|expr| expr.fa().ecn())
                    .alias("NSC"),
                SMC => col("Label")
                    .tag()
                    .non_stereospecific(identity, PermutationOptions::default())?
                    .alias("SMC"),
                SPC => col("Label")
                    .tag()
                    .positional(identity, PermutationOptions::default())
                    .alias("SPC"),
                SSC => col("Label").alias("SSC"),
                TMC => col("FattyAcid")
                    .tag()
                    .map(|expr| expr.fa().is_unsaturated())
                    .tag()
                    .sum()
                    // .non_stereospecific(
                    //     |expr| expr.fa().is_saturated(),
                    //     PermutationOptions::default().map(true),
                    // )?
                    .alias("TMC"),
                TPC => col("FattyAcid")
                    .tag()
                    .positional(
                        |expr| expr.fa().is_saturated(),
                        PermutationOptions::default().map(true),
                    )
                    .alias("TPC"),
                TSC => col("FattyAcid")
                    .tag()
                    .map(|expr| expr.fa().is_saturated())
                    .alias("TSC"),
                UMC => col("FattyAcid").tag().unsaturation().alias("UMC"),
                USC => col("FattyAcid")
                    .tag()
                    .map(|expr| expr.fa().unsaturated().sum())
                    .alias("USC"),
            }
            .alias(format!("Key{index}")),
        );
        // Value
        lazy_frame = lazy_frame.with_column(
            sum("Value")
                .over([as_struct(vec![col(format!("^Key[0-{index}]$"))])])
                .alias(format!("Value{index}")),
        );
    }
    // Group
    lazy_frame = lazy_frame
        .group_by([col(r#"^Key\d$"#), col(r#"^Value\d$"#)])
        .agg([as_struct(vec![col("Label"), col("FattyAcid"), col("Value")]).alias("Species")]);
    lazy_frame = lazy_frame.select([
        as_struct(vec![col(r#"^Key\d$"#)]).alias("Keys"),
        concat_arr(vec![col(r#"^Value\d$"#)])?.alias("Values"),
        col("Species"),
    ]);
    Ok(lazy_frame)
}

fn meta(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    // TODO [array_get?](https://docs.rs/polars/latest/polars/prelude/array/trait.ArrayNameSpace.html)
    let values = |index| {
        // TODO: .arr().to_list().list() for compute mean std with None
        concat_list([all()
            .exclude(["Keys", r#"^Value\d$"#])
            .arr()
            .get(lit(index as u32), true)])
    };
    for index in 0..settings.special.selections.len() {
        lazy_frame = lazy_frame.with_column(
            as_struct(vec![
                values(index)?.list().mean().alias("Mean"),
                values(index)?
                    .list()
                    .std(settings.special.ddof)
                    .alias("StandardDeviation"),
            ])
            .alias(format!("Value{index}")),
        );
    }
    // Group
    lazy_frame = lazy_frame.select([
        col("Keys"),
        concat_arr(vec![col(r#"^Value\d$"#)])?.alias("Values"),
    ]);
    Ok(lazy_frame)
}

fn sort(mut lazy_frame: LazyFrame, settings: &Settings) -> LazyFrame {
    let mut sort_options = SortMultipleOptions::default();
    if let Order::Descending = settings.special.order {
        sort_options = sort_options
            .with_order_descending(true)
            .with_nulls_last(true);
    }
    lazy_frame = match settings.special.sort {
        Sort::Key => lazy_frame.sort_by_exprs([col("Keys")], sort_options),
        Sort::Value => {
            let mut expr = col("Values");
            if settings.index.is_none() {
                expr = expr
                    .arr()
                    .to_list()
                    .list()
                    .eval(col("").struct_().field_by_name("Mean"), true);
            }
            lazy_frame.sort_by_exprs([expr], sort_options)
        }
    };
    // TODO sort species
    // lazy_frame = lazy_frame.with_columns([col("Species").list().eval(
    //     col("").sort_by(
    //         [col("").struct_().field_by_name("FA").fa().ecn()],
    //         Default::default(),
    //     ),
    //     true,
    // )]);
    lazy_frame
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

pub(crate) mod filtered;
pub(crate) mod unique;
