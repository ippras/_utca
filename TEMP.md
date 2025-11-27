    let stereospecific_numbers = |expr: Expr| -> PolarsResult<Expr> {
        Ok(as_struct(vec![
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .monounsaturated(value)
                            .alias("Monounsaturated")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .polyunsaturated(value)
                            .alias("Polyunsaturated")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .saturated(value)
                            .alias("Saturated")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| col(FATTY_ACID).fatty_acid().trans(value).alias("Trans"))
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .unsaturated(value, None)
                            .alias("Unsaturated")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .unsaturated(value, NonZeroI8::new(-9))
                            .alias("Unsaturated_9")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .unsaturated(value, NonZeroI8::new(-6))
                            .alias("Unsaturated_6")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .unsaturated(value, NonZeroI8::new(-3))
                            .alias("Unsaturated_3")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .unsaturated(value, NonZeroI8::new(9))
                            .alias("Unsaturated9")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .eicosapentaenoic_and_docosahexaenoic(value)
                            .alias("EicosapentaenoicAndDocosahexaenoic")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .fish_lipid_quality(value)
                            .alias("FishLipidQuality")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .health_promoting_index(value)
                            .alias("HealthPromotingIndex")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .hypocholesterolemic_to_hypercholesterolemic(value)
                            .alias("HypocholesterolemicToHypercholesterolemic")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .index_of_atherogenicity(value)
                            .alias("IndexOfAtherogenicity")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .index_of_thrombogenicity(value)
                            .alias("IndexOfThrombogenicity")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .linoleic_to_alpha_linolenic(value)
                            .alias("LinoleicToAlphaLinolenic")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .polyunsaturated_6_to_polyunsaturated_3(value)
                            .alias("Polyunsaturated_6ToPolyunsaturated_3")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .polyunsaturated_to_saturated(value)
                            .alias("PolyunsaturatedToSaturated")
                    })
                    .collect(),
            )?,
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        col(FATTY_ACID)
                            .fatty_acid()
                            .unsaturation_index(value)
                            .alias("UnsaturationIndex")
                    })
                    .collect(),
            )?,
            // P
            concat_arr(
                iter(expr.clone())
                    .map(|value| {
                        (value
                            * col(FATTY_ACID)
                                .fatty_acid()
                                .iodine_value()
                                .alias("IodineValue"))
                        .sum()
                        .alias("IodineValue")
                    })
                    .collect(),
            )?,
        ]))
    };