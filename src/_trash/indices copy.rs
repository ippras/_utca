use crate::utils::Hashed;
use egui::util::cache::{ComputerMut, FrameCache};
use lipid::prelude::*;
use polars::prelude::*;
use std::num::NonZeroI8;
use tracing::instrument;

/// Composition indices computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Composition indices computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<Value> {
        many(key)
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Composition indices key
#[derive(Clone, Copy, Debug, Hash, PartialEq)]
pub(crate) struct Key<'a> {
    pub(crate) data_frame: Hashed<&'a DataFrame>,
    // pub(crate) ddof: u8,
}

/// Composition indices value
type Value = DataFrame;

// fn one(key: Key) -> PolarsResult<Value> {
//     let sn1 = || {
//         col("FattyAcid")
//             .triacylglycerol()
//             .stereospecific_number1()
//             .fatty_acid()
//     };
//     let sn2 = || {
//         col("FattyAcid")
//             .triacylglycerol()
//             .stereospecific_number2()
//             .fatty_acid()
//     };
//     let sn3 = || {
//         col("FattyAcid")
//             .triacylglycerol()
//             .stereospecific_number3()
//             .fatty_acid()
//     };
//     let value = || col("Value");
//     println!("lazy_frame000: {:?}", &key.data_frame);
//     let lazy_frame = key
//         .data_frame
//         .value
//         .clone()
//         .lazy()
//         .select([col("Species")])
//         .explode([col("Species")])
//         .unnest([col("Species")]);
//     println!("lazy_frame1: {}", lazy_frame.clone().collect()?);
//     concat_lf_diagonal(
//         [
//             lazy_frame.clone().select(compute(sn1, value)?),
//             lazy_frame.clone().select(compute(sn2, value)?),
//             lazy_frame.clone().select(compute(sn3, value)?),
//         ],
//         UnionArgs {
//             rechunk: true,
//             ..Default::default()
//         },
//     )?
//     .collect()
// }

macro_rules! exprs {
    ($f:ident, $fatty_acid:expr, $value:ident $(,$args:expr)*) => {
        (|| {
            let list = concat_list([0, 1, 2].map(|index| $fatty_acid.$f($value(index) $(,$args)*)))?;
            Ok(as_struct(vec![
                list.clone().list().mean().alias("Mean"),
                list.clone().list().std(1).alias("StandardDeviation"),
                list.alias("Repetitions"),
            ]))
        })()
    };
}

macro_rules! tag {
    ($f:ident, $fatty_acid:expr, $value:ident $(,$args:expr)*) => {
        (|| -> PolarsResult<_> {
            let fatty_acid = |name| $fatty_acid.clone().struct_().field_by_name(name);
            let list = |name| concat_list((0..3).map(move |index| fatty_acid(name).fatty_acid().$f($value(index) $(,$args)*)).collect::<Vec<_>>());
            let stereospecific_number = |name| -> PolarsResult<_> {
                Ok(as_struct(vec![
                    list(name)?.list().mean().alias("Mean"),
                    list(name)?.list().std(1).alias("StandardDeviation"),
                    list(name)?.alias("Repetitions"),
                ]))
            };
            concat_arr(vec![
                stereospecific_number("StereospecificNumber1")?,
                stereospecific_number("StereospecificNumber2")?,
                stereospecific_number("StereospecificNumber3")?,
            ])
        })()
    };
}

fn many(key: Key) -> PolarsResult<Value> {
    // let DataType::List(_) = key.data_frame.schema().get_field("Value").unwrap().dtype();
    let mut lazy_frame = key.data_frame.value.clone().lazy();
    println!("lazy_frame0: {}", lazy_frame.clone().collect()?);
    let value = |index| {
        col("Value")
            .struct_()
            .field_by_name("Repetitions")
            .list()
            .get(lit(index), false)
    };
    // println!(
    //     "lazy_frame1: {}",
    //     lazy_frame
    //         .clone()
    //         .with_column(
    //             concat_list([as_struct(vec![
    //                 col("Value").struct_().field_by_name("Repetitions"),
    //                 col("Triacylglycerol")
    //                     .triacylglycerol()
    //                     .stereospecific_number1(),
    //             ])])?
    //             .list()
    //             .eval({
    //                 // col("")
    //                 //     .struct_()
    //                 //     .field_by_name("StereospecificNumber1")
    //                 //     .fatty_acid()
    //                 //     .carbon()
    //                 //     * col("").struct_().field_by_name("Repetitions").explode()
    //                 let value = col("").struct_().field_by_name("Repetitions");
    //                 let fatty_acid = col("")
    //                     .struct_()
    //                     .field_by_name("StereospecificNumber1")
    //                     .fatty_acid();
    //                 fatty_acid.unsaturated(value.explode(), None)
    //                 // lit(1.0)
    //                 // col("a_list").gt(col("a"))
    //                 // println!("value: {value:?}");
    //                 // ternary_expr(fatty_acid.is_unsaturated(None), value, lit(NULL))
    //             })
    //             // .list()
    //             // .eval(
    //             //     col("").struct_().field_by_index(1) // .fatty_acid()
    //             //                                         // .unsaturated(col("").struct_().field_by_index(0), None)
    //             // )
    //             .alias("name")
    //         )
    //         .collect()?
    // );

    println!(
        "lazy_frame1: {}",
        // lazy_frame
        //     .clone()
        //     .select([
        //         col("Triacylglycerol")
        //             .triacylglycerol()
        //             .try_map_expr(|sn| exprs!(unsaturated, sn.clone().fatty_acid(), value, None))?
        //             .alias("name")
        //     ])
        //     .collect()?
        lazy_frame
            .clone()
            .select([
                tag!(monounsaturated, col("Triacylglycerol"), value)?.alias("Monounsaturated"),
                tag!(polyunsaturated, col("Triacylglycerol"), value)?.alias("Polyunsaturated"),
                tag!(saturated, col("Triacylglycerol"), value)?.alias("Saturated"),
                tag!(unsaturated, col("Triacylglycerol"), value, None)?.alias("Unsaturated"),
            ])
            .collect()?
    );

    lazy_frame = lazy_frame.clone().select(compute(
        [
            col("Triacylglycerol")
                .triacylglycerol()
                .stereospecific_number1()
                .fatty_acid(),
            col("Triacylglycerol")
                .triacylglycerol()
                .stereospecific_number2()
                .fatty_acid(),
            col("Triacylglycerol")
                .triacylglycerol()
                .stereospecific_number3()
                .fatty_acid(),
        ],
        col("Value").struct_().field_by_name("Mean"),
    )?);
    println!("lazy_frame2: {}", lazy_frame.clone().collect()?);
    lazy_frame.collect()
}

