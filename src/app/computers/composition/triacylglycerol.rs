use crate::{
    app::panes::composition::settings::{Filter, Method, Order, Selection, Settings, Sort},
    special::composition::{MNC, MSC, NNC, NSC, SNC, SPC, SSC, TNC, TPC, TSC, UNC, USC},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use metadata::MetaDataFrame;
use polars::prelude::*;
use polars_ext::{ExprExt as _, column, functions::round};
use std::{
    convert::identity,
    hash::{Hash, Hasher},
    process::exit,
};
use tracing::warn;

/// Composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition computer
#[derive(Default)]
pub(crate) struct Computer;

// impl Computer {
//     pub(crate) async fn compute(&mut self, key: Key<'_>) -> PolarsResult<Value> {
//         let mut schema = Schema::with_capacity(3);
//         let mut compositions = Vec::new();
//         for index in 0..key.settings.groups.len() {
//             let composition = format!("Composition{index}");
//             compositions.push(col(&composition));
//             schema.insert(composition.into(), DataType::Null);
//         }
//         let mut lazy_frame = DataFrame::empty_with_schema(&schema).lazy();
//         println!("lazy_frame 0: {}", lazy_frame.clone().collect().unwrap());
//         for entry in key.entries {
//             if entry.checked {
//                 println!("entry: {entry:?}");
//                 let other = match key.settings.method {
//                     Method::Gunstone => self.gunstone(&entry.fatty_acids, key.settings)?.0,
//                     Method::VanderWal => {
//                         self.vander_wal(&entry.fatty_acids, key.settings)?
//                             .0
//                             .select([
//                                 // col("Composition"),
//                                 col("Composition").struct_().field_by_names(Some("*")),
//                                 col("Values").alias(&entry.name),
//                             ])
//                     }
//                 };
//                 println!("lazy_frame other: {}", other.clone().collect().unwrap());
//                 lazy_frame = lazy_frame.join(
//                     other,
//                     &compositions,
//                     &compositions,
//                     JoinArgs::new(key.settings.join.into())
//                         .with_coalesce(JoinCoalesce::CoalesceColumns),
//                 );
//             }
//         }
//         match key.settings.method {
//             Method::Gunstone => unreachable!(),
//             Method::VanderWal => {
//                 let mut lazy_frames = key.entries.into_iter().map(|entry| {
//                     self.vander_wal(&entry.fatty_acids, key.settings)
//                         .unwrap()
//                         .0
//                         .select([
//                             // col("Composition"),
//                             col("Composition").struct_().field_by_names(Some("*")),
//                             col("Values").alias(&entry.name),
//                         ])
//                 });
//                 if let Some(mut lazy_frame) = lazy_frames.next() {
//                     // lazy_frame
//                     let compositions = indexed_cols("Composition", 0..key.settings.groups.len());
//                     for other in lazy_frames {
//                         lazy_frame = lazy_frame.join(
//                             other,
//                             &compositions,
//                             &compositions,
//                             JoinArgs::new(key.settings.join.into())
//                                 .with_coalesce(JoinCoalesce::CoalesceColumns),
//                         );
//                     }
//                     // Filter
//                     if !key.settings.show.filtered {
//                         lazy_frame = lazy_frame.filter(
//                             all_horizontal([all()
//                                 .exclude([r#"^Composition\d$"#])
//                                 .struct_()
//                                 .field_by_name("Filter")])?
//                             .not(),
//                         );
//                     }
//                     // Sort
//                     lazy_frame = lazy_frame.compositions().sort(key.settings);
//                     // Meta
//                     lazy_frame = lazy_frame.compositions().meta(key.settings)?;
//                     println!("Meta: {}", lazy_frame.clone().collect().unwrap());
//                     return lazy_frame.collect();
//                 }
//             }
//         }
//         let mut schema = Schema::with_capacity(3);
//         schema.insert("Index".into(), DataType::UInt32);
//         for index in 0..key.settings.groups.len() {
//             schema.insert(format!("Composition{index}").into(), DataType::Null);
//         }
//         Ok(DataFrame::empty_with_schema(&schema))
//     }
// }

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        warn!("index: {:?}", key.settings.index);
        let mut settings = key.settings.clone();
        if settings.confirmed.selections.is_empty() {
            settings.confirmed.selections.push_back(Selection {
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
        // Filter
        lazy_frame = filter(lazy_frame, settings);
        // Sort
        lazy_frame = sort(lazy_frame, settings);
        // Index
        lazy_frame = lazy_frame.with_row_index("Index", None);
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
    pub(crate) frames: &'a [MetaDataFrame],
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.frames.hash(state);
        self.settings.index.hash(state);
        self.settings.confirmed.hash(state);
    }
}

/// Composition value
type Value = DataFrame;

fn compute(lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    match settings.confirmed.method {
        Method::Gunstone => vander_wal(lazy_frame, settings),
        Method::VanderWal => vander_wal(lazy_frame, settings),
    }
}

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
fn gunstone(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    println!("lazy_frame g0: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = lazy_frame.select([
        col("Index"),
        col("Label"),
        col("FattyAcid"),
        col("Calculated")
            .struct_()
            .field_by_names(["Triacylglycerol"]),
    ]);
    println!("lazy_frame g1: {}", lazy_frame.clone().collect().unwrap());
    let lazy_frame = lazy_frame.with_columns([
        col("Triacylglycerol")
            .filter(col("FattyAcid").fa().is_saturated())
            .sum()
            .alias("S"),
        col("Triacylglycerol")
            .filter(col("FattyAcid").fa().is_unsaturated())
            .sum()
            .alias("U"),
    ]);
    println!("lazy_frame g0: {}", lazy_frame.clone().collect().unwrap());
    let s = lazy_frame.clone().collect()?["_Sum"]
        .f64()?
        .first()
        .unwrap();
    println!("lazy_frame g1: {}", lazy_frame.clone().collect().unwrap());

    exit(0);
    // // Cartesian product (TAG from FA)
    // let mut tags = fatty_acids.cartesian_product()?;
    // // Filter
    // tags = tags.filter(settings);

    // // let gunstone = Gunstone::new(s);
    // let lazy_frame = key.fatty_acids.0.clone().lazy();
    // // lazy_frame = lazy_frame.select([
    // //     col("Label"),
    // //     col("Formula"),
    // //     col("TAG.Experimental"),
    // //     col("DAG1223.Experimental"),
    // //     col("MAG2.Experimental"),
    // //     col("DAG13.DAG1223.Calculated"),
    // //     col("DAG13.MAG2.Calculated"),
    // // ]);
    // // lazy_frame = lazy_frame.with_columns([s().alias("S"), u().alias("U")]);
    // println!("key.data_frame: {}", lazy_frame.clone().collect().unwrap());
    // lazy_frame.collect().unwrap()
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

// fn compose(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
//     // Composition
//     for (index, selection) in settings.confirmed.selections.iter().enumerate() {
//         lazy_frame = lazy_frame.with_column(
//             match selection.composition {
//                 MC => col("FattyAcid")
//                     .tag()
//                     .mass(Some(lit(settings.confirmed.adduct)))
//                     .map(
//                         column(round(settings.confirmed.round_mass)),
//                         GetOutput::same_type(),
//                     )
//                     .alias("MC"),
//                 PMC => col("FattyAcid")
//                     .tag()
//                     .positional(
//                         |expr| expr.fa().mass(None).round(settings.confirmed.round_mass),
//                         PermutationOptions::default().map(true),
//                     )
//                     .alias("PMC"),
//                 SMC => col("FattyAcid")
//                     .tag()
//                     .map(|expr| expr.fa().mass(None).round(settings.confirmed.round_mass))
//                     .alias("SMC"),
//                 EC => col("FattyAcid").tag().ecn().alias("NC"),
//                 PEC => col("FattyAcid")
//                     .tag()
//                     .positional(
//                         |expr| expr.fa().ecn(),
//                         PermutationOptions::default().map(true),
//                     )
//                     .alias("PNC"),
//                 SEC => col("FattyAcid")
//                     .tag()
//                     .map(|expr| expr.fa().ecn())
//                     .alias("SNC"),
//                 SC => col("Label")
//                     .tag()
//                     .non_stereospecific(identity, PermutationOptions::default())?
//                     .alias("SC"),
//                 PSC => col("Label")
//                     .tag()
//                     .positional(identity, PermutationOptions::default())
//                     .alias("PSC"),
//                 SSC => col("Label").alias("SSC"),
//                 TC => col("FattyAcid")
//                     .tag()
//                     .non_stereospecific(
//                         |expr| expr.fa().is_saturated(),
//                         PermutationOptions::default().map(true),
//                     )?
//                     .alias("TC"),
//                 PTC => col("FattyAcid")
//                     .tag()
//                     .positional(
//                         |expr| expr.fa().is_saturated(),
//                         PermutationOptions::default().map(true),
//                     )
//                     .alias("PTC"),
//                 STC => col("FattyAcid")
//                     .tag()
//                     .map(|expr| expr.fa().is_saturated())
//                     .alias("STC"),
//                 UC => col("FattyAcid").tag().unsaturation().alias("UC"),
//                 PUC => col("FattyAcid")
//                     .tag()
//                     .positional(
//                         |expr| expr.fa().unsaturated().sum(),
//                         PermutationOptions::default().map(true),
//                     )
//                     .alias("PUC"),
//                 SUC => col("FattyAcid")
//                     .tag()
//                     .map(|expr| expr.fa().unsaturated().sum())
//                     .alias("SUC"),
//             }
//             .alias(format!("Key{index}")),
//         );
//         // Value
//         lazy_frame = lazy_frame.with_column(
//             sum("Value")
//                 .over([as_struct(vec![col(format!("^Key[0-{index}]$"))])])
//                 .alias(format!("Value{index}")),
//         );
//     }
//     // Group
//     lazy_frame = lazy_frame
//         .group_by([col(r#"^Key\d$"#), col(r#"^Value\d$"#)])
//         .agg([as_struct(vec![col("Label"), col("FattyAcid"), col("Value")]).alias("Species")]);
//     lazy_frame = lazy_frame.select([
//         as_struct(vec![col(r#"^Key\d$"#)]).alias("Keys"),
//         concat_arr(vec![col(r#"^Value\d$"#)])?.alias("Values"),
//         col("Species"),
//     ]);
//     Ok(lazy_frame)
// }
fn compose(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    // Composition
    for (index, selection) in settings.confirmed.selections.iter().enumerate() {
        lazy_frame = lazy_frame.with_column(
            match selection.composition {
                MNC => col("FattyAcid")
                    .tag()
                    .mass(Some(lit(settings.confirmed.adduct)))
                    .map(
                        column(round(settings.confirmed.round_mass)),
                        GetOutput::same_type(),
                    )
                    .alias("MNC"),
                MSC => col("FattyAcid")
                    .tag()
                    .map(|expr| expr.fa().mass(None).round(settings.confirmed.round_mass))
                    .alias("MSC"),
                NNC => col("FattyAcid").tag().ecn().alias("NNC"),
                NSC => col("FattyAcid")
                    .tag()
                    .map(|expr| expr.fa().ecn())
                    .alias("NSC"),
                SNC => col("Label")
                    .tag()
                    .non_stereospecific(identity, PermutationOptions::default())?
                    .alias("SNC"),
                SPC => col("Label")
                    .tag()
                    .positional(identity, PermutationOptions::default())
                    .alias("SPC"),
                SSC => col("Label").alias("SSC"),
                TNC => col("FattyAcid")
                    .tag()
                    .non_stereospecific(
                        |expr| expr.fa().is_saturated(),
                        PermutationOptions::default().map(true),
                    )?
                    .alias("TNC"),
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
                UNC => col("FattyAcid").tag().unsaturation().alias("UNC"),
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
        concat_list([all()
            .exclude(["Keys", r#"^Value\d$"#])
            .arr()
            .get(lit(index as u32), false)])
    };
    for index in 0..settings.confirmed.selections.len() {
        lazy_frame = lazy_frame.with_column(
            as_struct(vec![
                values(index)?.list().mean().alias("Mean"),
                values(index)?
                    .list()
                    .std(settings.confirmed.ddof)
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

fn filter(mut lazy_frame: LazyFrame, settings: &Settings) -> LazyFrame {
    println!("lazy_frame: {}", lazy_frame.clone().collect().unwrap());
    if !settings.confirmed.show_filtered {
        let mut predicate = lit(true);
        for (index, selection) in settings.confirmed.selections.iter().enumerate() {
            // Key
            // for (key, value) in &group.filter.key {
            //     // match group.composition {
            //     //     SC => {
            //     //         let expr = col("Keys").struct_().field_by_index(index as _).tag();
            //     //         if value[0] {
            //     //             predicate = predicate
            //     //                 .and(expr.clone().sn1().neq(lit(LiteralValue::from(key.clone()))));
            //     //         }
            //     //         if value[1] {
            //     //             predicate = predicate
            //     //                 .and(expr.clone().sn2().neq(lit(LiteralValue::from(key.clone()))));
            //     //         }
            //     //         if value[2] {
            //     //             predicate = predicate
            //     //                 .and(expr.clone().sn3().neq(lit(LiteralValue::from(key.clone()))));
            //     //         }
            //     //     }
            //     //     PSC => {
            //     //         let expr = col("Keys").struct_().field_by_index(index as _).tag();
            //     //         predicate = predicate
            //     //             .and(expr.clone().sn1().neq(lit(LiteralValue::from(key.clone()))))
            //     //             .and(expr.clone().sn2().neq(lit(LiteralValue::from(key.clone()))));
            //     //     }
            //     //     // MC | ECNC | UC => {
            //     //     //     let expr = col("Keys").struct_().field_by_index(index as _);
            //     //     //     predicate = predicate.and(expr.neq(lit(LiteralValue::from(key.clone()))));
            //     //     // }
            //     //     _ => {
            //     //         // let expr = col("Keys")
            //     //         //     .struct_()
            //     //         //     .field_by_index(index as _)
            //     //         //     .triacylglycerol();
            //     //         // predicate = predicate.and(expr.sn1().neq(lit(key.to_string())));
            //     //     }
            //     // }
            //     let expr = col("Keys").struct_().field_by_index(index as _);
            //     match group.composition {
            //         MC => {
            //             predicate = predicate.and(expr.neq(lit(LiteralValue::from(key.clone()))));
            //         }
            //         _ => {
            //             let expr = expr.tag();
            //             if value[0] {
            //                 predicate = predicate
            //                     .and(expr.clone().sn1().neq(lit(LiteralValue::from(key.clone()))));
            //             }
            //             if value[1] {
            //                 predicate = predicate
            //                     .and(expr.clone().sn2().neq(lit(LiteralValue::from(key.clone()))));
            //             }
            //             if value[2] {
            //                 predicate = predicate
            //                     .and(expr.clone().sn3().neq(lit(LiteralValue::from(key.clone()))));
            //             }
            //         }
            //     }
            // }
            // Value
            let mut expr = col("Values").arr().get(lit(index as u32), false);
            if settings.index.is_none() {
                expr = expr.struct_().field_by_name("Mean");
            }
            // predicate = predicate.and(expr.gt(lit(selection.filter.value)));
        }
        lazy_frame = lazy_frame.filter(predicate);
    }
    lazy_frame
}

fn sort(mut lazy_frame: LazyFrame, settings: &Settings) -> LazyFrame {
    let mut sort_options = SortMultipleOptions::default();
    if let Order::Descending = settings.confirmed.order {
        sort_options = sort_options
            .with_order_descending(true)
            .with_nulls_last(true);
    }
    lazy_frame = match settings.confirmed.sort {
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

// /// Extension methods for [`DataFrame`]
// trait DataFrameExt {
//     fn cartesian_product(&self) -> PolarsResult<Triacylglycerol>;
// }

// impl DataFrameExt for DataFrame {
//     // TODO https://github.com/pola-rs/polars/issues/18587
//     fn cartesian_product(&self) -> PolarsResult<Triacylglycerol> {
//         let lazy_frame = self.clone().lazy().with_row_index("Index", None);
//         // Fatty acid with stereospecific number value
//         let fatty_acid = |name| {
//             as_struct(vec![
//                 col("Index"),
//                 col("FA"),
//                 col("Target")
//                     .struct_()
//                     .field_by_name("Calculated")
//                     .struct_()
//                     .field_by_name(name)
//                     .alias("Value"),
//             ])
//         };
//         Ok(Triacylglycerol(
//             lazy_frame
//                 .clone()
//                 .select([fatty_acid("DAG13").alias("SN1")])
//                 .cross_join(
//                     lazy_frame.clone().select([fatty_acid("MAG2").alias("SN2")]),
//                     None,
//                 )
//                 .cross_join(lazy_frame.select([fatty_acid("DAG13").alias("SN3")]), None)
//                 .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")]),
//         ))
//     }
//     // fn cartesian_product(&self) -> PolarsResult<Tags> {
//     //     let lazy_frame = self.0.clone().lazy().with_row_index("Index", None);
//     //     Ok(Tags(
//     //         lazy_frame
//     //             .clone()
//     //             .select([fatty_acid("DAG13.Calculated")?.alias("SN1")])
//     //             .cross_join(
//     //                 lazy_frame
//     //                     .clone()
//     //                     .select([fatty_acid("MAG2.Calculated")?.alias("SN2")]),
//     //                 None,
//     //             )
//     //             .cross_join(
//     //                 lazy_frame.select([fatty_acid("DAG13.Calculated")?.alias("SN3")]),
//     //                 None,
//     //             )
//     //             .select([as_struct(vec![col("SN1"), col("SN2"), col("SN3")]).alias("TAG")]),
//     //     ))
//     // }
// }

// fn sort_by_ecn() -> Expr {
//     col("").sort_by(
//         [col("").struct_().field_by_name("FA").fa().ecn()],
//         Default::default(),
//     )
// }

// fn sort_by_mass() -> Expr {
//     col("").sort_by(
//         [col("").struct_().field_by_name("FA").fa().mass(None)],
//         Default::default(),
//     )
// }

// fn sort_by_type() -> Expr {
//     col("").sort_by(
//         [col("").struct_().field_by_name("FA").fa().saturated()],
//         Default::default(),
//     )
// }

// fn sort_by_species() -> Expr {
//     col("").sort_by(
//         [
//             col("")
//                 .struct_()
//                 .field_by_name("FA")
//                 .struct_()
//                 .field_by_name("Carbons"),
//             col("")
//                 .struct_()
//                 .field_by_name("FA")
//                 .struct_()
//                 .field_by_name("Doubles")
//                 .list()
//                 .len(),
//             col("")
//                 .struct_()
//                 .field_by_name("FA")
//                 .struct_()
//                 .field_by_name("Triples")
//                 .list()
//                 .len(),
//             col("")
//                 .struct_()
//                 .field_by_name("FA")
//                 .struct_()
//                 .field_by_name("Label"),
//             col("").struct_().field_by_name("Index"),
//         ],
//         Default::default(),
//     )
// }

// fn sort_by_unsaturation() -> Expr {
//     col("").sort_by(
//         [col("").struct_().field_by_name("FA").fa().unsaturation()],
//         Default::default(),
//     )
// }

// // Triacylglycerol species
// fn species() -> Expr {
//     concat_str(
//         [col("TAG")
//             .struct_()
//             // .field_by_name("FA")
//             // .struct_()
//             .field_by_names([r#"^SN[1-3]$"#])
//             .fa()
//             .species()],
//         "",
//         true,
//     )
// }

// // Triacylglycerol value
// fn value() -> Expr {
//     col("TAG")
//         .struct_()
//         .field_by_name("SN1")
//         .struct_()
//         .field_by_name("Value")
//         * col("TAG")
//             .struct_()
//             .field_by_name("SN2")
//             .struct_()
//             .field_by_name("Value")
//         * col("TAG")
//             .struct_()
//             .field_by_name("SN3")
//             .struct_()
//             .field_by_name("Value")
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
//                 Sn::.sn1.One => discrimination.get(&index),
//                 Sn::.sn2.Two => discrimination.get(&index),
//                 Sn::.sn3.Three => discrimination.get(&index),
//             }
//             .map_or(value, |&f| f * value)
//         })
//         .normalized()
// }
