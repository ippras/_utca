//! [Calculator (2017)](https://lipidlibrary.shinyapps.io/Triglyceride_Property_Calculator/)
//! [Calculator (2021)](https://tri.marangoni.tech)
//! [Moorthy2016](https://doi.org/10.1007/s11746-016-2935-1)
//! [Seilert2021](https://doi.org/10.1002/aocs.12515)

// Большинство ограничений, описанных в предыдущем разделе, были преодолены в
// основополагающей работе Весдорпа (1990) (которая затем была переиздана в 2013
// (Marangoni & Wesdorp, 2013) и в 2016 (Moorthy et al., 2016), а затем повторно
// параметризована в 2021 (Seilert & Flöter, 2021)) и в другой модели
// Зеберг-Миккельсена и Стенби (1999).
#[cfg(test)]
mod test {
    use anyhow::Result;
    use lipid::prelude::*;
    use polars::{
        lazy::dsl::{max_horizontal, min_horizontal, sum_horizontal},
        prelude::*,
    };
    use std::ops::Rem;

    // ГОСТ 8.157-75
    const T_0: f64 = 273.15;

    #[test]
    fn test() -> Result<()> {
        let df = df! {
            // STEREOSPECIFIC_NUMBERS1 => [fatty_acid!(C18 { 9 => C })?],
            // STEREOSPECIFIC_NUMBERS2 => [fatty_acid!(C16 { })?],
            // STEREOSPECIFIC_NUMBERS3 => [fatty_acid!(C18 { 9 => C, 12 => C })?],
            STEREOSPECIFIC_NUMBERS1 => [fatty_acid!(C18 { 9 => C })?],
            STEREOSPECIFIC_NUMBERS2 => [fatty_acid!(C18 { 9 => C })?],
            STEREOSPECIFIC_NUMBERS3 => [fatty_acid!(C18 { 9 => C, 12 => C })?],

            // STEREOSPECIFIC_NUMBERS1 => [fatty_acid!(C18 { 9 => C })?],
            // STEREOSPECIFIC_NUMBERS2 => [fatty_acid!(C16 { })?],
            // STEREOSPECIFIC_NUMBERS3 => [fatty_acid!(C18 { 9 => C })?],

            // STEREOSPECIFIC_NUMBERS1 => [fatty_acid!(C16 { })?],
            // STEREOSPECIFIC_NUMBERS2 => [fatty_acid!(C16 { })?],
            // STEREOSPECIFIC_NUMBERS3 => [fatty_acid!(C16 { })?],
        }?;
        println!("df: {df}");
        let lazy_frame = df
            .lazy()
            .with_columns([
                lit(3.89).alias("h"),      // #2
                lit(-17.16).alias("h_0"),  // #1
                lit(-22.29).alias("h_xy"), // #5
                lit(2.29).alias("h_odd"),  // #10
                lit(2.88).alias("k"),      // #7
                lit(0.77).alias("x_0"),    // #8
                //
                lit(-30.2).alias("h_O"), // #40
                lit(-15.9).alias("h_E"), // #41
                lit(-37.7).alias("h_J"), // #42
                //
                lit(401.15).alias("T_∞,e"),
                //
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
                // A
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
                // B
                lit(-3.7).alias("B_O"),
                lit(-1.5).alias("B_J"),
                lit(-1.8).alias("B_N"),
            ])
            .with_columns([
                col(STEREOSPECIFIC_NUMBERS1)
                    .fatty_acid()
                    .carbon()
                    .cast(DataType::Float64)
                    .alias("n_1"),
                col(STEREOSPECIFIC_NUMBERS2)
                    .fatty_acid()
                    .carbon()
                    .cast(DataType::Float64)
                    .alias("n_2"),
                col(STEREOSPECIFIC_NUMBERS3)
                    .fatty_acid()
                    .carbon()
                    .cast(DataType::Float64)
                    .alias("n_3"),
                col(STEREOSPECIFIC_NUMBERS1)
                    .fatty_acid()
                    .unsaturation()
                    .cast(DataType::Float64)
                    .alias("u_1"),
                col(STEREOSPECIFIC_NUMBERS2)
                    .fatty_acid()
                    .unsaturation()
                    .cast(DataType::Float64)
                    .alias("u_2"),
                col(STEREOSPECIFIC_NUMBERS3)
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
                    [col(r#"^StereospecificNumbers\d$"#).fatty_acid().oleic()],
                    false,
                )?
                .cast(DataType::Float64)
                .alias("n_O"),
                sum_horizontal(
                    [col(r#"^StereospecificNumbers\d$"#)
                        .fatty_acid()
                        .equal(C18DT9.clone())],
                    false,
                )?
                .cast(DataType::Float64)
                .alias("n_E"),
                sum_horizontal(
                    [col(r#"^StereospecificNumbers\d$"#).fatty_acid().linoleic()],
                    false,
                )?
                .cast(DataType::Float64)
                .alias("n_J"),
                sum_horizontal(
                    [col(r#"^StereospecificNumbers\d$"#)
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
                .alias("ΔH_f^unsaturated"),
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
                .alias("T_f^unsaturated"),
            )
            .select([
                col("ΔH_f^unsaturated"),
                col("A_s"),
                col("A_u"),
                col("B_u"),
                col("T_f^unsaturated"),
            ]);
        // ΔH_sat = h * n + h_0 + h_xy * f_xy + h_odd * f_odd * f_β
        // Δ*H*<sup>unsat</sup> = Δ*H*<sup>sat</sup> + *h*ₒ*n*ₒ + *h*ⱼ*n*ⱼ
        println!("lazy_frame: {}", lazy_frame.collect()?);
        Ok(())
    }
}
// *   *n*₁ = 18, *u*₁ = 1
// *   *n*₂ = 16, *u*₂ = 0
// *   *n*₃ = 18, *u*₃ = 2
