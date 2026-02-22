use rayon::prelude::*;

/// Calculate coverage statistics from Vec<Vec> - for H5 reading
pub fn calculate_coverage_stats_from_vec(
    cov_matrix: &[Vec<u16>],
    sample_names: &[String],
) -> Vec<SampleStats> {
    use rayon::prelude::*;

    sample_names
        .par_iter()
        .enumerate()
        .map(|(j, name)| {
            let sample_cov = &cov_matrix[j];
            let covered: Vec<u16> = sample_cov.iter().copied().filter(|&x| x > 0).collect();

            SampleStats {
                sample_name: name.clone(),
                n_covered: covered.len(),
                n_total: sample_cov.len(),
                mean_coverage: if !covered.is_empty() {
                    covered.iter().map(|&x| x as f32).sum::<f32>() / covered.len() as f32
                } else {
                    0.0
                },
                coverage_distribution: calculate_distribution(&covered),
            }
        })
        .collect()
}

/// Calculate coverage statistics - ported from R::get_stats
pub fn calculate_coverage_stats(
    cov_matrix: &ndarray::Array2<u16>,
    sample_names: &[String],
) -> Vec<SampleStats> {
    let (_, n_samples) = cov_matrix.dim();

    (0..n_samples)
        .into_par_iter()
        .map(|j| {
            let sample_cov: Vec<u16> = cov_matrix.column(j).to_vec();
            let covered: Vec<u16> = sample_cov.into_iter().filter(|&x| x > 0).collect();

            SampleStats {
                sample_name: sample_names[j].clone(),
                n_covered: covered.len(),
                n_total: cov_matrix.nrows(),
                mean_coverage: if !covered.is_empty() {
                    covered.iter().map(|&x| x as f32).sum::<f32>() / covered.len() as f32
                } else {
                    0.0
                },
                coverage_distribution: calculate_distribution(&covered),
            }
        })
        .collect()
}

#[derive(Debug, Clone)]
pub struct SampleStats {
    pub sample_name: String,
    pub n_covered: usize,
    pub n_total: usize,
    pub mean_coverage: f32,
    pub coverage_distribution: Vec<(u16, usize)>, // (threshold, count)
}

pub fn calculate_distribution(covered: &[u16]) -> Vec<(u16, usize)> {
    let thresholds = [1, 2, 3, 4, 5, 10];
    thresholds
        .iter()
        .map(|&thr| {
            let count = covered.iter().filter(|&&x| x >= thr).count();
            (thr, count)
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
        assert_eq!(stats[0].n_covered, 2); // Both samples covered
        assert_eq!(stats[1].n_covered, 2);
    }

    #[test]
    fn test_coverage_distribution() {
        let covered = vec![1u16, 2, 5, 10, 15];
        let dist = calculate_distribution(&covered);

        assert_eq!(dist.len(), 6); // 6 thresholds
        assert_eq!(dist[0].0, 1); // First threshold
        assert_eq!(dist[0].1, 5); // All >= 1
        assert_eq!(dist[5].0, 10); // Last threshold
        assert_eq!(dist[5].1, 2); // Only 2 >= 10
    }
}
