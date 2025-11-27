use crate::{
    app::states::calculation::{Indices, Settings},
    utils::HashedDataFrame,
};
use const_format::formatcp;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use polars_ext::expr::{ExprExt, ExprIfExt};
use tracing::instrument;

const STEREOSPECIFIC_NUMBERS: [&str; 3] = [
    STEREOSPECIFIC_NUMBERS123,
    STEREOSPECIFIC_NUMBERS13,
    STEREOSPECIFIC_NUMBERS2,
];

const ARRAY: &str = r#"^.*\.Array$"#;
const MEAN: &str = r#"^.*\.Mean$"#;

/// Calculation biodiesel properties computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Calculation biodiesel properties computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        compute(key)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Calculation biodiesel properties key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) frame: &'a HashedDataFrame,
    pub(crate) ddof: u8,
    pub(crate) precision: usize,
    pub(crate) save: bool,
    pub(crate) significant: bool,
}

impl<'a> Key<'a> {
    pub(crate) fn new(frame: &'a HashedDataFrame, settings: &'a Settings) -> Self {
        Self {
            frame,
            ddof: settings.ddof,
            precision: settings.precision,
            save: settings.threshold.save,
            significant: settings.significant,
        }
    }
}

/// Calculation biodiesel properties value
type Value = DataFrame;

