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
