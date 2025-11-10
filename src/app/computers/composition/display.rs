use crate::{
    app::states::composition::{
        Composition, ECN_MONO, ECN_STEREO, MASS_MONO, MASS_STEREO, SPECIES_MONO,
        SPECIES_POSITIONAL, SPECIES_STEREO, TYPE_MONO, TYPE_POSITIONAL, TYPE_STEREO,
        UNSATURATION_MONO, UNSATURATION_STEREO,
    },
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::ExprIfExt as _;
use std::{
    hash::{Hash, Hasher},
    iter::once,
};

/// Display composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Display composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.data_frame.data_frame.clone().lazy();
        // println!("Display 0: {}", lazy_frame.clone().collect().unwrap());
        // Restructure
        let exprs = key
            .compositions
            .iter()
            .enumerate()
            .map(|(index, composition)| {
                as_struct(vec![
                    col("Keys")
                        .struct_()
                        .field_by_index(index as _)
                        .alias("Key"),
                    col("Values")
                        .arr()
                        .get(lit(index as u32), false)
                        .alias("Value"),
                ])
                .alias(index.to_string())
            })
            .chain(once(col("Species")))
            .collect::<Vec<_>>();
        lazy_frame = lazy_frame.select(exprs);
        // println!("Display 2: {}", lazy_frame.clone().collect().unwrap());
        // Форматирование
        let species = col("Species").list().eval(as_struct(vec![
            {
                let label = || col("").struct_().field_by_name(LABEL);
                format_str(
                    "[{}; {}; {}]",
                    [
                        label().triacylglycerol().stereospecific_number1(),
                        label().triacylglycerol().stereospecific_number2(),
                        label().triacylglycerol().stereospecific_number3(),
                    ],
                )?
                .alias(LABEL)
            },
            {
                let triacylglycerol = || col("").struct_().field_by_name(TRIACYLGLYCEROL);
                format_str(
                    "[{}; {}; {}]",
                    [
                        triacylglycerol()
                            .triacylglycerol()
                            .stereospecific_number1()
                            .fatty_acid()
                            .format(),
                        triacylglycerol()
                            .triacylglycerol()
                            .stereospecific_number2()
                            .fatty_acid()
                            .format(),
                        triacylglycerol()
                            .triacylglycerol()
                            .stereospecific_number3()
                            .fatty_acid()
                            .format(),
                    ],
                )?
                .alias(TRIACYLGLYCEROL)
            },
            col("")
                .struct_()
                .field_by_name("Value")
                .struct_()
                .field_by_name("Mean")
                .percent_if(key.percent)
                .alias("Value"),
        ]));
        let exprs =
            key.compositions
                .iter()
                .enumerate()
                .map(|(index, composition)| {
                    // Value
                    let percent = |name| {
                        col(index.to_string())
                            .struct_()
                            .field_by_name("Value")
                            .struct_()
                            .field_by_name(name)
                            .percent_if(key.percent)
                    };
                    let value = as_struct(vec![
                        percent("Mean"),
                        percent("StandardDeviation"),
                        percent("Array"),
                    ]);
                    // Key
                    let key = || col(index.to_string()).struct_().field_by_name("Key");
                    let key = match *composition {
                        ECN_MONO | MASS_MONO | UNSATURATION_MONO => format_str("({})", [key()])?,
                        SPECIES_MONO | TYPE_MONO => format_str(
                            "({}; {}; {})", // { 1, 2, 3: {}, {}, {}}
                            [
                                key().triacylglycerol().stereospecific_number1(),
                                key().triacylglycerol().stereospecific_number2(),
                                key().triacylglycerol().stereospecific_number3(),
                            ],
                        )?,
                        ECN_STEREO | MASS_STEREO | SPECIES_STEREO | TYPE_STEREO
                        | UNSATURATION_STEREO => format_str(
                            "[{}; {}; {}]", // { 1: {}; 2: {}; 3: {}}
                            [
                                key().triacylglycerol().stereospecific_number1(),
                                key().triacylglycerol().stereospecific_number2(),
                                key().triacylglycerol().stereospecific_number3(),
                            ],
                        )?,
                        SPECIES_POSITIONAL | TYPE_POSITIONAL => format_str(
                            "{{}; {}; {}}",
                            [
                                key().triacylglycerol().stereospecific_number1(),
                                key().triacylglycerol().stereospecific_number2(),
                                key().triacylglycerol().stereospecific_number3(),
                            ],
                        )?,
                    };
                    Ok(as_struct(vec![key.alias("Key"), value.alias("Value")])
                        .alias(index.to_string()))
                })
                .chain(once(Ok(species)))
                .collect::<PolarsResult<Vec<_>>>()?;
        lazy_frame = lazy_frame.select(exprs);
        // println!("Display 3: {}", lazy_frame.clone().collect().unwrap());
        // when(col("Keys"))
        //     .then(col("Keys"))
        //     .when(col("Keys"))
        //     .then(col("Keys"));
        let data_frame = lazy_frame.collect()?;
        Ok(data_frame)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Display composition key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: &'a HashedDataFrame,
    pub(crate) kind: Kind,
    pub(crate) percent: bool,
    pub(crate) compositions: &'a [Composition],
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.data_frame.hash.hash(state);
        self.kind.hash(state);
        self.percent.hash(state);
    }
}

