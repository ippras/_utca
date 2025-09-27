## Polars

### `explode` `implode`

```rust
lazy_frame = lazy_frame
    .group_by_stable([col("Index"), col("Keys"), col("Species")])
    .agg([as_struct(vec![
        col("Values").explode().struct_().field_by_name("Mean").percent_if(key.percent),
        col("Values").explode().struct_().field_by_name("StandardDeviation").percent_if(key.percent),
        col("Values").explode().struct_().field_by_name("Array").percent_if(key.percent),
        // col("Values").explode().struct_().field_by_name("Repetitions").percent_if(key.percent),
    ])
    .implode()
    .alias("Values")]);
```
