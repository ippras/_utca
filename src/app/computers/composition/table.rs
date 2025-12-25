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

/// Table composition computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Table composition computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        let mut lazy_frame = key.frame.data_frame.clone().lazy();
        // | Keys      | Values              | Species         |
        // | ---       | ---                 | ---             |
        // | struct[1] | list[array[f64, 3]] | list[struct[3]] |
        lazy_frame = compute(lazy_frame, key)?;
        // | Index | Key[n] | Value[n]  | Species         |
        // | ---   | ---    | ---       | ---             |
        // | u32   | str    | struct[3] | list[struct[3]] |
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

fn compute(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    concat_lf_diagonal(
        [body(lazy_frame.clone(), key)?, sum(lazy_frame, key)?],
        UnionArgs::default(),
    )
}

/// Body
fn body(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    let mut exprs = Vec::new();
    for index in 0..key.selections.len() {
        // Key
        let triacylglycerol = col(KEYS)
            .struct_()
            .field_by_index(index as _)
            .triacylglycerol();
        let args = [
            triacylglycerol.clone().stereospecific_number1(),
            triacylglycerol.clone().stereospecific_number2(),
            triacylglycerol.stereospecific_number3(),
        ];
        exprs.push(
            match key.selections[index].composition {
                ECN_STEREO | MASS_STEREO | SPECIES_STEREO | TYPE_STEREO | UNSATURATION_STEREO => {
                    format_str("[{}; {}; {}]", args)?
                }
                SPECIES_POSITIONAL | TYPE_POSITIONAL => format_str("[{}/2; {}; {}/2]", args)?,
                ECN_MONO | MASS_MONO | SPECIES_MONO | TYPE_MONO | UNSATURATION_MONO => {
                    format_str("[{}/3; {}/3; {}/3]", args)?
                }
            }
            .alias(format!("{KEY}[{index}]")),
        );
        // Value
        exprs.push(
            mean_and_standard_deviation(col(VALUES).list().get(lit(index as IdxSize), false), key)
                .alias(format!("{VALUE}[{index}]")),
        );
    }
    let species = col(SPECIES).list().eval(as_struct(vec![
        // Label
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
        // Triacylglycerol
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
        // Value
        mean_and_standard_deviation(element().struct_().field_by_name(VALUE), key).alias(VALUE),
    ]));
    exprs.push(species);
    Ok(lazy_frame.select(exprs).with_row_index(INDEX, None))
    // Ok(lazy_frame.select(
    //     (0..key.selections.len())
    //         .map(|index| {
    //             as_struct(vec![
    //                 {
    //                     let triacylglycerol = col(KEYS)
    //                         .struct_()
    //                         .field_by_index(index as _)
    //                         .triacylglycerol();
    //                     let args = [
    //                         triacylglycerol.clone().stereospecific_number1(),
    //                         triacylglycerol.clone().stereospecific_number2(),
    //                         triacylglycerol.stereospecific_number3(),
    //                     ];
    //                     match key.selections[index].composition {
    //                         ECN_STEREO | MASS_STEREO | SPECIES_STEREO | TYPE_STEREO
    //                         | UNSATURATION_STEREO => format_str("[{}; {}; {}]", args).unwrap(),
    //                         SPECIES_POSITIONAL | TYPE_POSITIONAL => {
    //                             format_str("[{}/2; {}; {}/2]", args).unwrap()
    //                         }
    //                         ECN_MONO | MASS_MONO | SPECIES_MONO | TYPE_MONO | UNSATURATION_MONO => {
    //                             format_str("[{}/3; {}/3; {}/3]", args).unwrap()
    //                         }
    //                     }
    //                     .alias(KEY)
    //                 },
    //                 mean_and_standard_deviation(
    //                     col(VALUES).list().get(lit(index as IdxSize), false),
    //                     key,
    //                 )
    //                 .alias(VALUE),
    //             ])
    //             .alias(index.to_string())
    //         })
    //         .chain(once(species))
    //         .collect::<Vec<_>>(),
    // ))
}

/// Sum
fn sum(lazy_frame: LazyFrame, key: Key) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame.select([mean_and_standard_deviation(
        eval_arr(col(VALUES).list().last(), |element| element.sum())?,
        key,
    )
    .alias(format!("{VALUE}[{}]", key.selections.len() - 1))]))
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