/// Display composition value
type Value = DataFrame;

/// Display kind
#[derive(Clone, Copy, Debug, Hash)]
pub enum Kind {
    Enrichment,
    Selectivity,
}

fn format(column: Column) -> PolarsResult<Option<Column>> {
    // match column.dtype() {
    //     DataType::UInt64 => todo!(),
    //     DataType::Float64 => todo!(),
    //     DataType::Decimal(_, _) => todo!(),
    //     DataType::String => todo!(),
    //     DataType::Binary => todo!(),
    //     DataType::BinaryOffset => todo!(),
    //     DataType::Date => todo!(),
    //     DataType::Datetime(time_unit, time_zone) => todo!(),
    //     DataType::Duration(time_unit) => todo!(),
    //     DataType::Time => todo!(),
    //     DataType::Array(data_type, _) => todo!(),
    //     DataType::List(data_type) => todo!(),
    //     DataType::Null => todo!(),
    //     DataType::Categorical(categories, categorical_mapping) => todo!(),
    //     DataType::Enum(frozen_categories, categorical_mapping) => todo!(),
    //     DataType::Struct(fields) => todo!(),
    //     _ => column,
    // }
    let fields = column
        .struct_()?
        .fields_as_series()
        .into_iter()
        .map(|series| {
            series
                .iter()
                .map(|any_value| any_value.str_value())
                .collect::<StringChunked>()
                .into_series()
                .with_name(series.name().clone())
        })
        .collect::<Vec<_>>();
    Ok(Some(
        StructChunked::from_series(PlSmallStr::EMPTY, column.len(), fields.iter())?.into_column(),
    ))
}

fn calculation(key: Key) -> PolarsResult<Expr> {
    match key.kind {
        Kind::Enrichment => format_str(
            "{} / {}",
            [
                col(STEREOSPECIFIC_NUMBERS2)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .percent_if(key.percent),
                col(STEREOSPECIFIC_NUMBERS123)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .percent_if(key.percent),
            ],
        ),
        Kind::Selectivity => format_str(
            "({} / {}) / ({} / {})",
            [
                col(STEREOSPECIFIC_NUMBERS2)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .percent_if(key.percent),
                col(STEREOSPECIFIC_NUMBERS123)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .percent_if(key.percent),
                col(STEREOSPECIFIC_NUMBERS2)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                    .sum()
                    .percent_if(key.percent),
                col(STEREOSPECIFIC_NUMBERS123)
                    .struct_()
                    .field_by_name("Experimental")
                    .struct_()
                    .field_by_name("Mean")
                    .filter(col(FATTY_ACID).fatty_acid().is_unsaturated(None))
                    .sum()
                    .percent_if(key.percent),
            ],
        ),
    }
}
