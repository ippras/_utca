use lipid::prelude::*;
use polars::prelude::*;

pub(super) fn compute(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    println!("lazy_frame g0: {}", lazy_frame.clone().collect().unwrap());
    lazy_frame = lazy_frame.select([
        col(LABEL),
        col(FATTY_ACID),
        col(STEREOSPECIFIC_NUMBERS123) * lit(100),
        col(STEREOSPECIFIC_NUMBERS2) * lit(100),
    ]);
    println!("lazy_frame g1: {}", lazy_frame.clone().collect().unwrap());
    let s = || {
        col(STEREOSPECIFIC_NUMBERS123)
            .filter(col(FATTY_ACID).fatty_acid().is_saturated())
            .sum()
    };
    let s2 = || {
        col(STEREOSPECIFIC_NUMBERS2)
            .filter(col(FATTY_ACID).fatty_acid().is_saturated())
            .sum()
            .alias("S[2]")
    };
    let as3 = || lit(-9) * s().pow(2) * s2() + lit(6) * s() * s2().pow(2) - s2().pow(3);
    let cs3 = || lit(0) * lit(10).pow(-4);
    let as2u = || {
        (lit(600) * s() * s2() - lit(18) * s() * s2().pow(2) + lit(27) * s().pow(2) * s2()
            - lit(900) * s().pow(2)
            - lit(100) * s2().pow(2)
            + lit(3) * s2().pow(3))
        .alias("A")
    };
    let cs2u = || lit(300) * s() * s2();
    let alpha = |a: Expr, c: Expr| {
        lit(0.5) + (a.clone().pow(2) - lit(4) * a.clone() * c).sqrt() / (lit(2) * a)
    };
    let s2u = lit(7.03);
    let su2 = lit(36.08);
    let s2u_to_su2 = || {
        (s2u / su2) * lit(-1200) * s() * s2() + lit(18) * s() * s2().pow(2)
            - lit(27) * s().pow(2) * s2()
            + lit(1800) * s().pow(2)
            + lit(200) * s2().pow(2)
            - lit(3) * s2().pow(3)
            - lit(600) * s() * s2()
            - lit(18) * s() * s2().pow(2)
            - lit(27) * s().pow(2) * s2()
            + lit(900) * s().pow(2)
            + lit(100) * s2().pow(2)
            - lit(3) * s2().pow(3)
    };
    lazy_frame = lazy_frame.with_columns([
        //
        s().alias("S"),
        as3().alias("AS_3"),
        as2u().alias("AS_2U"),
        s2u_to_su2().alias("S2U/SU2"),
    ]);
    println!("lazy_frame g2: {}", lazy_frame.clone().collect().unwrap());
    // $(S_2U)$:
    // * $A = 600SS_{[2]} - 18SS_{[2]}^3 + 27S^2S_{[2]} – 900S^2 – 100S_{[2]}^2 + 3S^3$
    // * $C = 300SS_{[2]} - 100S_{[2]} - (S_2U) \cdot 10^4$

    // let a = -9 * col("S").powi(2) * S_{[2]} + 6 * SS_{[2]}^2 - S_{[2]}^3;
    // let alpha = 0.5 + (A ^ 2 - 4AC).sqrt() / 2A;
    Ok(lazy_frame)
}

// 1.92 + 2.85 + 0.79 + 1.47 = 7.03
// 0.55 + 1.32 + 0.44 + 1.42 = 3.73
