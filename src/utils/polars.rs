use polars::prelude::Expr;

pub fn hash(expr: Expr) -> Expr {
    expr.hash(1, 2, 3, 4).alias("Hash")
}