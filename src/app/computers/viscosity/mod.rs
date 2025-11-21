//! [Rabelo (2000)](https://doi.org/10.1007/s11746-000-0197-z)
//! Rabelo2000
//! https://doi.org/10.1007/s11746-000-0197-z
//! Viscosity prediction for fatty systems
//! Dynamic viscosities (η)

use lipid::prelude::*;
use polars::{
    lazy::dsl::{max_horizontal, min_horizontal, sum_horizontal},
    prelude::*,
};
use std::ops::Rem;

// ГОСТ 8.157-75
const T_0: f64 = 273.15;

fn fatty_acid(lazy_frame: LazyFrame, t: f64) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame
        .with_columns([
            lit(-6.09).alias("_A_1"),
            lit(-3.536).alias("_A_2"),
            lit(5.40).alias("_A_3"),
            lit(3.10).alias("_A_4"),
            lit(-0.066).alias("_A_5"),
            lit(1331.5).alias("_B_1"),
            lit(41.6).alias("_C_1"),
            lit(4.135).alias("_C_2"),
            lit(-8.0).alias("_C_3"),
        ])
        .with_columns([
            col(FATTY_ACID)
                .fatty_acid()
                .carbon()
                .cast(DataType::Float64)
                .alias("_n_C"),
            col(FATTY_ACID)
                .fatty_acid()
                .double_bounds_unsaturation()
                .cast(DataType::Float64)
                .alias("_n_D"),
        ])
        .with_columns([
            (col("_C_1") + col("_C_2") * col("_n_C") + col("_C_3") * col("_n_D")).alias("_C"),
            col("_B_1").alias("_B"),
            ((col("_A_1") - col("_A_2"))
                / (lit(1) + ((col("_n_C") - col("_A_3")) / col("_A_4")).exp())
                + col("_A_2")
                + col("_A_5") * col("_n_D"))
            .alias("_A"),
        ])
        .with_column(
            (col("_A") + col("_B") / (lit(T_0 + 40.0) - col("_C")))
                .exp()
                .alias("η"),
        ))
}

fn triacylglycerol(lazy_frame: LazyFrame, t: f64) -> PolarsResult<LazyFrame> {
    Ok(lazy_frame
        .with_columns([
            lit(-4.01).alias("_A_1"),
            lit(-2.954).alias("_A_2"),
            lit(28.9).alias("_A_3"),
            lit(6.5).alias("_A_4"),
            lit(-0.0033).alias("_A_5"),
            lit(1156).alias("_B_1"),
            lit(99.1).alias("_C_1"),
            lit(0.851).alias("_C_2"),
            lit(-3.65).alias("_C_3"),
        ])
        .with_columns([
            col(TRIACYLGLYCEROL)
                .triacylglycerol()
                .carbon()
                .cast(DataType::Float64)
                .alias("_n_C"),
            col(TRIACYLGLYCEROL)
                .triacylglycerol()
                .map(|expr| expr.fatty_acid().double_bounds_unsaturation())
                .triacylglycerol()
                .sum()
                .cast(DataType::Float64)
                .alias("_n_D"),
        ])
        .with_columns([
            (col("_C_1") + col("_C_2") * col("_n_C") + col("_C_3") * col("_n_D")).alias("_C"),
            col("_B_1").alias("_B"),
            ((col("_A_1") - col("_A_2"))
                / (lit(1) + ((col("_n_C") - col("_A_3")) / col("_A_4")).exp())
                + col("_A_2")
                + col("_A_5") * col("_n_D"))
            .alias("_A"),
        ])
        .with_column(
            (col("_A") + col("_B") / (lit(t) - col("_C")))
                .exp()
                .alias("η"),
        ))
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;

    #[test]
    fn test1() -> Result<()> {
        let data_frame = df! {
            FATTY_ACID => [
                fatty_acid!(C16 { })?,
                fatty_acid!(C18 { 9 => C })?,
                fatty_acid!(C18 { 9 => C, 12 => C })?,
            ],
        }?;
        println!("data_frame: {data_frame}");
        let lazy_frame = fatty_acid(data_frame.lazy(), T_0 + 40.0)?;
        println!("lazy_frame: {}", lazy_frame.collect()?);
        Ok(())
    }

    #[test]
    fn test2() -> Result<()> {
        let data_frame = df! {
            TRIACYLGLYCEROL => df! {
                STEREOSPECIFIC_NUMBERS1 => [
                    fatty_acid!(C16 { })?,
                    fatty_acid!(C18 { 9 => C })?,
                    fatty_acid!(C18 { 9 => C, 12 => C })?,
                    fatty_acid!(C18 { 9 => C })?,
                ],
                STEREOSPECIFIC_NUMBERS2 => [
                    fatty_acid!(C16 { })?,
                    fatty_acid!(C18 { 9 => C })?,
                    fatty_acid!(C18 { 9 => C, 12 => C })?,
                    fatty_acid!(C16 { })?,
                ],
                STEREOSPECIFIC_NUMBERS3 => [
                    fatty_acid!(C16 { })?,
                    fatty_acid!(C18 { 9 => C })?,
                    fatty_acid!(C18 { 9 => C, 12 => C })?,
                    fatty_acid!(C18 { 9 => C, 12 => C })?,
                ],
            }?.into_struct(PlSmallStr::EMPTY).into_series(),
        }?;
        println!("data_frame: {data_frame}");
        let lazy_frame = triacylglycerol(data_frame.lazy(), T_0 + 60.0)?;
        println!("lazy_frame: {}", lazy_frame.collect()?);
        Ok(())
    }
}
