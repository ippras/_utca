## Polars

### `explode` `implode`

```rust
lazy_frame = lazy_frame
    .group_by_stable([col("Index"), col("Keys"), col("Species")])
    .agg([as_struct(vec![
        col("Values").explode().struct_().field_by_name("Mean").percent(key.percent),
        col("Values").explode().struct_().field_by_name("StandardDeviation").percent(key.percent),
        col("Values").explode().struct_().field_by_name("Array").percent(key.percent),
        // col("Values").explode().struct_().field_by_name("Repetitions").percent(key.percent),
    ])
    .implode()
    .alias("Values")]);
```

### `horizontal`

```rust
use polars::lazy::dsl::sum_horizontal,
```

= Polars

<https://stackoverflow.com/questions/tagged/rust-polars>

== Create DataFrames

[source, rust]
let data_frame = df! {
    "STRING" => &[""],
    "FLOAT" => &[0.0],
    "EMPTY_LIST" => &[Series::from_iter(empty::<u8>())],
    "LIST" => &[Series::from_iter([0, 1, 2, 3, 4, 5, 6, 7, 8, 9])],
}?;

== Create list of structs

<https://stackoverflow.com/questions/73807745/list-of-structs-in-rust-polars-column>

[source, rust]
let foo = StructChunked::new("foo", &[
    Series::new("f1", ["a", "c", "e"]),
    Series::new("f2", ["b", "d", "f"]),
]).unwrap();

== Delete row

[source, rust]
let data_frame = data_frame
    .slice(0, row_index)
    .vstack(&data_frame.slice(row_index + 1, usize::MAX))?;

[source, rust]
pub(crate) fn delete_row(&mut self, row: usize) -> PolarsResult<()> {
    self.data_frame = self
        .data_frame
        .slice(0, row)
        .vstack(&self.data_frame.slice((row + 1) as _, usize::MAX))?;
    self.data_frame.as_single_chunk_par();
    Ok(())
}

* <https://stackoverflow.com/questions/71486019/how-to-drop-row-in-polars-python>
* <https://stackoverflow.com/a/71495211/1522758>

== Index list (enumerate)

<https://github.com/pola-rs/polars/pull/13478>

[source, rust]
expr.list()
    .eval(col("").cum_count(false), false);

[source, toml]
features = ["cum_agg"]

== Filtering by data in another DataFrame

https://stackoverflow.com/questions/72546690/lightweight-syntax-for-filtering-a-polars-dataframe-on-a-multi-column-key

* <https://stackoverflow.com/questions/76523245/how-to-get-current-index-of-element-in-polars-list>

// link:https://stackoverflow.com/questions/73717556/how-to-swap-column-values-on-conditions-in-python-polars[Swap column values on conditions]
// link:https://github.com/pola-rs/polars/issues/15894[Expand List & Array to columns]

// link:https://github.com/pola-rs/polars/issues/16110[sort an array of structs]
// link:https://stackoverflow.com/questions/78440430/sorting-a-polars-liststruct-by-struct-value[Sorting a polars list[struct[]] by struct value]



== See also

* link:https://www.rhosignal.com/posts/polars-pandas-cheatsheet[polars-pandas cheat sheet]
* link:https://franzdiebold.github.io/polars-cheat-sheet/Polars_cheat_sheet.pdf[polars cheat sheet]