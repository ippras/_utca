//! [Hammond (1954)](https://doi.org/10.1007/BF02639027)

use lipid::prelude::*;
use polars::prelude::*;

// Молярная рефракция, молярный объем и показатель преломления
// * R_m считается функцией объема, фактически занимаемого молекулами моля
//   вещества, почти не зависит от температуры.
// * V_m и константы k_4, k_5 и k_6 будут справедливы только для одной
//   температуры - 20°C.
// * n также будет справедлив только для одной температуры - 20°C.

/// Molar refraction ("R_m"), molar volume ("V_m"), refractive index ("n")
fn fatty_acids(mut lazy_frame: LazyFrame, intermediate: bool) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
        .with_columns([
            lit(4.641).alias("_k_1"),
            lit(-0.30).alias("_k_2"),
            lit(8.247).alias("_k_3"),
            lit(16.54).alias("_k_4"),
            ternary_expr(
                col(FATTY_ACID).fatty_acid().is_monoenoic(),
                lit(-6.65),
                lit(-6.87),
            )
            .alias("_k_5"),
            lit(47.99).alias("_k_6"),
        ])
        .with_columns([
            col(FATTY_ACID)
                .fatty_acid()
                .carbon()
                .cast(DataType::Float64)
                .alias("_C"),
            col(FATTY_ACID)
                .fatty_acid()
                .double_bounds_unsaturation()
                .cast(DataType::Float64)
                .alias("_D"),
        ])
        .with_columns([
            (col("_k_1") * col("_C") + col("_k_2") * col("_D") + col("_k_3")).alias("R_m"),
            (col("_k_4") * col("_C") + col("_k_5") * col("_D") + col("_k_6")).alias("V_m"),
            (((lit(2) * col("_k_1") + col("_k_4")) * col("_C")
                + (lit(2) * col("_k_2") + col("_k_5")) * col("_D")
                + (lit(2) * col("_k_3") + col("_k_6")))
                / ((col("_k_4") - col("_k_1")) * col("_C")
                    + (col("_k_5") - col("_k_2")) * col("_D")
                    + (col("_k_6") - col("_k_3"))))
            .sqrt()
            .alias("n"),
        ]);
    if !intermediate {
        lazy_frame = lazy_frame.drop(by_name(
            ["_k_1", "_k_2", "_k_3", "_k_4", "_k_5", "_k_6", "_C", "_D"],
            true,
        ));
    }
    Ok(lazy_frame)
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test1() -> PolarsResult<()> {
        let data_frame = df! {
            FATTY_ACID => [
                fatty_acid!(C16 { })?,
                fatty_acid!(C18 { })?,
                fatty_acid!(C18 { 9 => C })?,
                fatty_acid!(C18 { 9 => C, 12 => C })?,
                fatty_acid!(C18 { 9 => C, 12 => C, 15 => C })?,
                fatty_acid!(C20 { })?,
                fatty_acid!(C20 { 11 => C })?,
                fatty_acid!(C22 { })?,
                fatty_acid!(C22 { 13 => C })?,
                fatty_acid!(C24 { })?,
                fatty_acid!(C24 { 15 => C })?,
            ],
        }?;
        println!("data_frame: {data_frame}");
        let lazy_frame = fatty_acids(data_frame.lazy(), false)?;
        println!("lazy_frame: {}", lazy_frame.collect()?);
        Ok(())
    }
}
