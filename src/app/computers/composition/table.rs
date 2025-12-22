use crate::{
    app::states::composition::{
        ECN_MONO, ECN_STEREO, MASS_MONO, MASS_STEREO, SPECIES_MONO, SPECIES_POSITIONAL,
        SPECIES_STEREO, Selection, Settings, TYPE_MONO, TYPE_POSITIONAL, TYPE_STEREO,
        UNSATURATION_MONO, UNSATURATION_STEREO,
    },
    r#const::{KEY, KEYS, MEAN, SAMPLE, SPECIES, STANDARD_DEVIATION, VALUE, VALUES},
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
        println!("T0: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = format(lazy_frame, key)?;
        let data_frame = lazy_frame.collect()?;
        HashedDataFrame::new(data_frame)
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
    pub(crate) ddof: u8,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) selections: &'a Vec<Selection>,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(data_frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame: data_frame,
            ddof: settings.ddof,
            percent: settings.percent,
            precision: settings.precision,
            selections: &settings.selections,
            significant: settings.significant,
        }
    }
}

/// Table composition value
type Value = HashedDataFrame;

/// Format
fn format(mut lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let species = col(SPECIES).list().eval(as_struct(vec![
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
            .field_by_name(VALUE)
            .arr()
            .mean()
            .percent(key.percent)
            .precision(key.precision, key.significant)
            .alias(VALUE),
    ]));
    // Key, value
    lazy_frame = lazy_frame.select(
        (0..key.selections.len())
            .map(|index| {
                as_struct(vec![
                    {
                        let triacylglycerol = col(KEYS)
                            .struct_()
                            .field_by_index(index as _)
                            .triacylglycerol();
                        let args = [
                            triacylglycerol.clone().stereospecific_number1(),
                            triacylglycerol.clone().stereospecific_number2(),
                            triacylglycerol.stereospecific_number3(),
                        ];
                        match key.selections[index].composition {
                            ECN_STEREO | MASS_STEREO | SPECIES_STEREO | TYPE_STEREO
                            | UNSATURATION_STEREO => format_str("[{}; {}; {}]", args).unwrap(),
                            SPECIES_POSITIONAL | TYPE_POSITIONAL => {
                                format_str("[{}/2; {}; {}/2]", args).unwrap()
                            }
                            ECN_MONO | MASS_MONO | SPECIES_MONO | TYPE_MONO | UNSATURATION_MONO => {
                                format_str("[{}/3; {}/3; {}/3]", args).unwrap()
                            }
                        }
                        .alias(KEY)
                    },
                    mean_and_standard_deviation(
                        col(VALUES).list().get(lit(index as IdxSize), false),
                        key,
                    )
                    .alias(VALUE),
                ])
                .alias(index.to_string())
            })
            .chain(once(species))
            .collect::<Vec<_>>(),
    );
    println!("T1: {}", lazy_frame.clone().collect().unwrap());
    // Format
    // let exprs =
    //     key.selections
    //         .iter()
    //         .enumerate()
    //         .map(|(index, selection)| {
    //             // // Value
    //             // let value = mean_and_standard_deviation(
    //             //     col(index.to_string()).struct_().field_by_name(VALUE),
    //             //     key,
    //             // );
    //             // Key
    //             let key = col(index.to_string())
    //                 .struct_()
    //                 .field_by_name(KEY)
    //                 .triacylglycerol();
    //             let args = [
    //                 key.clone().stereospecific_number1(),
    //                 key.clone().stereospecific_number2(),
    //                 key.stereospecific_number3(),
    //             ];
    //             let key = match selection.composition {
    //                 ECN_STEREO | MASS_STEREO | SPECIES_STEREO | TYPE_STEREO
    //                 | UNSATURATION_STEREO => format_str("[{}; {}; {}]", args)?,
    //                 SPECIES_POSITIONAL | TYPE_POSITIONAL => format_str("[{}/2; {}; {}/2]", args)?,
    //                 ECN_MONO | MASS_MONO | SPECIES_MONO | TYPE_MONO | UNSATURATION_MONO => {
    //                     format_str("[{}/3; {}/3; {}/3]", args)?
    //                 }
    //             };
    //             Ok(
    //                 as_struct(vec![key.alias(KEY), col(index.to_string()).alias(VALUE)])
    //                     .alias(index.to_string()),
    //             )
    //         })
    //         .chain(once(Ok(species)))
    //         .collect::<PolarsResult<Vec<_>>>()?;
    Ok(lazy_frame)
}

fn mean_and_standard_deviation(array: Expr, key: Key) -> Expr {
    as_struct(vec![
        array
            .clone()
            .arr()
            .mean()
            .percent(key.percent)
            .precision(key.precision, key.significant)
            .alias(MEAN),
        array
            .clone()
            .arr()
            .std(key.ddof)
            .percent(key.percent)
            .precision(key.precision + 1, key.significant)
            .alias(STANDARD_DEVIATION),
        array
            .arr()
            .eval(
                element()
                    .percent(key.percent)
                    .precision(key.precision, key.significant),
                false,
            )
            .alias(SAMPLE),
    ])
}
