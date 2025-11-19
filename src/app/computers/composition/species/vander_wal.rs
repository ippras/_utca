use lipid::prelude::*;
use polars::prelude::*;

// 1,3-sn 2-sn 1,2,3-sn
// PSC:
// [abc] = 2*[a_{13}]*[_b2]*[c_{13}]
// [aab] = 2*[a_{13}]*[a_2]*[b13]
// [aba] = [a13]^2*[b2]
// `2*[a_{13}]` - потому что зеркальные ([abc]=[cba], [aab]=[baa]).
// SSC: [abc] = [a_{13}]*[b_2]*[c_{13}]
pub(super) fn compute(mut lazy_frame: LazyFrame) -> PolarsResult<LazyFrame> {
    // Cartesian product (TAG from FA)
    lazy_frame = lazy_frame
        .clone()
        .select([as_struct(vec![
            col(LABEL),
            col(FATTY_ACID),
            col(STEREOSPECIFIC_NUMBERS13).alias("Value"),
        ])
        .alias(STEREOSPECIFIC_NUMBERS1)])
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col(LABEL),
                col(FATTY_ACID),
                col(STEREOSPECIFIC_NUMBERS2).alias("Value"),
            ])
            .alias(STEREOSPECIFIC_NUMBERS2)]),
            None,
        )
        .cross_join(
            lazy_frame.clone().select([as_struct(vec![
                col(LABEL),
                col(FATTY_ACID),
                col(STEREOSPECIFIC_NUMBERS13).alias("Value"),
            ])
            .alias(STEREOSPECIFIC_NUMBERS3)]),
            None,
        );
    // Restruct
    let label = |name| col(name).struct_().field_by_name(LABEL).alias(name);
    let fatty_acid = |name| col(name).struct_().field_by_name(FATTY_ACID).alias(name);
    let value = |name| col(name).struct_().field_by_name("Value");
    lazy_frame = lazy_frame.select([
        as_struct(vec![
            label(STEREOSPECIFIC_NUMBERS1),
            label(STEREOSPECIFIC_NUMBERS2),
            label(STEREOSPECIFIC_NUMBERS3),
        ])
        .alias(LABEL),
        as_struct(vec![
            fatty_acid(STEREOSPECIFIC_NUMBERS1),
            fatty_acid(STEREOSPECIFIC_NUMBERS2),
            fatty_acid(STEREOSPECIFIC_NUMBERS3),
        ])
        .alias(TRIACYLGLYCEROL),
        value(STEREOSPECIFIC_NUMBERS1)
            * value(STEREOSPECIFIC_NUMBERS2)
            * value(STEREOSPECIFIC_NUMBERS3),
    ]);
    Ok(lazy_frame)
}
