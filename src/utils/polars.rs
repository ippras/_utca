use polars::prelude::*;

/// Extension methods for [`StructNameSpace`]
pub trait ExprExt {
    fn first_non_nan_struct_field(self) -> Expr;
}

impl ExprExt for Expr {
    fn first_non_nan_struct_field(self) -> Expr {
        self.clone()
            .struct_()
            .field_by_index(0)
            .fill_nan(self.struct_().field_by_index(1))
    }
}

pub fn hash(expr: Expr) -> Expr {
    expr.hash(1, 2, 3, 4).alias("Hash")
}
