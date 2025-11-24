//! [Rabelo (2000)](https://doi.org/10.1007/s11746-000-0197-z)

use super::T_0;
use lipid::prelude::*;
use polars::prelude::*;

#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Options {
    pub temperature: f64,
    pub intermediate: bool,
}

/// Dynamic viscosities (η)
pub(crate) fn fatty_acids(mut lazy_frame: LazyFrame, options: Options) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
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
            (col("_A") + col("_B") / (lit(options.temperature) - col("_C")))
                .exp()
                .alias("η"),
        );
    if !options.intermediate {
        lazy_frame = lazy_frame.drop(by_name(
            [
                "_A_1", "_A_2", "_A_3", "_A_4", "_A_5", "_B_1", "_C_1", "_C_2", "_C_3", "_n_C",
                "_n_D", "_A", "_B", "_C",
            ],
            true,
        ));
    }
    Ok(lazy_frame)
}

/// Dynamic viscosities (η)
pub(crate) fn triacylglycerols(
    mut lazy_frame: LazyFrame,
    options: Options,
) -> PolarsResult<LazyFrame> {
    lazy_frame = lazy_frame
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
            (col("_A") + col("_B") / (lit(options.temperature) - col("_C")))
                .exp()
                .alias("η"),
        );
    if !options.intermediate {
        lazy_frame = lazy_frame.drop(by_name(
            [
                "_A_1", "_A_2", "_A_3", "_A_4", "_A_5", "_B_1", "_C_1", "_C_2", "_C_3", "_n_C",
                "_n_D", "_A", "_B", "_C",
            ],
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
                fatty_acid!(C18 { 9 => C })?,
                fatty_acid!(C18 { 9 => C, 12 => C })?,
            ],
        }?;
        let lazy_frame = fatty_acids(
            data_frame.lazy(),
            Options {
                temperature: T_0 + 40.0,
                ..Default::default()
            },
        )?;
        println!("lazy_frame: {}", lazy_frame.collect()?);
        Ok(())
    }

    #[test]
    fn test2() -> PolarsResult<()> {
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
        let lazy_frame = triacylglycerols(
            data_frame.lazy(),
            Options {
                temperature: T_0 + 60.0,
                ..Default::default()
            },
        )?;
        println!("lazy_frame: {}", lazy_frame.collect()?);
        Ok(())
    }
}
