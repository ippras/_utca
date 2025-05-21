use polars::prelude::*;

/// Convert type `Boolean` to `String`
pub fn r#type(series: &Series) -> PolarsResult<Series> {
    Ok(series
        .bool()?
        .iter()
        .map(|saturated| Some(if saturated? { "S" } else { "U" }))
        .collect::<StringChunked>()
        .into_series())
}

pub fn tag_map(
    f: impl Fn(&Series) -> PolarsResult<Series>,
) -> impl Fn(&Series) -> PolarsResult<Series> {
    move |series| {
        let r#struct = series.struct_()?;
        let sn1 = r#struct.field_by_name("StereospecificNumber1")?;
        let sn2 = r#struct.field_by_name("StereospecificNumber2")?;
        let sn3 = r#struct.field_by_name("StereospecificNumber3")?;
        Ok(StructChunked::from_series(
            PlSmallStr::EMPTY,
            series.len(),
            [
                f(&sn1)?.with_name("StereospecificNumber1".into()),
                f(&sn2)?.with_name("StereospecificNumber2".into()),
                f(&sn3)?.with_name("StereospecificNumber3".into()),
            ]
            .iter(),
        )?
        .into_series())
    }
}

// pub use self::expr::ExprExt;

// pub fn column(
//     function: impl Fn(&Series) -> PolarsResult<Series>,
// ) -> impl Fn(Column) -> PolarsResult<Option<Column>> {
//     move |column| {
//         let Some(series) = column.as_series() else {
//             return Ok(None);
//         };
//         Ok(Some(function(series)?.into_column()))
//     }
// }

// fn normalize(series: &Series) -> PolarsResult<Series> {
//     let chunked_array = series.f64()?;
//     let sum = chunked_array.sum();
//     Ok(chunked_array
//         .iter()
//         .map(|option| Some(option.unwrap_or_default() / sum?))
//         .collect::<Float64Chunked>()
//         .into_series())
// }

// pub fn hash(series: &Series) -> PolarsResult<Series> {
//     use egui::util::hash;

//     Ok(series
//         .iter()
//         .map(|value| Ok(Some(hash(value))))
//         .collect::<PolarsResult<UInt64Chunked>>()?
//         .into_series())
// }

// pub fn round(decimals: u32) -> impl Fn(&Series) -> PolarsResult<Series> {
//     move |series| series.round(decimals)
// }
