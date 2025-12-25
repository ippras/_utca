use crate::{
    app::states::composition::{
        Composition, ECN_MONO, ECN_STEREO, MASS_MONO, MASS_STEREO, Order, SPECIES_MONO,
        SPECIES_POSITIONAL, SPECIES_STEREO, Settings, Sort, TYPE_MONO, TYPE_POSITIONAL,
        TYPE_STEREO, UNSATURATION_MONO, UNSATURATION_STEREO,
    },
    r#const::{KEY, KEYS, SPECIES, VALUE, VALUES},
    utils::HashedDataFrame,
};
use const_format::formatcp;
use egui::{
    emath::OrderedFloat,
    util::cache::{ComputerMut, FrameCache},
};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::eval_arr;
use std::{convert::identity, sync::LazyLock};
use tracing::instrument;

/// Starts with `KEY`
const KEY_: &str = formatcp!(r#"^{KEY}.*$"#);

/// Starts with `VALUE`
const VALUE_: &str = formatcp!(r#"^{VALUE}.*$"#);

const SCHEMA: LazyLock<SchemaRef> = LazyLock::new(|| {
    Arc::new(Schema::from_iter([
        field!(LABEL[DataType::String]),
        field!(TRIACYLGLYCEROL),
        Field::new(
            PlSmallStr::from_static(VALUE),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
    ]))
});

/// Composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        schema(key.frame)?;
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        lazy_frame = compute(lazy_frame, key)?;
        lazy_frame.collect()
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
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) index: Option<usize>,
    pub(crate) adduct: OrderedFloat<f64>,
    pub(crate) ddof: u8,
    pub(crate) order: Order,
    pub(crate) round_mass: u32,
    pub(crate) compositions: &'a Vec<Composition>,
    pub(crate) sort: Sort,
}

impl<'a> Key<'a> {
    pub(crate) fn new(data_frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame: data_frame,
            index: settings.index,
            adduct: OrderedFloat(settings.adduct),
            ddof: settings.ddof,
            order: settings.order,
            round_mass: settings.round_mass,
            compositions: &settings.compositions,
            sort: settings.sort,
        }
    }
}

/// Composition value
type Value = DataFrame;

fn schema(data_frame: &DataFrame) -> PolarsResult<()> {
    let _cast = data_frame.schema().matches_schema(&SCHEMA)?;
    Ok(())
}

fn compute(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    println!("OG 0: {}", lazy_frame.clone().collect().unwrap());
    // Compose
    lazy_frame = compose(lazy_frame, key)?;
    // Sort
    lazy_frame = sort(lazy_frame, key);
    Ok(lazy_frame)
}