fn compute(key: Key) -> PolarsResult<Value> {
    let mut lazy_frame = key.frame.data_frame.clone().lazy();
    println!(
        "lazy_frame PRO 0: {}",
        lazy_frame.clone().collect().unwrap()
    );
    // Пока не будет готов
    // https://github.com/pola-rs/polars/pull/23316
    lazy_frame = lazy_frame
        .with_row_index("Index", None)
        .unnest(
            cols([formatcp!(r#"^StereospecificNumbers\d+$"#)]),
            // cols([STEREOSPECIFIC_NUMBERS123]),
            Some(PlSmallStr::from_static(".")),
        )
        .select([
            col("Index"),
            // .eval(
            //     col(FATTY_ACID)
            //         .fatty_acid()
            //         .cetane_number(element())
            //         .percent_if(true)
            //         .name()
            //         .replace("Array", "CetaneNumber.Array", true),
            //     false,
            // )
            col(ARRAY),
            // .arr()
            // .eval(
            //     as_struct(vec![element().cum_count(false).alias("Index"), element()]),
            //     false,
            // )
            // .implode(),
            // col(FATTY_ACID)
            //     .fatty_acid()
            //     .cetane_number(col(MEAN))
            //     .percent_if(true)
            //     .name()
            //     .replace("Mean", "CetaneNumber.Mean", true),
            // col(FATTY_ACID)
            //     .fatty_acid()
            //     .cold_filter_plugging_point(col(MEAN))
            //     .percent_if(true)
            //     .name()
            //     .replace("Mean", "ColdFilterPluggingPoint.Mean", true),
            // col(FATTY_ACID)
            //     .fatty_acid()
            //     .degree_of_unsaturation(col(MEAN))
            //     .percent_if(true)
            //     .name()
            //     .replace("Mean", "DegreeOfUnsaturation.Mean", true),
            // BiodieselProperties::iodine_value(col(FATTY_ACID).fatty_acid(), col(MEAN))
            //     .percent_if(true)
            //     .name()
            //     .replace("Mean", "IodineValue.Mean", true),
            // col(FATTY_ACID)
            //     .fatty_acid()
            //     .long_chain_saturated_factor(col(MEAN))
            //     .percent_if(true)
            //     .name()
            //     .replace("Mean", "LongChainSaturatedFactor.Mean", true),
            // col(FATTY_ACID)
            //     .fatty_acid()
            //     .oxidation_stability(col(MEAN))
            //     .percent_if(true)
            //     .name()
            //     .replace("Mean", "OxidationStability.Mean", true),
        ])
        .explode(cols([ARRAY]));
    println!(
        "lazy_frame PRO 1: {}",
        lazy_frame.clone().collect().unwrap()
    );
    lazy_frame = lazy_frame.with_columns([as_struct(vec![col(ARRAY)])]);
    println!(
        "lazy_frame PRO 2: {}",
        lazy_frame.clone().collect().unwrap()
    );
    lazy_frame = lazy_frame.group_by("Index").agg([
        col(ARRAY),
        //    col("rain").sum().alias("sum_rain"),
        //    col("rain").quantile(lit(0.5), QuantileMethod::Nearest).alias("median_rain"),
    ]);
    // lazy_frame = lazy_frame.unnest(cols(["*"]), Some(PlSmallStr::from_static(".")));
    println!(
        "lazy_frame PRO 3: {}",
        lazy_frame.clone().collect().unwrap()
    );
    std::process::exit(1);
    // let fatty_acid = || col(FATTY_ACID).fatty_acid();
    // let values = |expr: Expr| {
    //     (0..length).map(move |index| {
    //         expr.clone()
    //             .struct_()
    //             .field_by_name("Array")
    //             .arr()
    //             .get(lit(index), false)
    //     })
    // };
    // let stereospecific_numbers = |expr: Expr| -> PolarsResult<Expr> {
    //     Ok(as_struct(vec![
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().monounsaturated(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().polyunsaturated(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().saturated(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().trans(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().unsaturated(value, None))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-9)))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-6)))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(-3)))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().unsaturated(value, NonZeroI8::new(9)))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().eicosapentaenoic_and_docosahexaenoic(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().fish_lipid_quality(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().health_promoting_index(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().hypocholesterolemic_to_hypercholesterolemic(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().index_of_atherogenicity(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().index_of_thrombogenicity(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().linoleic_to_alpha_linolenic(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().polyunsaturated_6_to_polyunsaturated_3(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().polyunsaturated_to_saturated(value))
    //                 .collect(),
    //         )?,
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| fatty_acid().unsaturation_index(value))
    //                 .collect(),
    //         )?,
    //         // P
    //         concat_arr(
    //             values(expr.clone())
    //                 .map(|value| {
    //                     (value * fatty_acid().iodine_value())
    //                         .sum()
    //                         .alias("IodineValue")
    //                 })
    //                 .collect(),
    //         )?,
    //         // concat_arr(
    //         //     values(expr.clone())
    //         //         .map(|value| {
    //         //             (lit(0.6683) * fatty_acid().unsaturation_index(value) + lit(0.250364))
    //         //                 .alias("IodineValue.Wang2012")
    //         //         })
    //         //         .collect(),
    //         // )?,
    //         // concat_arr(
    //         //     values(expr.clone())
    //         //         .map(|value| {
    //         //             (lit(-0.1209) * fatty_acid().unsaturation_index(value) + lit(0.650958))
    //         //                 .alias("CetaneNumber")
    //         //         })
    //         //         .collect(),
    //         // )?,
    //         // concat_arr(
    //         //     values(expr.clone())
    //         //         .map(|value| {
    //         //             (lit(-0.0384) * fatty_acid().degree_of_unsaturation(value) + lit(0.777))
    //         //                 .alias("OxidativeStability")
    //         //         })
    //         //         .collect(),
    //         // )?,
    //         // concat_arr(
    //         //     values(expr.clone())
    //         //         .map(|value| {
    //         //             (lit(1.7556) * fatty_acid().degree_of_unsaturation(value) + lit(-0.14772))
    //         //                 .alias("ColdFilterPluggingPoint")
    //         //         })
    //         //         .collect(),
    //         // )?,
    //         // concat_arr(
    //         //     values(expr.clone())
    //         //         .map(|value| {
    //         //             fatty_acid()
    //         //                 .long_chain_saturated_factor(value)
    //         //                 .alias("LongChainSaturatedFactor")
    //         //         })
    //         //         .collect(),
    //         // )?,
    //     ]))
    // };
    // lazy_frame = lazy_frame.select([
    //     stereospecific_numbers(col(STEREOSPECIFIC_NUMBERS123))?.alias(STEREOSPECIFIC_NUMBERS123),
    //     stereospecific_numbers(col(STEREOSPECIFIC_NUMBERS13))?.alias(STEREOSPECIFIC_NUMBERS13),
    //     stereospecific_numbers(col(STEREOSPECIFIC_NUMBERS2))?.alias(STEREOSPECIFIC_NUMBERS2),
    // ]);
    // // Mean and standard deviation
    // let exprs = STEREOSPECIFIC_NUMBERS
    //     .into_iter()
    //     .map(|stereospecific_numbers| {
    //         as_struct(
    //             key.indices
    //                 .iter_visible()
    //                 .map(|name| {
    //                     as_struct(vec![
    //                         col(stereospecific_numbers)
    //                             .struct_()
    //                             .field_by_name(name)
    //                             .clone()
    //                             .arr()
    //                             .mean()
    //                             .alias("Mean"),
    //                         col(stereospecific_numbers)
    //                             .struct_()
    //                             .field_by_name(name)
    //                             .clone()
    //                             .arr()
    //                             .std(key.ddof)
    //                             .alias("StandardDeviation"),
    //                         col(stereospecific_numbers)
    //                             .struct_()
    //                             .field_by_name(name)
    //                             .alias("Array"),
    //                     ])
    //                     .alias(name)
    //                 })
    //                 .collect(),
    //         )
    //         .alias(stereospecific_numbers)
    //     })
    //     .collect::<Vec<_>>();
    // lazy_frame = lazy_frame.select(exprs);
    // // Format
    // lazy_frame = lazy_frame
    //     .unnest(all(), Some(PlSmallStr::from_static("_")))
    //     .unnest(all(), Some(PlSmallStr::from_static("_")))
    //     .with_columns([
    //         col(r#"^.*_Mean$"#).precision(key.precision, key.significant),
    //         col(r#"^.*_StandardDeviation$"#).precision(key.precision, key.significant),
    //         col(r#"^.*_Array$"#)
    //             .arr()
    //             .eval(element().precision(key.precision, key.significant), false),
    //     ]);
    // let exprs = STEREOSPECIFIC_NUMBERS.map(|stereospecific_number| {
    //     as_struct(
    //         key.indices
    //             .iter_visible()
    //             .map(|name| {
    //                 as_struct(vec![
    //                     col(format!("{stereospecific_number}_{name}_Mean")).alias("Mean"),
    //                     col(format!("{stereospecific_number}_{name}_StandardDeviation"))
    //                         .alias("StandardDeviation"),
    //                     col(format!("{stereospecific_number}_{name}_Array")).alias("Array"),
    //                 ])
    //                 .alias(name)
    //             })
    //             .collect(),
    //     )
    //     .alias(stereospecific_number)
    // });
    // lazy_frame = lazy_frame.select(exprs);
    lazy_frame.collect()
}
