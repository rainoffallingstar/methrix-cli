/// Calculate coverage statistics from sample-major coverage vectors.
pub fn calculate_coverage_stats_from_vec(
    cov_matrix: &[Vec<u32>],
    sample_names: &[String],
) -> Vec<SampleStats> {
    sample_names
        .iter()
        .enumerate()
        .map(|(sample_index, name)| {
            let sample_cov = &cov_matrix[sample_index];
            calculate_sample_stats(name.clone(), sample_cov.len(), sample_cov.iter().copied())
        })
        .collect()
}

/// Calculate coverage statistics without allocating a full temporary column
/// or a second vector containing only covered values.
pub fn calculate_coverage_stats(
    cov_matrix: &ndarray::Array2<u32>,
    sample_names: &[String],
) -> Vec<SampleStats> {
    let (_, n_samples) = cov_matrix.dim();

    (0..n_samples)
        .map(|sample_index| {
            calculate_sample_stats(
                sample_names[sample_index].clone(),
                cov_matrix.nrows(),
                cov_matrix.column(sample_index).iter().copied(),
            )
        })
        .collect()
}

fn calculate_sample_stats(
    sample_name: String,
    n_total: usize,
    coverage_values: impl Iterator<Item = u32>,
) -> SampleStats {
    let thresholds = [1, 2, 3, 4, 5, 10];
    let mut distribution_counts = [0usize; 6];
    let mut n_covered = 0usize;
    let mut coverage_sum = 0u64;

    for coverage in coverage_values {
        if coverage == 0 {
            continue;
        }
        n_covered += 1;
        coverage_sum += u64::from(coverage);
        for (threshold_index, threshold) in thresholds.iter().enumerate() {
            if coverage >= *threshold {
                distribution_counts[threshold_index] += 1;
            }
        }
    }

    SampleStats {
        sample_name,
        n_covered,
        n_total,
        mean_coverage: if n_covered > 0 {
            coverage_sum as f32 / n_covered as f32
        } else {
            0.0
        },
        coverage_distribution: thresholds.into_iter().zip(distribution_counts).collect(),
    }
}

#[derive(Debug, Clone)]
pub struct SampleStats {
    pub sample_name: String,
    pub n_covered: usize,
    pub n_total: usize,
    pub mean_coverage: f32,
    pub coverage_distribution: Vec<(u32, usize)>,
}

pub fn calculate_distribution(covered: &[u32]) -> Vec<(u32, usize)> {
    let thresholds = [1, 2, 3, 4, 5, 10];
    thresholds
        .iter()
        .map(|&threshold| {
            let count = covered
                .iter()
                .filter(|&&coverage| coverage >= threshold)
                .count();
            (threshold, count)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calculate_coverage_stats() {
        let mut cov = ndarray::Array2::zeros((3, 2));
        cov[(0, 0)] = 5;
        cov[(0, 1)] = 10;
        cov[(1, 0)] = 2;
        cov[(1, 1)] = 0;
        cov[(2, 0)] = 0;
        cov[(2, 1)] = 1;

        let sample_names = vec!["sample1".to_string(), "sample2".to_string()];

        let stats = calculate_coverage_stats(&cov, &sample_names);

        assert_eq!(stats.len(), 2);
        assert_eq!(stats[0].sample_name, "sample1");
        assert_eq!(stats[0].n_covered, 2);
        assert_eq!(stats[0].mean_coverage, 3.5);
        assert_eq!(stats[1].n_covered, 2);
        assert_eq!(stats[1].mean_coverage, 5.5);
    }

    #[test]
    fn vector_and_ndarray_stats_are_equivalent() {
        let cov = ndarray::Array2::from_shape_vec((3, 2), vec![5, 10, 2, 0, 0, 1]).unwrap();
        let sample_names = vec!["sample1".to_string(), "sample2".to_string()];
        let vector_stats =
            calculate_coverage_stats_from_vec(&[vec![5, 2, 0], vec![10, 0, 1]], &sample_names);
        let array_stats = calculate_coverage_stats(&cov, &sample_names);

        for (vector, array) in vector_stats.iter().zip(array_stats.iter()) {
            assert_eq!(vector.n_covered, array.n_covered);
            assert_eq!(vector.n_total, array.n_total);
            assert_eq!(vector.mean_coverage, array.mean_coverage);
            assert_eq!(vector.coverage_distribution, array.coverage_distribution);
        }
    }

    #[test]
    fn test_coverage_distribution() {
        let covered = vec![1u32, 2, 5, 10, 15];
        let dist = calculate_distribution(&covered);

        assert_eq!(dist.len(), 6);
        assert_eq!(dist[0].0, 1);
        assert_eq!(dist[0].1, 5);
        assert_eq!(dist[5].0, 10);
        assert_eq!(dist[5].1, 2);
    }
}