fn compose(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // eval_arr(col(VALUE).arr().eval(element().cum_count(false), false), |expr| )?;
    println!("OG 1: {}", lazy_frame.clone().collect().unwrap());
    // Composition
    for (index, composition) in key.compositions.iter().enumerate() {
        lazy_frame = lazy_frame.with_column(
            match *composition {
                MASS_MONO => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map(|_| {
                        col(TRIACYLGLYCEROL)
                            .triacylglycerol()
                            .relative_atomic_mass(Some(lit(key.adduct.0)))
                            .round(key.round_mass, RoundMode::HalfToEven)
                    })
                    .alias("MMC"),
                MASS_STEREO => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map(|expr| {
                        expr.fatty_acid()
                            .relative_atomic_mass(None)
                            .round(key.round_mass, RoundMode::HalfToEven)
                    })
                    .alias("MSC"),
                ECN_MONO => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map(|_| {
                        col(TRIACYLGLYCEROL)
                            .triacylglycerol()
                            .equivalent_carbon_number()
                    })
                    .alias("NMC"),
                ECN_STEREO => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map(|expr| expr.fatty_acid().equivalent_carbon_number())
                    .alias("NSC"),
                SPECIES_MONO => col(LABEL)
                    .triacylglycerol()
                    .non_stereospecific(identity)
                    .alias("SMC"),
                SPECIES_POSITIONAL => col(LABEL)
                    .triacylglycerol()
                    .positional(identity)
                    .alias("SPC"),
                SPECIES_STEREO => col(LABEL).alias("SSC"),
                TYPE_MONO => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .non_stereospecific(|expr| expr.fatty_acid().is_saturated().not())
                    .triacylglycerol()
                    .map(|expr| expr.fatty_acid().r#type())
                    .alias("TMC"),
                TYPE_POSITIONAL => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .positional(|expr| expr.fatty_acid().is_saturated().not())
                    .triacylglycerol()
                    .map(|expr| expr.fatty_acid().r#type())
                    .alias("TPC"),
                TYPE_STEREO => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map(|expr| expr.fatty_acid().r#type())
                    .alias("TSC"),
                UNSATURATION_MONO => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map(|_| col(TRIACYLGLYCEROL).triacylglycerol().unsaturation())
                    .alias("UMC"),
                UNSATURATION_STEREO => col(TRIACYLGLYCEROL)
                    .triacylglycerol()
                    .map(|expr| expr.fatty_acid().unsaturation())
                    .alias("USC"),
            }
            .alias(format!("Key{index}")),
        );
        // let expr = as_struct(vec![
        //     array.clone().arr().mean().alias(MEAN),
        //     array.clone().arr().std(key.ddof).alias(STANDARD_DEVIATION),
        //     array.alias(SAMPLE),
        // ])
        lazy_frame = lazy_frame.with_column(
            eval_arr(col(VALUE), |expr| expr.sum())?
                .over([as_struct(vec![col(format!("^Key[0-{index}]$"))])])
                .alias(format!("Value{index}")),
        );
    }
    println!("OG 2: {}", lazy_frame.clone().collect().unwrap());
    // Group
    lazy_frame = lazy_frame
        .group_by([col(r#"^Key\d$"#), col(r#"^Value\d$"#)])
        .agg([as_struct(vec![col(LABEL), col(TRIACYLGLYCEROL), col(VALUE)]).alias(SPECIES)]);
    println!("OG 3: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = lazy_frame.select([
        as_struct(vec![col(r#"^Key\d$"#)]).alias(KEYS),
        concat_list(vec![col(r#"^Value\d$"#)])?.alias(VALUES),
        col(SPECIES),
    ]);
    println!("OG 4: {}", lazy_frame.clone().collect().unwrap());
    Ok(lazy_frame)
}

fn sort(mut lazy_frame: LazyFrame, key: Key) -> LazyFrame {
    let mut sort_options = SortMultipleOptions::default();
    if let Order::Descending = key.order {
        sort_options = sort_options
            .with_maintain_order(true)
            .with_order_descending(true)
            .with_nulls_last(true);
    }
    lazy_frame = match key.sort {
        Sort::Key => lazy_frame.sort_by_exprs([col(KEYS)], sort_options),
        Sort::Value => {
            let mut expr = col(VALUES);
            if key.index.is_none() {
                expr = expr.list().eval(element().arr().mean());
            }
            lazy_frame.sort_by_exprs([expr], sort_options)
        }
    };
    // Sort species by value
    lazy_frame = lazy_frame.with_columns([col(SPECIES).list().eval(element().sort_by(
        [element().struct_().field_by_name(VALUE)],
        SortMultipleOptions {
            descending: vec![true],
            nulls_last: vec![true],
            ..Default::default()
        },
    ))]);
    lazy_frame
}

// fn mean_and_standard_deviation(
//     mut lazy_frame: LazyFrame,
//     settings: &Settings,
// ) -> PolarsResult<LazyFrame> {
//     // TODO [array_get?](https://docs.rs/polars/latest/polars/prelude/array/trait.ArrayNameSpace.html)
//     let list = |index| {
//         // TODO: .arr().to_list().list() for compute mean std with None
//         concat_list([all()
//             .exclude_cols([KEYS, r#"^Value\d$"#])
//             .as_expr()
//             .arr()
//             .get(lit(index as u32), true)])
//     };
//     for index in 0..settings.special.selections.len() {
//         lazy_frame = lazy_frame.with_column(
//             as_struct(vec![
//                 list(index)?.list().mean().alias(MEAN),
//                 list(index)?
//                     .list()
//                     .std(settings.special.ddof)
//                     .alias(STANDARD_DEVIATION),
//             ])
//             .alias(format!("Value{index}")),
//         );
//     }
//     // Group
//     lazy_frame = lazy_frame.select([
//         col(KEYS),
//         concat_arr(vec![col(r#"^Value\d$"#)])?.alias(VALUES),
//     ]);
//     Ok(lazy_frame)
// }

pub(crate) mod filtered;
pub(crate) mod species;
pub(crate) mod sum;
pub(crate) mod table;
pub(crate) mod unique;
