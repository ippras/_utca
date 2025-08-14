use crate::{
    app::panes::composition::settings::{Filter, Order, Selection, Settings, Sort},
    special::composition::{MMC, MSC, NMC, NSC, SMC, SPC, SSC, TMC, TPC, TSC, UMC, USC},
    utils::{Hashed, hash},
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use metadata::MetaDataFrame;
use polars::prelude::*;
use std::{
    convert::identity,
    hash::{Hash, Hasher},
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
        // let mut settings = key.settings.clone();
        // if settings.special.selections.is_empty() {
        //     settings.special.selections.push_back(Selection {
        //         composition: SSC,
        //         filter: Filter::new(),
        //     });
        // }
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        lazy_frame = compute(lazy_frame, key.settings)?;
        // let mut lazy_frame = match settings.index {
        //     Some(index) => {
        //         let frame = &key.frames[index];
        //         let mut lazy_frame = frame.data.clone().lazy();
        //         lazy_frame = compute(lazy_frame, settings)?;
        //         lazy_frame
        //     }
        //     None => {
        //         let compute = |frame: &MetaDataFrame| -> PolarsResult<LazyFrame> {
        //             Ok(compute(frame.data.clone().lazy(), settings)?.select([
        //                 hash(col("Keys")),
        //                 col("Keys"),
        //                 col("Values").alias(frame.meta.format(".").to_string()),
        //             ]))
        //         };
        //         let mut lazy_frame = compute(&key.frames[0])?;
        //         for frame in &key.frames[1..] {
        //             lazy_frame = lazy_frame.join(
        //                 compute(frame)?,
        //                 [col("Hash"), col("Keys")],
        //                 [col("Hash"), col("Keys")],
        //                 JoinArgs::new(JoinType::Full).with_coalesce(JoinCoalesce::CoalesceColumns),
        //             );
        //         }
        //         lazy_frame = lazy_frame.drop(by_name(["Hash"], true));
        //         lazy_frame = mean_and_standard_deviation(lazy_frame, settings)?;
        //         lazy_frame
        //     }
        // };
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
    // pub(crate) frames: &'a Hashed<Vec<MetaDataFrame>>,
    pub(crate) data_frame: &'a Hashed<DataFrame>,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash(state);
        self.settings.index.hash(state);
        self.settings.special.hash(state);
    }
}

/// Composition value
type Value = DataFrame;

fn compute(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    println!("lazy_frame0: {:?}", lazy_frame.clone().collect().unwrap());
    // Compose
    lazy_frame = compose(lazy_frame, settings)?;
    println!("lazy_frame1: {:?}", lazy_frame.clone().collect().unwrap());
    // // Filter
    // lazy_frame = filter(lazy_frame, settings);
    // Sort
    lazy_frame = sort(lazy_frame, settings);
    Ok(lazy_frame)
}

fn compose(mut lazy_frame: LazyFrame, settings: &Settings) -> PolarsResult<LazyFrame> {
    // Composition
    for (index, selection) in settings.special.selections.iter().enumerate() {
        lazy_frame = lazy_frame.with_column(
            match selection.composition {
                MMC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .mass(Some(lit(settings.special.adduct)))
                    .round(settings.special.round_mass, RoundMode::HalfToEven)
                    .alias("MMC"),
                MSC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map_expr(|expr| {
                        expr.fatty_acid()
                            .mass(None)
                            .round(settings.special.round_mass, RoundMode::HalfToEven)
                    })
                    .alias("MSC"),
                NMC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .equivalent_carbon_number()
                    .alias("NMC"),
                NSC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map_expr(|expr| expr.fatty_acid().equivalent_carbon_number())
                    .alias("NSC"),
                SMC => col(LABEL)
                    .triacylglycerol()
                    .non_stereospecific(identity)
                    .alias("SMC"),
                SPC => col(LABEL)
                    .triacylglycerol()
                    .positional(identity)
                    .alias("SPC"),
                SSC => col(LABEL).alias("SSC"),
                TMC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .non_stereospecific(|expr| expr.fatty_acid().is_saturated().not())
                    .triacylglycerol()
                    .map_expr(|expr| expr.fatty_acid().r#type())
                    .alias("TMC"),
                TPC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .positional(|expr| expr.fatty_acid().is_saturated().not())
                    .triacylglycerol()
                    .map_expr(|expr| expr.fatty_acid().r#type())
                    .alias("TPC"),
                TSC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map_expr(|expr| expr.fatty_acid().r#type())
                    .alias("TSC"),
                UMC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .unsaturation()
                    .alias("UMC"),
                USC => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map_expr(|expr| expr.fatty_acid().unsaturation())
                    .alias("USC"),
            }
            .alias(format!("Key{index}")),
        );
        let predicate = DataTypeExpr::OfExpr(Box::new(col("Value")))
            .equals(DataTypeExpr::Literal(DataType::Float64));
        lazy_frame = lazy_frame.with_column(
            ternary_expr(
                predicate,
                sum("Value"),
                col("Value").struct_().field_by_name("name"),
            )
            .over([as_struct(vec![col(format!("^Key[0-{index}]$"))])])
            .alias(format!("Value{index}")),
        );
    }
    // Group
    lazy_frame = lazy_frame
        .group_by([col(r#"^Key\d$"#), col(r#"^Value\d$"#)])
        .agg([as_struct(vec![col("Label"), col(TRIACYLGLYCEROL), col("Value")]).alias("Species")]);
    lazy_frame = lazy_frame.select([
        as_struct(vec![col(r#"^Key\d$"#)]).alias("Keys"),
        concat_arr(vec![col(r#"^Value\d$"#)])?.alias("Values"),
        col("Species"),
    ]);
    Ok(lazy_frame)
}

fn mean_and_standard_deviation(
    mut lazy_frame: LazyFrame,
    settings: &Settings,
) -> PolarsResult<LazyFrame> {
    // TODO [array_get?](https://docs.rs/polars/latest/polars/prelude/array/trait.ArrayNameSpace.html)
    let list = |index| {
        // TODO: .arr().to_list().list() for compute mean std with None
        concat_list([all()
            .exclude_cols(["Keys", r#"^Value\d$"#])
            .as_expr()
            .arr()
            .get(lit(index as u32), true)])
    };
    for index in 0..settings.special.selections.len() {
        lazy_frame = lazy_frame.with_column(
            as_struct(vec![
                list(index)?.list().mean().alias("Mean"),
                list(index)?
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
                    .eval(col("").struct_().field_by_name("Mean"));
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

pub(super) mod filtered;
pub(super) mod indices;
pub(super) mod species;
pub(super) mod unique;
