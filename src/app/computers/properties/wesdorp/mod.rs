//! [Calculator (2017)](https://lipidlibrary.shinyapps.io/Triglyceride_Property_Calculator/)
//! [Calculator (2021)](https://tri.marangoni.tech)
//! [Moorthy2016](https://doi.org/10.1007/s11746-016-2935-1)
//! [Seilert2021](https://doi.org/10.1002/aocs.12515)

use super::T_0;
use lipid::prelude::*;
use polars::{
    lazy::dsl::{max_horizontal, min_horizontal, sum_horizontal},
    prelude::*,
};
use std::ops::Rem;

// 1 ℎ𝑜 𝑘𝐽 𝑚𝑜𝑙⁄ -31.95 -35.86 -17.16
// 2 ℎ 𝑘𝐽 𝑚𝑜𝑙⁄ ⋅ 𝑛𝐶 2.7 3.86 3.89
// 3 𝑠𝑜 𝐽 𝐾 ⋅ 𝑚𝑜𝑙⁄ -19.09 -39.59 31.04
// 4 𝑠 𝐽 𝐾 ⋅ 𝑚𝑜𝑙⁄ ⋅ 𝑛𝐶 6.79 10.13 9.83
// 5 ℎ𝑥𝑦 𝑘𝐽 𝑚𝑜𝑙⁄ -13.28 -19.35 -22.29
// 6 𝑠𝑥𝑦 𝐽 𝐾 ⋅ 𝑚𝑜𝑙⁄ -36.7 -52.51 -64.58
// 7 𝑘𝑥 = 𝑘𝑦 = 𝑘 𝑛𝐶 4.39 1.99 2.88
// 8 𝑥𝑜 𝑛𝐶 1.25 2.46 0.77
// 9 𝑇∞,𝑒 𝐾 401.15 401.15 401.15
// 10 ℎ𝑜𝑑𝑑 𝑘𝐽 𝑚𝑜𝑙⁄ - - 2.29
// 11 𝑠𝑜𝑑𝑑 𝐽 𝑚𝑜𝑙⁄ - - -
// 12 𝐴𝑜 1/𝑛𝐶 -9.058 -8.454 -8.048
// 13 𝐴𝑜𝑑𝑑 1/𝑛𝐶 -0.196 -0.308 -0.019
// 14 𝐴𝑥 1/𝑛𝐶 0.003 -0.104 0.074
// 15 𝐴𝑥2 1/𝑛𝐶 -0.062 -0.019 -0.035
// 16 𝐴𝑥𝑦 1/𝑛𝐶 0.115 0.074 0.008
// 17 𝐴𝑦 1/𝑛𝐶 -0.453 -0.497 -0.404
// 18 𝐴𝑦2 1/𝑛𝐶 -0.006 0.012 0.011
// 19 𝐵𝑜 1/𝑛𝐶 -4.484 -0.265 2.670
// 20 𝐵𝑜𝑑𝑑 1/𝑛𝐶 -0.003 0.005 0.008
// 21 𝐵𝑥 1/𝑛𝐶 -0.001 0.550 -0.317
// 22 𝐵𝑥2 1/𝑛𝐶 0.149 0.074 0.086
// 23 𝐵𝑥𝑦 1/𝑛𝐶 -0.366 -0.341 0.041
// 24 𝐵𝑦 1/𝑛𝐶 1.412 2.342 0.550
// 25 𝐵𝑦2 1/𝑛𝐶 -0.002 -0.136 9e-4
// 26 𝐴̂ 𝑂 1/𝑛𝑂 -3.46 -2.2 -2.93
// 27 𝐴̂ 𝐸 1/𝑛𝐸 -1.38 -1.34 -1.68
// 28 𝐴̂𝐽 1/𝑛𝐽 -3.35 -2.51 -4.69
// 29 𝐴̂ 𝑁 1/𝑛𝑁 -4.2 -2.23 -5.18
// 30 𝐴̂ 𝑂𝑂 1/𝑛𝑂𝑂 -0.01 0.27 0.89
// 31 𝐴̂ 𝐸𝐸 1/𝑛𝐸𝐸 0.01 0.04 0.4
// 32 𝐴̂𝐽𝐽 1/𝑛𝐽𝐽 -3.68 0.55 1.21
// 33 𝐴̂ 𝑁𝑁 1/𝑛𝑁𝑁 -0.98 1.51 1.38
// 34 𝐴̂ 𝑂𝐽 1/𝑛𝑂𝐽 0.53 -1 0.71
// 35 𝐴̂ 𝑂𝑁 1/𝑛𝑂𝑁 0.83 0.76 0.69
// 36 𝐴̂𝐽𝑁 1/𝑛𝐽𝑁 -2.97 1.12 0.73
// 37 𝐵̂𝑂 1/𝑛𝑂 0 -4.3 -3.7
// 38 𝐵̂𝐽 1/𝑛𝐽 5.4 -7.8 -1.5
// 39 𝐵̂𝑁 1/𝑛𝑁 2.6 -13.7 -1.8
// 40 ℎ̂ 𝑂 𝑘𝐽 𝑚𝑜𝑙⁄ ⋅ 𝑛𝑂 -31.7 -28.3 -30.2
// 41 ℎ̂ 𝐸 𝑘𝐽 𝑚𝑜𝑙⁄ ⋅ 𝑛𝐸 -11.7 (-15.9) -15.9
// 42 ℎ̂ 𝐽 𝑘𝐽 𝑚𝑜𝑙⁄ ⋅ 𝑛𝐽 (-37.7) (-37.7) -37.7
fn triacylglycerols(mut lazy_frame: LazyFrame, options: Options) -> PolarsResult<LazyFrame> {
    lazy_frame = match options.polymorphism {
        Polymorphism::Alpha => lazy_frame.with_columns([
            lit(-31.95).alias("h_0"),
            lit(2.7).alias("h"),
            lit(-19.09).alias("s_0"),
            lit(6.79).alias("s"),
            lit(-13.28).alias("h_xy"),
            lit(-36.7).alias("s_xy"),
            lit(4.39).alias("k"),
            lit(1.25).alias("x_0"),
            lit(401.15).alias("T_∞,e"),
            lit(NULL).alias("h_odd"),
            lit(NULL).alias("s_odd"),
            lit(-9.058).alias("A_0"),
            lit(-0.196).alias("A_odd"),
            lit(0.003).alias("A_x"),
            lit(-0.062).alias("A_x^2"),
            lit(0.115).alias("A_xy"),
            lit(-0.453).alias("A_y"),
            lit(-0.006).alias("A_y^2"),
            lit(-4.484).alias("B_0"),
            lit(-0.003).alias("B_odd"),
            lit(-0.001).alias("B_x"),
            lit(0.149).alias("B_x^2"),
            lit(-0.366).alias("B_xy"),
            lit(1.412).alias("B_y"),
            lit(-0.002).alias("B_y^2"),
            lit(-3.46).alias("A_O"),
            lit(-1.38).alias("A_E"),
            lit(-3.35).alias("A_J"),
            lit(-4.2).alias("A_N"),
            lit(-0.01).alias("A_OO"),
            lit(0.01).alias("A_EE"),
            lit(-3.68).alias("A_JJ"),
            lit(-0.98).alias("A_NN"),
            lit(0.53).alias("A_OJ"),
            lit(0.83).alias("A_ON"),
            lit(-2.97).alias("A_JN"),
            lit(0).alias("B_O"),
            lit(5.4).alias("B_J"),
            lit(2.6).alias("B_N"),
            lit(-31.7).alias("h_O"),
            lit(-11.7).alias("h_E"),
            lit(-37.7).alias("h_J"),
        ]),
        Polymorphism::BetaPrime => lazy_frame.with_columns([
            lit(-35.86).alias("h_0"),
            lit(3.86).alias("h"),
            lit(-39.59).alias("s_0"),
            lit(10.13).alias("s"),
            lit(-19.35).alias("h_xy"),
            lit(-52.51).alias("s_xy"),
            lit(1.99).alias("k"),
            lit(2.46).alias("x_0"),
            lit(401.15).alias("T_∞,e"),
            lit(NULL).alias("h_odd"),
            lit(NULL).alias("s_odd"),
            lit(-8.454).alias("A_0"),
            lit(-0.308).alias("A_odd"),
            lit(-0.104).alias("A_x"),
            lit(-0.019).alias("A_x^2"),
            lit(0.074).alias("A_xy"),
            lit(-0.497).alias("A_y"),
            lit(0.012).alias("A_y^2"),
            lit(-0.265).alias("B_0"),
            lit(0.005).alias("B_odd"),
            lit(0.550).alias("B_x"),
            lit(0.074).alias("B_x^2"),
            lit(-0.341).alias("B_xy"),
            lit(2.342).alias("B_y"),
            lit(-0.136).alias("B_y^2"),
            lit(-2.2).alias("A_O"),
            lit(-1.34).alias("A_E"),
            lit(-2.51).alias("A_J"),
            lit(-2.23).alias("A_N"),
            lit(0.27).alias("A_OO"),
            lit(0.04).alias("A_EE"),
            lit(0.55).alias("A_JJ"),
            lit(1.51).alias("A_NN"),
            lit(-1).alias("A_OJ"),
            lit(0.76).alias("A_ON"),
            lit(1.12).alias("A_JN"),
            lit(-4.3).alias("B_O"),
            lit(-7.8).alias("B_J"),
            lit(-13.7).alias("B_N"),
            lit(-28.3).alias("h_O"),
            lit(-15.9).alias("h_E"),
            lit(-37.7).alias("h_J"),
        ]),
        Polymorphism::Beta => lazy_frame.with_columns([
            lit(-17.16).alias("h_0"),
            lit(3.89).alias("h"),
            lit(31.04).alias("s_0"),
            lit(9.83).alias("s"),
            lit(-22.29).alias("h_xy"),
            lit(-64.58).alias("s_xy"),
            lit(2.88).alias("k"),
            lit(0.77).alias("x_0"),
            lit(401.15).alias("T_∞,e"),
            lit(2.29).alias("h_odd"),
            lit(NULL).alias("s_odd"),
            lit(-8.048).alias("A_0"),
            lit(-0.019).alias("A_odd"),
            lit(0.074).alias("A_x"),
            lit(-0.035).alias("A_x^2"),
            lit(0.008).alias("A_xy"),
            lit(-0.404).alias("A_y"),
            lit(0.011).alias("A_y^2"),
            lit(2.670).alias("B_0"),
            lit(0.008).alias("B_odd"),
            lit(-0.317).alias("B_x"),
            lit(0.086).alias("B_x^2"),
            lit(0.041).alias("B_xy"),
            lit(0.550).alias("B_y"),
            lit(9e-4).alias("B_y^2"),
            lit(-2.93).alias("A_O"),
            lit(-1.68).alias("A_E"),
            lit(-4.69).alias("A_J"),
            lit(-5.18).alias("A_N"),
            lit(0.89).alias("A_OO"),
            lit(0.4).alias("A_EE"),
            lit(1.21).alias("A_JJ"),
            lit(1.38).alias("A_NN"),
            lit(0.71).alias("A_OJ"),
            lit(0.69).alias("A_ON"),
            lit(0.73).alias("A_JN"),
            lit(-3.7).alias("B_O"),
            lit(-1.5).alias("B_J"),
            lit(-1.8).alias("B_N"),
            lit(-30.2).alias("h_O"),
            lit(-15.9).alias("h_E"),
            lit(-37.7).alias("h_J"),
        ]),
    };
    lazy_frame = lazy_frame
        .with_columns([
            col(TRIACYLGLYCEROL)
                .struct_()
                .field_by_name(STEREOSPECIFIC_NUMBERS1)
                .fatty_acid()
                .carbon()
                .cast(DataType::Float64)
                .alias("n_1"),
            col(TRIACYLGLYCEROL)
                .struct_()
                .field_by_name(STEREOSPECIFIC_NUMBERS2)
                .fatty_acid()
                .carbon()
                .cast(DataType::Float64)
                .alias("n_2"),
            col(TRIACYLGLYCEROL)
                .struct_()
                .field_by_name(STEREOSPECIFIC_NUMBERS3)
                .fatty_acid()
                .carbon()
                .cast(DataType::Float64)
                .alias("n_3"),
            col(TRIACYLGLYCEROL)
                .struct_()
                .field_by_name(STEREOSPECIFIC_NUMBERS1)
                .fatty_acid()
                .unsaturation()
                .cast(DataType::Float64)
                .alias("u_1"),
            col(TRIACYLGLYCEROL)
                .struct_()
                .field_by_name(STEREOSPECIFIC_NUMBERS2)
                .fatty_acid()
                .unsaturation()
                .cast(DataType::Float64)
                .alias("u_2"),
            col(TRIACYLGLYCEROL)
                .struct_()
                .field_by_name(STEREOSPECIFIC_NUMBERS3)
                .fatty_acid()
                .unsaturation()
                .cast(DataType::Float64)
                .alias("u_3"),
        ])
        .with_columns([
            sum_horizontal([col(r#"^n_\d$"#)], false)?.alias("n"),
            sum_horizontal([col(r#"^u_\d$"#)], false)?.alias("u"),
        ])
        .with_columns([
            min_horizontal([col("n_1"), col("n_3")])?.alias("P"),
            col("n_2").alias("Q"),
            max_horizontal([col("n_1"), col("n_3")])?.alias("R"),
        ])
        .with_columns([
            (col("Q") - col("P")).alias("x"),
            (col("R") - col("P")).alias("y"),
        ])
        .with_columns([
            sum_horizontal(
                [col(TRIACYLGLYCEROL)
                    .struct_()
                    .field_by_name("*")
                    .fatty_acid()
                    .oleic()],
                false,
            )?
            .cast(DataType::Float64)
            .alias("n_O"),
            sum_horizontal(
                [col(TRIACYLGLYCEROL)
                    .struct_()
                    .field_by_name("*")
                    .fatty_acid()
                    .equal(C18DT9.clone())],
                false,
            )?
            .cast(DataType::Float64)
            .alias("n_E"),
            sum_horizontal(
                [col(TRIACYLGLYCEROL)
                    .struct_()
                    .field_by_name("*")
                    .fatty_acid()
                    .linoleic()],
                false,
            )?
            .cast(DataType::Float64)
            .alias("n_J"),
            sum_horizontal(
                [col(TRIACYLGLYCEROL)
                    .struct_()
                    .field_by_name("*")
                    .fatty_acid()
                    .alpha_linolenic()],
                false,
            )?
            .cast(DataType::Float64)
            .alias("n_N"),
        ])
        .with_columns([
            (col("n_O") - lit(1)).clip_min(lit(0)).alias("n_OO"),
            (col("n_E") - lit(1)).clip_min(lit(0)).alias("n_EE"),
            (col("n_J") - lit(1)).clip_min(lit(0)).alias("n_JJ"),
            (col("n_N") - lit(1)).clip_min(lit(0)).alias("n_NN"),
            (col("n_O") * col("n_J")).alias("n_OJ"),
            (col("n_O") * col("n_N")).alias("n_ON"),
            (col("n_J") * col("n_N")).alias("n_JN"),
        ])
        .with_columns([
            (lit(2)
                - (-((col("x") - col("x_0")) / col("k")).pow(2)).exp()
                - (-(col("y") / col("k")).pow(2)).exp())
            .alias("f_xy"),
            any_horizontal([col(r#"^n_\d$"#).rem(lit(2))])?.alias("f_odd"),
            col("y").neq(lit(0)).alias("f_asym"),
            lit(true).alias("f_β"),
        ])
        .with_column(
            (col("h") * col("n")
                + col("h_0")
                + col("h_xy") * col("f_xy")
                + col("h_odd") * col("f_odd") * col("f_β"))
            .alias("ΔH_f^saturated"),
        )
        .with_column(
            (col("ΔH_f^saturated")
                + col("h_O") * col("n_O")
                + col("h_E") * col("n_E")
                + col("h_J") * col("n_J"))
            .alias("ΔH_f"),
        )
        .with_columns([
            (col("A_0")
                + col("A_odd") * col("f_odd")
                + col("A_x^2") * col("x").pow(2)
                + col("A_x") * col("x")
                + col("A_xy") * col("x") * col("y")
                + col("A_y") * col("y")
                + col("A_y^2") * col("y").pow(2))
            .alias("A_s"),
            (col("B_0")
                + col("B_odd") * col("f_odd")
                + col("B_x^2") * col("x").pow(2)
                + col("B_x") * col("x")
                + col("B_xy") * col("x") * col("y")
                + col("B_y") * col("y")
                + col("B_y^2") * col("y").pow(2))
            .alias("B_s"),
        ])
        .with_columns([
            (col("A_s")
                + col("A_O") * col("n_O")
                + col("A_E") * col("n_E")
                + col("A_J") * col("n_J")
                + col("A_N") * col("n_N")
                + col("A_OO") * col("n_OO")
                + col("A_EE") * col("n_EE")
                + col("A_JJ") * col("n_JJ")
                + col("A_NN") * col("n_NN")
                + col("A_OJ") * col("n_OJ")
                + col("A_ON") * col("n_ON")
                + col("A_JN") * col("n_JN"))
            .alias("A_u"),
            (col("B_s")
                + col("B_O") * col("n_O")
                + col("B_J") * col("n_J")
                + col("B_N") * col("n_N"))
            .alias("B_u"),
        ])
        .with_column(
            (col("T_∞,e")
                * (lit(1) + col("A_u") / col("n") - col("A_u") * col("B_u") / col("n").pow(2))
                - lit(T_0))
            .alias("T_f"),
        );
    if !options.intermediate {
        lazy_frame = lazy_frame.select([cols([TRIACYLGLYCEROL, "ΔH_f", "T_f"]).as_expr()]);
    }
    Ok(lazy_frame)
}

/// Options
#[derive(Clone, Copy, Debug, Default)]
pub(crate) struct Options {
    pub intermediate: bool,
    pub kind: Kind,
    pub polymorphism: Polymorphism,
}

/// Kind
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Kind {
    #[default]
    Moorthy2016,
    Seilert2021,
}

/// Polymorphism
#[derive(Clone, Copy, Debug, Default)]
pub(crate) enum Polymorphism {
    Alpha,
    BetaPrime,
    #[default]
    Beta,
}

// Большинство ограничений, описанных в предыдущем разделе, были преодолены в
// основополагающей работе Весдорпа (1990) (которая затем была переиздана в 2013
// (Marangoni & Wesdorp, 2013) и в 2016 (Moorthy et al., 2016), а затем повторно
// параметризована в 2021 (Seilert & Flöter, 2021)) и в другой модели
// Зеберг-Миккельсена и Стенби (1999).
#[cfg(test)]
mod test {
    use super::*;

    // ┌─────────────────────────────────┬────────────┬────────────┐
    // │ Triacylglycerol                 ┆ ΔH_f       ┆ T_f        │
    // │ ---                             ┆ ---        ┆ ---        │
    // │ struct[3]                       ┆ f64        ┆ f64        │
    // ╞═════════════════════════════════╪════════════╪════════════╡
    // │ {{16,[]},{16,[]},{16,[]}}       ┆ 168.022281 ┆ 64.481825  │
    // │ {{18,[{9,false,false}]},{18,[{… ┆ 100.762281 ┆ -1.324231  │
    // │ {{18,[{9,false,false}, {12,fal… ┆ 78.262281  ┆ -23.289587 │
    // │ {{18,[{9,false,false}]},{16,[]… ┆ 103.768067 ┆ 6.875577   │
    // └─────────────────────────────────┴────────────┴────────────┘
    #[test]
    fn test() -> PolarsResult<()> {
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
        let lazy_frame = triacylglycerols(
            data_frame.lazy(),
            Options {
                intermediate: false,
                kind: Kind::Moorthy2016,
                polymorphism: Polymorphism::BetaPrime,
            },
        )?;
        println!("lazy_frame: {}", lazy_frame.collect()?);
        Ok(())
    }
}
