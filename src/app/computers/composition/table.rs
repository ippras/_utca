use crate::{
    app::states::composition::{
        ECN_MONO, ECN_STEREO, MASS_MONO, MASS_STEREO, SPECIES_MONO, SPECIES_POSITIONAL,
        SPECIES_STEREO, Selection, Settings, TYPE_MONO, TYPE_POSITIONAL, TYPE_STEREO,
        UNSATURATION_MONO, UNSATURATION_STEREO,
    },
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::prelude::*;
use std::iter::once;

/// Table composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Table composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        // ┌─────────────────┬────────────────────────────────┬───────────────────────────────────────────────┐
        // │ Keys            ┆ Values                         ┆ Species                                       │
        // │ ---             ┆ ---                            ┆ ---                                           │
        // │ struct[1]       ┆ array[struct[3], n]            ┆ list[struct[3]]                               │
        // ╞═════════════════╪════════════════════════════════╪═══════════════════════════════════════════════╡
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // println!("display 0: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = format(lazy_frame, key)?;
        let data_frame = lazy_frame.collect()?;
        Ok(data_frame)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Table composition key
#[derive(Clone, Hash, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) selections: &'a Vec<Selection>,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(data_frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame: data_frame,
            percent: settings.percent,
            precision: settings.precision,
            selections: &settings.selections,
            significant: settings.significant,
        }
    }
}

/// Table composition value
type Value = DataFrame;

fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    // Restructure
    lazy_frame = lazy_frame.select(
        (0..key.selections.len())
            .map(|index| {
                as_struct(vec![
                    col("Keys")
                        .struct_()
                        .field_by_index(index as _)
                        .alias("Key"),
                    col("Values")
                        .arr()
                        .get(lit(index as IdxSize), false)
                        .alias("Value"),
                ])
                .alias(index.to_string())
            })
            .chain(once(col("Species")))
            .collect::<Vec<_>>(),
    );
    // Format
    let species = col("Species").list().eval(as_struct(vec![
        {
            let label = element().struct_().field_by_name(LABEL).triacylglycerol();
            format_str(
                "[{}; {}; {}]",
                [
                    label.clone().stereospecific_number1(),
                    label.clone().stereospecific_number2(),
                    label.stereospecific_number3(),
                ],
            )?
            .alias(LABEL)
        },
        {
            let triacylglycerol = {
                element()
                    .struct_()
                    .field_by_name(TRIACYLGLYCEROL)
                    .triacylglycerol()
            };
            format_str(
                "[{}; {}; {}]",
                [
                    triacylglycerol
                        .clone()
                        .stereospecific_number1()
                        .fatty_acid()
                        .format(),
                    triacylglycerol
                        .clone()
                        .stereospecific_number2()
                        .fatty_acid()
                        .format(),
                    triacylglycerol
                        .stereospecific_number3()
                        .fatty_acid()
                        .format(),
                ],
            )?
            .alias(TRIACYLGLYCEROL)
        },
        element()
            .struct_()
            .field_by_name("Value")
            .struct_()
            .field_by_name("Mean")
            .percent(key.percent)
            .precision(key.precision, key.significant)
            .alias("Value"),
    ]));
    let exprs = key
        .selections
        .iter()
        .enumerate()
        .map(|(index, selection)| {
            // Value
            let field = |name| {
                col(index.to_string())
                    .struct_()
                    .field_by_name("Value")
                    .struct_()
                    .field_by_name(name)
            };
            let value = as_struct(vec![
                field("Mean")
                    .percent(key.percent)
                    .precision(key.precision, key.significant),
                field("StandardDeviation")
                    .percent(key.percent)
                    .precision(key.precision, key.significant),
                field("Array").arr().eval(
                    element()
                        .percent(key.percent)
                        .precision(key.precision, key.significant),
                    false,
                ),
            ]);
            // Key
            let key = col(index.to_string())
                .struct_()
                .field_by_name("Key")
                .triacylglycerol();
            let args = [
                key.clone().stereospecific_number1(),
                key.clone().stereospecific_number2(),
                key.stereospecific_number3(),
            ];
            let key = match selection.composition {
                ECN_STEREO | MASS_STEREO | SPECIES_STEREO | TYPE_STEREO | UNSATURATION_STEREO => {
                    format_str("[{}; {}; {}]", args)?
                }
                SPECIES_POSITIONAL | TYPE_POSITIONAL => format_str("[{}/2; {}; {}/2]", args)?,
                ECN_MONO | MASS_MONO | SPECIES_MONO | TYPE_MONO | UNSATURATION_MONO => {
                    format_str("[{}/3; {}/3; {}/3]", args)?
                }
            };
            Ok(as_struct(vec![key.alias("Key"), value.alias("Value")]).alias(index.to_string()))
        })
        .chain(once(Ok(species)))
        .collect::<PolarsResult<Vec<_>>>()?;
    Ok(lazy_frame.select(exprs))
}
