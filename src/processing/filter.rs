use ndarray::Array2;
use rayon::prelude::*;

trait SeparatedString {
    fn separated_string(&self) -> String;
}

impl SeparatedString for usize {
    fn separated_string(&self) -> String {
        let s = self.to_string();
        let mut result = String::new();
        let chars: Vec<char> = s.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i).is_multiple_of(3) {
                result.push(',');
            }
            result.push(*c);
        }
        result
    }
}

pub type FilteredMatrices = (Array2<f32>, Array2<u32>, Vec<usize>);

/// Remove uncovered loci - ported from R::remove_uncovered
/// Returns (filtered_beta, filtered_cov, covered_indices)
pub fn remove_uncovered(
    beta_matrix: Array2<f32>,
    cov_matrix: Array2<u32>,
) -> Result<FilteredMatrices, anyhow::Error> {
    let (n_cpgs, n_samples) = cov_matrix.dim();

    // Find CpGs covered in at least one sample
    let covered_mask: Vec<bool> = (0..n_cpgs)
        .into_par_iter()
        .map(|i| (0..n_samples).any(|j| cov_matrix[(i, j)] > 0))
        .collect();

    let n_covered = covered_mask.iter().filter(|&&x| x).count();

    if n_covered == 0 {
        anyhow::bail!("No CpGs have coverage in any sample");
    }

    println!(
        "-Removed {} uncovered CpGs (retained {})",
        (n_cpgs - n_covered).separated_string(),
        n_covered.separated_string()
    );

    // Collect covered indices
    let covered_indices: Vec<usize> = covered_mask
        .iter()
        .enumerate()
        .filter_map(|(i, &covered)| if covered { Some(i) } else { None })
        .collect();

    // Filter matrices
    let mut filtered_beta = Array2::zeros((n_covered, n_samples));
    let mut filtered_cov = Array2::zeros((n_covered, n_samples));

    for (out_idx, &in_idx) in covered_indices.iter().enumerate() {
        for j in 0..n_samples {
            filtered_beta[(out_idx, j)] = beta_matrix[(in_idx, j)];
            filtered_cov[(out_idx, j)] = cov_matrix[(in_idx, j)];
        }
    }

    Ok((filtered_beta, filtered_cov, covered_indices))
}

/// Coverage filter - ported from R::coverage_filter
pub fn coverage_filter(cov_matrix: &Array2<u32>, cov_thr: u32, min_samples: usize) -> Vec<bool> {
    let (n_cpgs, n_samples) = cov_matrix.dim();

    (0..n_cpgs)
        .into_par_iter()
        .map(|i| {
            let n_covered = (0..n_samples)
                .filter(|&j| cov_matrix[(i, j)] >= cov_thr)
                .count();
            n_covered >= min_samples
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_remove_uncovered() {
        let beta = Array2::zeros((3, 2));
        let mut cov = Array2::zeros((3, 2));

        // Set some values
        cov[(0, 0)] = 5;
        cov[(0, 1)] = 3;
        cov[(1, 0)] = 0;
        cov[(1, 1)] = 0;
        cov[(2, 0)] = 2;

        let (filtered_beta, filtered_cov, indices) = remove_uncovered(beta, cov).unwrap();

        // Should only keep rows 0 and 2
        assert_eq!(filtered_beta.nrows(), 2);
        assert_eq!(filtered_cov.nrows(), 2);
        assert_eq!(indices, vec![0, 2]);
    }
}
