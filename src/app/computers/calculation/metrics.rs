use crate::{
    app::parameters::{Metric, Parameters},
    utils::HashedDataFrame,
};
use egui::util::cache::{ComputerMut, FrameCache};
use polars::{error::PolarsResult, prelude::*};
use std::{
    f64::consts::{E, FRAC_1_SQRT_2},
    hash::{Hash, Hasher},
};
use tracing::instrument;

/// Metrics computed
pub(crate) type Computed = FrameCache<Value, Computer>;

/// Metrics computer
#[derive(Default)]
pub(crate) struct Computer;

impl Computer {
    #[instrument(skip(self), err)]
    fn try_compute(&mut self, key: Key) -> PolarsResult<DataFrame> {
        let mut lazy_frame = key.hashed_data_frame.data_frame.clone().lazy();
        // println!("Metrics 0: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame = lazy_frame.select([all().exclude_cols(["Composition", "Species"]).as_expr()]);
        let schema = lazy_frame.collect_schema()?;
        let mean = |expr: Expr| expr.struct_().field_by_name("Mean").fill_null(0);
        let exprs = schema
            .iter_names_cloned()
            .map(|left| -> PolarsResult<_> {
                Ok(concat_arr(vec![match key.parameters.metric {
                    // Similarity between two discrete probability distributions
                    Metric::HellingerDistance => {
                        hellinger_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    Metric::JensenShannonDistance => {
                        jensen_shannon_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    Metric::BhattacharyyaDistance => {
                        bhattacharyya_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    // Distance between two points
                    Metric::ChebyshevDistance => {
                        chebyshev_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    Metric::EuclideanDistance => {
                        euclidean_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    Metric::ManhattanDistance => {
                        manhattan_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    // Distance between two series
                    Metric::CosineDistance => {
                        cosine_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    Metric::JaccardDistance => {
                        jaccard_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    Metric::OverlapDistance => {
                        overlap_distance(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    // Correlation between two series
                    Metric::PearsonCorrelation => {
                        pearson_corr(mean(col(left.clone())), mean(all().as_expr()))
                    }
                    Metric::SpearmanRankCorrelation => {
                        spearman_rank_corr(mean(col(left.clone())), mean(all().as_expr()), false)
                    }
                }])?
                .alias(left))
            })
            .collect::<PolarsResult<Vec<_>>>()?;
        lazy_frame = lazy_frame.select(exprs).explode(all());
        // lazy_frame = lazy_frame
        //     .select([
        //         nth(2)
        //             .as_expr()
        //             .struct_()
        //             .field_by_name("Mean")
        //             .fill_null(0)
        //             .alias(LEFT),
        //         nth(3)
        //             .as_expr()
        //             .struct_()
        //             .field_by_name("Mean")
        //             .fill_null(0)
        //             .alias(RIGHT),
        //     ])
        //     .select([
        //         // Similarity between two data points
        //         euclidean_distance(col(LEFT), col(RIGHT)).alias("EuclideanDistance"),
        //         chebyshev_distance(col(LEFT), col(RIGHT)).alias("ChebyshevDistance"),
        //         manhattan_distance(col(LEFT), col(RIGHT)).alias("ManhattanDistance"),
        //         // Similarity between two sets
        //         cosine_distance(col(LEFT), col(RIGHT)).alias("CosineDistance"),
        //         jaccard_distance(col(LEFT), col(RIGHT))?.alias("JaccardDistance"),
        //         // Similarity between two probability distributions
        //         bhattacharyya_distance(col(LEFT), col(RIGHT)).alias("BhattacharyyaDistance"),
        //         hellinger_distance(col(LEFT), col(RIGHT)).alias("HellingerDistance"),
        //         jensen_shannon_distance(col(LEFT), col(RIGHT)).alias("JensenShannonDistance"),
        //         // Correlation between two
        //         pearson_corr(col(LEFT), col(RIGHT)).alias("PearsonCorrelation"),
        //         spearman_rank_corr(col(LEFT), col(RIGHT), false).alias("SpearmanRankCorrelation"),
        //     ]);

        // println!("Metrics 1: {}", lazy_frame.clone().collect().unwrap());
        lazy_frame.collect()
    }
}

impl ComputerMut<Key<'_>, Value> for Computer {
    fn compute(&mut self, key: Key) -> Value {
        self.try_compute(key).unwrap()
    }
}

/// Metrics key
#[derive(Clone, Copy, Debug)]
pub(crate) struct Key<'a> {
    pub(crate) hashed_data_frame: &'a HashedDataFrame,
    pub(crate) parameters: &'a Parameters,
}

impl Hash for Key<'_> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.hashed_data_frame.hash(state);
        self.parameters.hash(state);
    }
}

/// Metrics value
type Value = DataFrame;

// fn hierarchical_cluster(data_frame: DataFrame) {
// use linfa::{Dataset, DatasetBase, dataset::Records, traits::Transformer as _};
// use linfa_hierarchical::HierarchicalCluster;
// use linfa_kernel::{Kernel, KernelMethod};
//
//     // let dataset = linfa_datasets::iris();
//     let array = data_frame
//         .to_ndarray::<Float64Type>(IndexOrder::default())
//         .unwrap();
//     let dataset = Dataset::new(array, Default::default());
//     // Dataset::new(data, targets)
//     //     .map_targets(|x| *x as usize)
//     //     .with_feature_names(feature_names);
//     // let t = DatasetBase::from(data_frame.to_ndarray(IndexOrder::default()).unwrap());
//     let kernel = Kernel::params()
//         .method(KernelMethod::Gaussian(1.0))
//         .transform(array);
//     // let kernel = HierarchicalCluster::default()
//     //     .num_clusters(3)
//     //     .transform(kernel)
//     //     .unwrap();
//     // for (id, target) in kernel
//     //     .targets()
//     //     .iter()
//     //     .zip(dataset.into().targets().into_iter())
//     // {
//     //     let name = match *target {
//     //         0 => "setosa",
//     //         1 => "versicolor",
//     //         2 => "virginica",
//     //         _ => unreachable!(),
//     //     };
//     //     print!("({id} {name}) ");
//     // }
// }

fn overlap_distance(a: Expr, b: Expr) -> Expr {
    lit(1) - min(a.clone(), b.clone()).sum() / min(a.sum(), b.sum())
}

fn sørensen_coefficient(a: Expr, b: Expr) -> Expr {
    lit(1) - lit(2) * min(a.clone(), b.clone()).sum() / (a.sum() + b.sum())
}

fn bhattacharyya_distance(a: Expr, b: Expr) -> Expr {
    -(a * b).sqrt().sum().log(lit(E))
}

fn hellinger_distance(a: Expr, b: Expr) -> Expr {
    lit(FRAC_1_SQRT_2) * (a.sqrt() - b.sqrt()).pow(2).sum().sqrt()
}

fn euclidean_distance(a: Expr, b: Expr) -> Expr {
    (a - b).pow(2).sum().sqrt()
}

fn chebyshev_distance(a: Expr, b: Expr) -> Expr {
    (a - b).abs().max()
}

fn manhattan_distance(a: Expr, b: Expr) -> Expr {
    (a - b).abs().sum()
}

fn cosine_distance(a: Expr, b: Expr) -> Expr {
    lit(1) - (a.clone() * b.clone()).sum() / (a.pow(2).sum().sqrt() * b.pow(2).sum().sqrt())
}

fn bray_curtis_dissimilarity(a: Expr, b: Expr) -> Expr {
    (a.clone() - b.clone()).abs().sum() / (a + b).sum()
}

fn jaccard_distance(a: Expr, b: Expr) -> Expr {
    // Ok(lit(1) - min_horizontal([a.clone(), b.clone()])?.sum() / max_horizontal([a, b])?.sum())
    lit(1) - min(a.clone(), b.clone()).sum() / max(a, b).sum()
}

fn jensen_shannon_distance(mut a: Expr, mut b: Expr) -> Expr {
    pub const SQRT_LN_2: f64 = 0.83255461115769768820626950400765053927898406982421875_f64;

    fn kullback_leibler_divergence(a: Expr, b: Expr) -> Expr {
        (a.clone() * (a / b).log(lit(E))).fill_nan(0).sum()
    }

    a = a.clone() / a.sum();
    b = b.clone() / b.sum();
    let m = (a.clone() + b.clone()) / lit(2);
    (lit(0.5) * kullback_leibler_divergence(a, m.clone())
        + lit(0.5) * kullback_leibler_divergence(b, m))
    .sqrt()
        / lit(SQRT_LN_2)
}

fn max(a: Expr, b: Expr) -> Expr {
    ternary_expr(a.clone().gt_eq(b.clone()), a, b)
}

fn min(a: Expr, b: Expr) -> Expr {
    ternary_expr(a.clone().lt_eq(b.clone()), a, b)
}
