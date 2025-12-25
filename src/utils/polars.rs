use crate::r#const::{MEAN, SAMPLE, STANDARD_DEVIATION};
use polars::prelude::*;
use polars_ext::expr::ExprExt as _;
use std::sync::LazyLock;

pub const MEAN_AND_STANDARD_DEVIATION: LazyLock<DataType> = LazyLock::new(|| {
    DataType::Struct(vec![
        Field::new(PlSmallStr::from_static(MEAN), DataType::Float64),
        Field::new(
            PlSmallStr::from_static(STANDARD_DEVIATION),
            DataType::Float64,
        ),
        Field::new(
            PlSmallStr::from_static(SAMPLE),
            DataType::Array(Box::new(DataType::Float64), 0),
        ),
    ])
});

#[derive(Clone, Copy, Debug, Default)]
pub struct MeanAndStandardDeviationOptions {
    pub(crate) ddof: u8,
    pub(crate) percent: bool,
    pub(crate) precision: usize,
    pub(crate) significant: bool,
}

pub fn mean_and_standard_deviation(
    array: Expr,
    options: impl Into<MeanAndStandardDeviationOptions>,
) -> Expr {
    let options = options.into();
    as_struct(vec![
        array
            .clone()
            .arr()
            .mean()
            .percent(options.percent)
            .precision(options.precision, options.significant)
            .alias(MEAN),
        array
            .clone()
            .arr()
            .std(options.ddof)
            .percent(options.percent)
            .precision(options.precision + 1, options.significant)
            .alias(STANDARD_DEVIATION),
        array
            .arr()
            .eval(
                element()
                    .percent(options.percent)
                    .precision(options.precision, options.significant),
                false,
            )
            .alias(SAMPLE),
    ])
}

/// Extension methods for [`Schema`]
pub trait SchemaExt {
    fn array_lengths_recursive(&self) -> PolarsResult<Vec<usize>>;
}

impl SchemaExt for Schema {
    fn array_lengths_recursive(&self) -> PolarsResult<Vec<usize>> {
        let mut lengths = Vec::new();
        for data_type in self.iter_values() {
            check_array_lengths(data_type, &mut lengths);
        }
        Ok(lengths)
    }
}

pub fn check_array_lengths(dtype: &DataType, lengths: &mut Vec<usize>) {
    match dtype {
        DataType::Struct(fields) => {
            for field in fields {
                check_array_lengths(field.dtype(), lengths);
            }
        }
        DataType::Array(_, length) => lengths.push(*length),
        _ => {}
    }
}

pub(crate) fn format_standard_deviation(expr: Expr) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        expr.clone().is_not_null(),
        format_str("±{}", [expr])?,
        lit(NULL),
    ))
}

pub(crate) fn format_sample(expr: Expr) -> PolarsResult<Expr> {
    Ok(ternary_expr(
        expr.clone().arr().len().gt(1),
        format_str(
            "[{}]",
            [expr
                .arr()
                .eval(element(), false)
                .arr()
                .join(lit(", "), false)],
        )?,
        lit(NULL),
    ))
}

// /// Format
// pub trait Format {
//     fn percent(&self) -> bool;

//     fn precision(&self) -> usize;

//     fn significant(&self) -> bool;
// }

// impl Format for Key<'_> {
//     fn percent(&self) -> bool {
//         self.percent
//     }

//     fn precision(&self) -> usize {
//         self.precision
//     }

//     fn significant(&self) -> bool {
//         self.significant
//     }
// }

// fn temp_float(expr: Expr, format: impl Format) -> Expr {
//     expr.percent(format.percent())
//         .precision(format.precision(), format.significant())
// }
