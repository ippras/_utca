use polars::prelude::*;

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
