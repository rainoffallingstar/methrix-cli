use anyhow::{Context, Result};
use rust_xlsxwriter::Workbook;

pub struct QCReportGenerator;

pub fn generate_coverage_report(
    output_path: &str,
    sample_stats: &[crate::processing::stats::SampleStats],
) -> Result<()> {
    let mut workbook = Workbook::new();

    let _worksheet = workbook.add_worksheet();

    // Get the worksheet we just added
    let worksheet = workbook
        .worksheet_from_index(0)
        .context("Failed to get worksheet")?;

    // Write headers
    worksheet.write_string(0, 0, "Sample")?;
    worksheet.write_string(0, 1, "Total CpGs")?;
    worksheet.write_string(0, 2, "Covered CpGs")?;
    worksheet.write_string(0, 3, "1X")?;
    worksheet.write_string(0, 4, "2X")?;
    worksheet.write_string(0, 5, "3X")?;
    worksheet.write_string(0, 6, "4X")?;
    worksheet.write_string(0, 7, "5X")?;
    worksheet.write_string(0, 8, "10X")?;

    // Write data
    for (row, stats) in sample_stats.iter().enumerate() {
        let row = (row + 1) as u32;

        worksheet.write_string(row, 0, &stats.sample_name)?;
        worksheet.write_number(row, 1, stats.n_total as f64)?;
        worksheet.write_number(row, 2, stats.n_covered as f64)?;

        for (col, (_, count)) in stats.coverage_distribution.iter().enumerate() {
            worksheet.write_number(row, (3 + col as u32) as u16, *count as f64)?;
        }
    }

    workbook.save(output_path)?;

    println!("QC report saved to: {}", output_path);
    Ok(())
}

/// Generate QC report from existing H5 file
pub fn generate_qc_report(input_dir: &str, output_path: &str) -> Result<()> {
    use std::path::Path;

    // Load H5 file (new default + legacy fallback)
    let assays_h5 = Path::new(input_dir).join("assays.h5");
    let legacy_h5 = Path::new(input_dir).join("methrix_data.h5");
    let h5_path = if assays_h5.exists() {
        assays_h5
    } else if legacy_h5.exists() {
        legacy_h5
    } else {
        anyhow::bail!(
            "H5 file not found: expected {} or {}",
            Path::new(input_dir).join("assays.h5").display(),
            Path::new(input_dir).join("methrix_data.h5").display()
        );
    };

    let file = hdf5::File::open(&h5_path).context("Failed to open H5 file")?;

    // Read coverage matrix (prefer root /cov; fallback /assays/cov for old layouts)
    let cov_dataset = match file.dataset("cov") {
        Ok(ds) => ds,
        Err(_) => {
            let assays_group = file
                .group("assays")
                .context("Failed to open assays group for cov fallback")?;
            assays_group
                .dataset("cov")
                .context("Failed to open cov dataset from /assays/cov fallback")?
        }
    };

    // Get sample names from colData
    let coldata_group = file
        .group("colData")
        .context("Failed to open colData group")?;

    let sample_dataset = coldata_group
        .dataset("sample_id")
        .context("Failed to open sample_id dataset")?;
    let sample_ids: Vec<hdf5::types::VarLenAscii> = sample_dataset
        .read_raw()
        .context("Failed to read sample IDs")?;

    let sample_names: Vec<String> = sample_ids
        .iter()
        .map(|s: &hdf5::types::VarLenAscii| s.to_string())
        .collect();

    let cov_matrix: Vec<u16> = cov_dataset
        .read_raw()
        .context("Failed to read coverage data")?;

    // Get dimensions from the dataset
    let cov_space = cov_dataset.space().context("Failed to get dataspace")?;
    let shape = cov_space.shape();

    // Convert coverage data into sample-major vectors, robust to both:
    // 1) [n_samples, n_cpgs] (current writer layout)
    // 2) [n_cpgs, n_samples] (legacy/alternative layout)
    let matrix_2d = reshape_cov_by_sample(&cov_matrix, &shape, sample_names.len())?;

    // Calculate stats
    let stats =
        crate::processing::stats::calculate_coverage_stats_from_vec(&matrix_2d, &sample_names);

    // Generate report
    generate_coverage_report(output_path, &stats)
}

fn reshape_cov_by_sample(raw: &[u16], shape: &[usize], n_samples: usize) -> Result<Vec<Vec<u16>>> {
    if shape.len() != 2 {
        anyhow::bail!("Expected 2D coverage matrix, got shape {:?}", shape);
    }

    let dim0 = shape[0];
    let dim1 = shape[1];
    let expected_len = dim0.checked_mul(dim1).context("Coverage shape overflow")?;
    if raw.len() != expected_len {
        anyhow::bail!(
            "Coverage raw length mismatch: got {}, expected {} (shape {:?})",
            raw.len(),
            expected_len,
            shape
        );
    }

    if dim0 == n_samples {
        // Stored as [samples, cpgs] in row-major.
        let n_cpgs = dim1;
        let mut out = Vec::with_capacity(n_samples);
        for sample_idx in 0..n_samples {
            let start = sample_idx * n_cpgs;
            out.push(raw[start..start + n_cpgs].to_vec());
        }
        return Ok(out);
    }

    if dim1 == n_samples {
        // Stored as [cpgs, samples] in row-major.
        let n_cpgs = dim0;
        let mut out = vec![vec![0u16; n_cpgs]; n_samples];
        for cpg_idx in 0..n_cpgs {
            let row_start = cpg_idx * n_samples;
            for sample_idx in 0..n_samples {
                out[sample_idx][cpg_idx] = raw[row_start + sample_idx];
            }
        }
        return Ok(out);
    }

    anyhow::bail!(
        "Coverage shape {:?} does not match sample count {}",
        shape,
        n_samples
    )
}

#[cfg(test)]
mod tests {
    use super::reshape_cov_by_sample;

    #[test]
    fn reshape_samples_by_cpgs_layout() {
        // [samples=2, cpgs=3]
        let raw = vec![1u16, 0, 5, 2, 3, 0];
        let shape = vec![2usize, 3usize];
        let out = reshape_cov_by_sample(&raw, &shape, 2).unwrap();
        assert_eq!(out, vec![vec![1, 0, 5], vec![2, 3, 0]]);
    }

    #[test]
    fn reshape_cpgs_by_samples_layout() {
        // [cpgs=3, samples=2]
        let raw = vec![1u16, 2, 0, 3, 5, 0];
        let shape = vec![3usize, 2usize];
        let out = reshape_cov_by_sample(&raw, &shape, 2).unwrap();
        assert_eq!(out, vec![vec![1, 0, 5], vec![2, 3, 0]]);
    }
}
