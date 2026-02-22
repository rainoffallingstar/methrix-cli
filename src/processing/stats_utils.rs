use ndarray::Array2;

/// Calculate coverage statistics from vector representation
pub fn calculate_coverage_stats_from_vec(
    cov_matrix: &[Vec<u16>],
    sample_names: &[String],
) -> Vec<super::stats::SampleStats> {
    let n_samples = cov_matrix.len();

    (0..n_samples)
        .map(|j| {
            let sample_cov = &cov_matrix[j];
            let covered: Vec<u16> = sample_cov.iter().copied().filter(|&x| x > 0).collect();

            super::stats::SampleStats {
                sample_name: sample_names[j].clone(),
                n_covered: covered.len(),
                n_total: sample_cov.len(),
                mean_coverage: if !covered.is_empty() {
                    covered.iter().map(|&x| x as f32).sum::<f32>() / covered.len() as f32
                } else {
                    0.0
                },
                coverage_distribution: super::stats::calculate_distribution(&covered),
            }
        })
        .collect()
}

pub fn calculate_distribution_ref(covered: &[&u16]) -> Vec<(u16, usize)> {
    let thresholds = [1, 2, 3, 4, 5, 10];
    thresholds
        .iter()
        .map(|&thr| {
            let count = covered.iter().filter(|&&&x| x >= thr).count();
            (thr, count)
        })
        .collect()
}
