use super::Mode;
use crate::{
    app::panes::composition::settings::{
        Filter, MMC, MSC, NMC, NSC, Order, SMC, SPC, SSC, Selection, Settings, Sort, TMC, TPC, TSC,
        UMC, USC,
    },
    utils::Hashed,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use std::{
    convert::identity,
    hash::{Hash, Hasher},
    sync::LazyLock,
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
        let data_frame = key.data_frame;
        let mode = mode(data_frame)?;
        let mut settings = key.settings.clone();
        if settings.special.selections.is_empty() {
            settings.special.selections.push_back(Selection {
                composition: SSC,
                filter: Filter::new(),
            });
        }
        let mut lazy_frame = key.data_frame.value.clone().lazy();
        lazy_frame = compute(lazy_frame, mode, settings.special.ddof, &settings)?;
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
    pub(crate) data_frame: &'a Hashed<DataFrame>,
    pub(crate) settings: &'a Settings,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash(state);
        self.settings.special.hash(state);
    }
}

/// Composition value
type Value = DataFrame;

fn mode(data_frame: &DataFrame) -> PolarsResult<Mode> {
    const ONE: LazyLock<Schema> = LazyLock::new(|| {
        Schema::from_iter([
            field!(LABEL[DataType::String]),
            field!(TRIACYLGLYCEROL),
            Field::new(PlSmallStr::from_static("Value"), DataType::Float64),
        ])
    });

    const MANY: LazyLock<Schema> = LazyLock::new(|| {
        Schema::from_iter([
            field!(LABEL[DataType::String]),
            field!(TRIACYLGLYCEROL),
            Field::new(
                PlSmallStr::from_static("Value"),
                DataType::Struct(vec![
                    Field::new(PlSmallStr::from_static("Mean"), DataType::Float64),
                    Field::new(
                        PlSmallStr::from_static("StandardDeviation"),
                        DataType::Float64,
                    ),
                    Field::new(
                        PlSmallStr::from_static("Repetitions"),
                        DataType::Array(Box::new(DataType::Float64), 0),
                    ),
                ]),
            ),
        ])
    });

    // if schema.matches_schema(&ONE).is_ok_and(|cast| !cast) {
    //     Ok(1)
    // } else if schema.matches_schema(&MANY).is_ok_and(|cast| !cast) {
    //     schema.get("Value");
    //     Ok(2)
    // } else {
    //     Err(
    //         polars_err!(SchemaMismatch: "Invalid composition schema: expected [`{ONE:?}`, `{MANY:?}`], got = `{schema:?}`"),
    //     )
    // }
    let schema = data_frame.schema();
    if let Some(label) = schema.get(LABEL)
        && *label == data_type!([DataType::String])
        && let Some(triacylglycerol) = schema.get(TRIACYLGLYCEROL)
        && *triacylglycerol == data_type!(TRIACYLGLYCEROL)
        && let Some(value) = schema.get("Value")
    {
        if *value == DataType::Float64 {
            return Ok(Mode::One);
        } else if let DataType::Struct(fields) = value
            && let [field1, field2, field3] = &**fields
            && field1.name == "Mean"
            && field1.dtype == DataType::Float64
            && field2.name == "StandardDeviation"
            && field2.dtype == DataType::Float64
            && field3.name == "Repetitions"
            && let DataType::Array(box DataType::Float64, length) = field3.dtype
        {
            return Ok(Mode::Many(length as _));
        }
    }
    polars_bail!(SchemaMismatch: "Invalid composition schema: expected [`{ONE:?}`, `{MANY:?}`], got = `{schema:?}`");
}

fn compute(
    mut lazy_frame: LazyFrame,
    mode: Mode,
    ddof: u8,
    settings: &Settings,
) -> PolarsResult<LazyFrame> {
    // Compose
    lazy_frame = compose(lazy_frame, mode, ddof, settings)?;
    // // Filter
    // lazy_frame = filter(lazy_frame, settings);
    // Sort
    lazy_frame = sort(lazy_frame, settings);
    Ok(lazy_frame)
}

fn compose(
    mut lazy_frame: LazyFrame,
    mode: Mode,
    ddof: u8,
    settings: &Settings,
) -> PolarsResult<LazyFrame> {
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
        // TODO
        // col("Value").meta().output_name();
        // let predicate = DataTypeExpr::OfExpr(Box::new(col("Value")))
        //     .equals(DataTypeExpr::Literal(DataType::Float64));
        // println!("predicate: {predicate:?}");
        // let expr = when(
        //     DataTypeExpr::OfExpr(Box::new(col("Value")))
        //         .equals(DataTypeExpr::Literal(DataType::Float64)),
        // )
        // .then(sum("Value").over([as_struct(vec![col(format!("^Key[0-{index}]$"))])]))
        // .otherwise(
        //     as_struct(vec![
        //         lit(1f64).alias("Mean"),
        //         lit(1f64).alias("StandardDeviation"),
        //         concat_arr(vec![lit(1f64)])?.alias("Repetitions"),
        //     ])
        //     .over([as_struct(vec![col(format!("^Key[0-{index}]$"))])]),
        // );
        let expr = match mode {
            Mode::One => sum("Value"),
            Mode::Many(length) => {
                // let array = || {
                //     col("Value")
                //         .struct_()
                //         .field_by_name("Repetitions")
                //         .arr()
                //         .to_list()
                //         .list()
                //         .eval(col("").sum())
                //         .list()
                //         .to_array(length as _).over([as_struct(vec![col(format!("^Key[0-{index}]$"))])])
                // };
                // as_struct(vec![
                //     array().arr().mean().alias("Mean"),
                //     array().arr().std(ddof).alias("StandardDeviation"),
                //     array().alias("Repetitions"),
                // ])
                let array = || {
                    concat_arr(
                        (0..length)
                            .map(|index| {
                                col("Value")
                                    .struct_()
                                    .field_by_name("Repetitions")
                                    .arr()
                                    .get(lit(index), false)
                                    .sum()
                            })
                            .collect(),
                    )
                };
                as_struct(vec![
                    array()?.arr().mean().alias("Mean"),
                    array()?.arr().std(ddof).alias("StandardDeviation"),
                    array()?.alias("Repetitions"),
                ])
            }
        }
        .over([as_struct(vec![col(format!("^Key[0-{index}]$"))])])
        .alias(format!("Value{index}"));
        lazy_frame = lazy_frame.with_column(expr);
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
    // Sort species by value
    lazy_frame = lazy_frame.with_columns([col("Species").list().eval(col("").sort_by(
        [col("").struct_().field_by_name("Value")],
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
//             .exclude_cols(["Keys", r#"^Value\d$"#])
//             .as_expr()
//             .arr()
//             .get(lit(index as u32), true)])
//     };
//     for index in 0..settings.special.selections.len() {
//         lazy_frame = lazy_frame.with_column(
//             as_struct(vec![
//                 list(index)?.list().mean().alias("Mean"),
//                 list(index)?
//                     .list()
//                     .std(settings.special.ddof)
//                     .alias("StandardDeviation"),
//             ])
//             .alias(format!("Value{index}")),
//         );
//     }
//     // Group
//     lazy_frame = lazy_frame.select([
//         col("Keys"),
//         concat_arr(vec![col(r#"^Value\d$"#)])?.alias("Values"),
//     ]);
//     Ok(lazy_frame)
// }

pub(super) mod filtered;
pub(super) mod indices;
pub(super) mod species;
pub(super) mod unique;