macro_rules! index {
    ($f:ident, $fatty_acid:expr, $value:expr $(,$args:expr)*) => {{
        concat_list($fatty_acid.clone().map(|fatty_acid| fatty_acid.$f($value.clone() $(,$args)*)))?.list().to_array(N)
    }};
}

fn compute<const N: usize>(
    fatty_acid: [FattyAcidExpr; N],
    values: Expr,
) -> PolarsResult<[Expr; 18]> {
    Ok([
        index!(monounsaturated, fatty_acid, values),
        index!(polyunsaturated, fatty_acid, values),
        index!(saturated, fatty_acid, values),
        index!(trans, fatty_acid, values),
        index!(unsaturated, fatty_acid, values, None),
        index!(unsaturated, fatty_acid, values, NonZeroI8::new(-9)),
        index!(unsaturated, fatty_acid, values, NonZeroI8::new(-6)),
        index!(unsaturated, fatty_acid, values, NonZeroI8::new(-3)),
        index!(unsaturated, fatty_acid, values, NonZeroI8::new(9)),
        index!(eicosapentaenoic_and_docosahexaenoic, fatty_acid, values),
        index!(fish_lipid_quality, fatty_acid, values),
        index!(health_promoting_index, fatty_acid, values),
        index!(
            hypocholesterolemic_to_hypercholesterolemic,
            fatty_acid,
            values
        ),
        index!(index_of_atherogenicity, fatty_acid, values),
        index!(index_of_thrombogenicity, fatty_acid, values),
        index!(linoleic_to_alpha_linolenic, fatty_acid, values),
        index!(polyunsaturated_to_saturated, fatty_acid, values),
        index!(unsaturation_index, fatty_acid, values),
        // fa().monounsaturated(values()),
        // fa().polyunsaturated(values()),
        // fa().saturated(values()),
        // fa().trans(values()),
        // fa().unsaturated(values(), None),
        // fa().unsaturated(values(), NonZeroI8::new(-9)),
        // fa().unsaturated(values(), NonZeroI8::new(-6)),
        // fa().unsaturated(values(), NonZeroI8::new(-3)),
        // fa().unsaturated(values(), NonZeroI8::new(9)),
        // fa().eicosapentaenoic_and_docosahexaenoic(values()),
        // fa().fish_lipid_quality(values()),
        // fa().health_promoting_index(values()),
        // fa().hypocholesterolemic_to_hypercholesterolemic(values()),
        // fa().index_of_atherogenicity(values()),
        // fa().index_of_thrombogenicity(values()),
        // fa().linoleic_to_alpha_linolenic(values()),
        // fa().polyunsaturated_to_saturated(values()),
        // fa().unsaturation_index(values()),
    ])
}

fn is_many(data_frame: &DataFrame) -> PolarsResult<bool> {
    let Some(experimental) = data_frame.schema().get("Experimental") else {
        polars_bail!(SchemaMismatch: "Поле 'Experimental' не найдено в схеме.");
    };
    let DataType::Struct(experimental_fields) = experimental else {
        polars_bail!(SchemaMismatch: "Поле 'Experimental' не является Struct.");
    };
    let Some(triacylglycerol) = experimental_fields
        .iter()
        .find(|field| field.name() == "Triacylglycerol")
    else {
        polars_bail!(SchemaMismatch: "Поле 'Triacylglycerol' не найдено внутри 'Experimental'.");
    };
    match triacylglycerol.dtype() {
        DataType::Struct(_) => {
            return Ok(true);
        }
        DataType::Float64 => {
            return Ok(false);
        }
        other_type => {
            polars_bail!(SchemaMismatch: "Поле 'Experimental' -> 'Triacylglycerol' имеет другой тип: {:?}", other_type);
        }
    }
}
