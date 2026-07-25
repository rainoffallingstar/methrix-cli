use anyhow::{bail, Context, Result};
use hdf5::types::VarLenUnicode;
use hdf5::{Dataset, File, H5Type};
use ndarray::s;
use std::collections::HashSet;
use std::path::Path;

use super::se_compat::{LOADER_COMPATIBILITY, SCHEMA_NAME, SCHEMA_VERSION};

const VALIDATION_CHUNK_ELEMENTS: usize = 262_144;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomHdf5Summary {
    pub sample_count: usize,
    pub cpg_count: usize,
    pub genome: String,
    pub schema_name: String,
    pub schema_version: String,
}

/// Validate the complete `methx.custom-hdf5/1.0.0` contract without
/// loading either assay into memory in full.
pub fn validate_custom_hdf5(path: impl AsRef<Path>) -> Result<CustomHdf5Summary> {
    let path = path.as_ref();
    let file = File::open(path)
        .with_context(|| format!("Failed to open custom HDF5 file {}", path.display()))?;

    let beta = required_dataset(&file, "beta")?;
    let coverage = required_dataset(&file, "cov")?;
    validate_assay_dataset::<f32>(&beta, "beta")?;
    validate_assay_dataset::<u32>(&coverage, "cov")?;

    let assay_shape = beta.shape();
    if coverage.shape() != assay_shape {
        bail!(
            "Assay shape mismatch: beta is {:?}, cov is {:?}",
            assay_shape,
            coverage.shape()
        );
    }
    let sample_count = assay_shape[0];
    let cpg_count = assay_shape[1];
    if sample_count == 0 || cpg_count == 0 {
        bail!(
            "Assays must contain at least one sample and one CpG, found {:?}",
            assay_shape
        );
    }

    let row_data = file
        .group("rowData")
        .context("Missing required /rowData group")?;
    let chr = required_dataset(&row_data, "chr")?;
    let seqnames = required_dataset(&row_data, "seqnames")?;
    let start = required_dataset(&row_data, "start")?;
    let end = required_dataset(&row_data, "end")?;
    let width = required_dataset(&row_data, "width")?;
    let strand = required_dataset(&row_data, "strand")?;
    validate_vector_dataset::<VarLenUnicode>(&chr, "rowData/chr", cpg_count)?;
    validate_vector_dataset::<VarLenUnicode>(&seqnames, "rowData/seqnames", cpg_count)?;
    validate_vector_dataset::<u32>(&start, "rowData/start", cpg_count)?;
    validate_vector_dataset::<u32>(&end, "rowData/end", cpg_count)?;
    validate_vector_dataset::<u32>(&width, "rowData/width", cpg_count)?;
    validate_vector_dataset::<VarLenUnicode>(&strand, "rowData/strand", cpg_count)?;

    let column_data = file
        .group("colData")
        .context("Missing required /colData group")?;
    let sample_id = required_dataset(&column_data, "sample_id")?;
    let sample_name = required_dataset(&column_data, "sample_name")?;
    validate_vector_dataset::<VarLenUnicode>(&sample_id, "colData/sample_id", sample_count)?;
    validate_vector_dataset::<VarLenUnicode>(&sample_name, "colData/sample_name", sample_count)?;

    let metadata = file
        .group("metadata")
        .context("Missing required /metadata group")?;
    let genome = read_required_string_scalar(&metadata, "genome")?;
    let schema_name = read_required_string_scalar(&metadata, "schema_name")?;
    let schema_version = read_required_string_scalar(&metadata, "schema_version")?;
    let loader_compatibility = read_required_string_scalar(&metadata, "loader_compatibility")?;
    if genome.trim().is_empty() {
        bail!("metadata/genome must not be empty");
    }
    if schema_name != SCHEMA_NAME {
        bail!(
            "Unsupported metadata/schema_name {:?}; expected {:?}",
            schema_name,
            SCHEMA_NAME
        );
    }
    if schema_version != SCHEMA_VERSION {
        bail!(
            "Unsupported metadata/schema_version {:?}; expected {:?}",
            schema_version,
            SCHEMA_VERSION
        );
    }
    if loader_compatibility != LOADER_COMPATIBILITY {
        bail!("metadata/loader_compatibility does not match the schema contract");
    }
    let is_h5 = required_dataset(&metadata, "is_h5")?;
    validate_vector_dataset::<bool>(&is_h5, "metadata/is_h5", 1)?;
    if !is_h5.read_raw::<bool>()?[0] {
        bail!("metadata/is_h5 must be true");
    }

    validate_row_data(&chr, &seqnames, &start, &end, &width, &strand, cpg_count)?;
    validate_column_data(&sample_id, &sample_name)?;
    validate_assay_values(&beta, &coverage, sample_count, cpg_count)?;

    Ok(CustomHdf5Summary {
        sample_count,
        cpg_count,
        genome,
        schema_name,
        schema_version,
    })
}

fn required_dataset(parent: &hdf5::Group, name: &str) -> Result<Dataset> {
    parent
        .dataset(name)
        .with_context(|| format!("Missing required dataset {}/{}", parent.name(), name))
}

fn validate_assay_dataset<T: H5Type>(dataset: &Dataset, name: &str) -> Result<()> {
    if dataset.ndim() != 2 {
        bail!("/{name} must be rank 2, found rank {}", dataset.ndim());
    }
    if !dataset.dtype()?.is::<T>() {
        bail!("/{name} has the wrong HDF5 element type");
    }
    if dataset.chunk().is_none() {
        bail!("/{name} must use chunked storage");
    }
    Ok(())
}

fn validate_vector_dataset<T: H5Type>(
    dataset: &Dataset,
    name: &str,
    expected_len: usize,
) -> Result<()> {
    if dataset.ndim() != 1 || dataset.shape() != [expected_len] {
        bail!(
            "/{name} must have shape [{expected_len}], found {:?}",
            dataset.shape()
        );
    }
    if !dataset.dtype()?.is::<T>() {
        bail!("/{name} has the wrong HDF5 element type");
    }
    Ok(())
}

fn read_required_string_scalar(parent: &hdf5::Group, name: &str) -> Result<String> {
    let dataset = required_dataset(parent, name)?;
    if dataset.ndim() != 0 {
        bail!("{}/{} must be a scalar", parent.name(), name);
    }
    if !dataset.dtype()?.is::<VarLenUnicode>() {
        bail!("{}/{} has the wrong HDF5 element type", parent.name(), name);
    }
    Ok(dataset.read_scalar::<VarLenUnicode>()?.to_string())
}

fn validate_row_data(
    chr: &Dataset,
    seqnames: &Dataset,
    start: &Dataset,
    end: &Dataset,
    width: &Dataset,
    strand: &Dataset,
    cpg_count: usize,
) -> Result<()> {
    for chunk_start in (0..cpg_count).step_by(VALIDATION_CHUNK_ELEMENTS) {
        let chunk_end = (chunk_start + VALIDATION_CHUNK_ELEMENTS).min(cpg_count);
        let chromosomes = chr.read_slice_1d::<VarLenUnicode, _>(s![chunk_start..chunk_end])?;
        let sequence_names =
            seqnames.read_slice_1d::<VarLenUnicode, _>(s![chunk_start..chunk_end])?;
        let starts = start.read_slice_1d::<u32, _>(s![chunk_start..chunk_end])?;
        let ends = end.read_slice_1d::<u32, _>(s![chunk_start..chunk_end])?;
        let widths = width.read_slice_1d::<u32, _>(s![chunk_start..chunk_end])?;
        let strands = strand.read_slice_1d::<VarLenUnicode, _>(s![chunk_start..chunk_end])?;

        for local_index in 0..chromosomes.len() {
            let row_index = chunk_start + local_index;
            let chromosome = chromosomes[local_index].as_str();
            if chromosome.is_empty() {
                bail!("rowData/chr is empty at CpG row {row_index}");
            }
            if chromosome != sequence_names[local_index].as_str() {
                bail!("rowData/chr and rowData/seqnames differ at CpG row {row_index}");
            }
            let row_start = starts[local_index];
            let row_end = ends[local_index];
            if row_start == 0 || row_end < row_start {
                bail!("Invalid 1-based CpG coordinates at row {row_index}: {row_start}-{row_end}");
            }
            let expected_width = row_end - row_start + 1;
            if widths[local_index] != expected_width {
                bail!(
                    "rowData/width is {} at row {}, expected {}",
                    widths[local_index],
                    row_index,
                    expected_width
                );
            }
            if !matches!(strands[local_index].as_str(), "+" | "-" | "*") {
                bail!(
                    "Invalid rowData/strand {:?} at CpG row {}",
                    strands[local_index].as_str(),
                    row_index
                );
            }
        }
    }
    Ok(())
}

fn validate_column_data(sample_id: &Dataset, sample_name: &Dataset) -> Result<()> {
    let sample_ids = sample_id.read_raw::<VarLenUnicode>()?;
    let sample_names = sample_name.read_raw::<VarLenUnicode>()?;
    let mut seen = HashSet::with_capacity(sample_ids.len());
    for (sample_index, (id, name)) in sample_ids.iter().zip(&sample_names).enumerate() {
        let id = id.as_str();
        if id != name.as_str() {
            bail!(
                "colData/sample_id and colData/sample_name differ at sample column {sample_index}"
            );
        }
        if id.is_empty()
            || id.trim() != id
            || id
                .chars()
                .any(|character| matches!(character, '\t' | '\n' | '\r'))
        {
            bail!(
                "Invalid sample ID {:?} at sample column {}",
                id,
                sample_index
            );
        }
        if !seen.insert(id.to_string()) {
            bail!(
                "Duplicate sample ID {:?} at sample column {}",
                id,
                sample_index
            );
        }
    }
    Ok(())
}

fn validate_assay_values(
    beta: &Dataset,
    coverage: &Dataset,
    sample_count: usize,
    cpg_count: usize,
) -> Result<()> {
    for sample_index in 0..sample_count {
        for chunk_start in (0..cpg_count).step_by(VALIDATION_CHUNK_ELEMENTS) {
            let chunk_end = (chunk_start + VALIDATION_CHUNK_ELEMENTS).min(cpg_count);
            let beta_values =
                beta.read_slice_1d::<f32, _>(s![sample_index, chunk_start..chunk_end])?;
            let coverage_values =
                coverage.read_slice_1d::<u32, _>(s![sample_index, chunk_start..chunk_end])?;
            for local_index in 0..beta_values.len() {
                let cpg_index = chunk_start + local_index;
                let beta_value = beta_values[local_index];
                let coverage_value = coverage_values[local_index];
                if beta_value.is_infinite()
                    || (beta_value.is_finite() && !(0.0..=1.0).contains(&beta_value))
                {
                    bail!(
                        "Invalid beta value {} at sample {}, CpG {}",
                        beta_value,
                        sample_index,
                        cpg_index
                    );
                }
                if (coverage_value == 0) != beta_value.is_nan() {
                    bail!(
                        "beta/cov missingness mismatch at sample {}, CpG {}: beta={}, cov={}",
                        sample_index,
                        cpg_index,
                        beta_value,
                        coverage_value
                    );
                }
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::validate_custom_hdf5;
    use crate::cli::process::MethrixData;
    use crate::genome::cpg::CpGSite;
    use crate::hdf5::se_compat::CustomHdf5Writer;
    use hdf5::types::VarLenUnicode;
    use ndarray::Array2;
    use std::str::FromStr;
    use tempfile::tempdir;

    fn write_valid_fixture(path: &std::path::Path) {
        let data = MethrixData {
            beta_matrix: Array2::from_shape_vec((2, 2), vec![0.25, f32::NAN, 0.75, 1.0]).unwrap(),
            cov_matrix: Array2::from_shape_vec((2, 2), vec![4, 0, 8, 2]).unwrap(),
            cpg_locations: vec![
                CpGSite {
                    chr: "chr1".to_string(),
                    start: 9,
                    end: 11,
                    strand: '+',
                },
                CpGSite {
                    chr: "chr1".to_string(),
                    start: 19,
                    end: 21,
                    strand: '+',
                },
            ],
            sample_names: vec!["sample_a".to_string(), "sample_b".to_string()],
            genome: "mini".to_string(),
        };
        CustomHdf5Writer::write_methrix_object_to_path(path, &data).unwrap();
    }

    #[test]
    fn validates_complete_native_schema() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("valid.h5");
        write_valid_fixture(&path);

        let summary = validate_custom_hdf5(&path).unwrap();
        assert_eq!(summary.sample_count, 2);
        assert_eq!(summary.cpg_count, 2);
        assert_eq!(summary.genome, "mini");
    }

    #[test]
    fn rejects_missing_required_dataset() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("missing.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        file.unlink("cov").unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("Missing required dataset"));
    }

    #[test]
    fn rejects_wrong_schema_version() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("wrong-version.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        let version = VarLenUnicode::from_str("9.9.9").unwrap();
        file.group("metadata")
            .unwrap()
            .dataset("schema_version")
            .unwrap()
            .write_scalar(&version)
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error
            .to_string()
            .contains("Unsupported metadata/schema_version"));
    }

    #[test]
    fn rejects_non_chunked_assay_storage() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("contiguous.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        file.unlink("beta").unwrap();
        file.new_dataset_builder()
            .with_data(
                Array2::from_shape_vec((2, 2), vec![0.25, 0.75, f32::NAN, 1.0])
                    .unwrap()
                    .view(),
            )
            .no_chunk()
            .create("beta")
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("must use chunked storage"));
    }

    #[test]
    fn rejects_wrong_assay_element_type() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("wrong-type.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        file.unlink("beta").unwrap();
        file.new_dataset_builder()
            .with_data(
                Array2::from_shape_vec((2, 2), vec![1u32, 2, 3, 4])
                    .unwrap()
                    .view(),
            )
            .chunk((1, 2))
            .create("beta")
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("wrong HDF5 element type"));
    }

    #[test]
    fn rejects_wrong_assay_rank() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("wrong-rank.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        file.unlink("beta").unwrap();
        file.new_dataset_builder()
            .with_data(&[0.25f32, f32::NAN, 0.75, 1.0])
            .chunk((2,))
            .create("beta")
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("/beta must be rank 2"));
    }

    #[test]
    fn rejects_row_metadata_length_mismatch() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("short-row-data.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        let row_data = file.group("rowData").unwrap();
        row_data.unlink("start").unwrap();
        row_data
            .new_dataset_builder()
            .with_data(&[10u32])
            .create("start")
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error
            .to_string()
            .contains("/rowData/start must have shape [2]"));
    }

    #[test]
    fn rejects_invalid_row_width() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("wrong-width.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        file.group("rowData")
            .unwrap()
            .dataset("width")
            .unwrap()
            .write_raw(&[1u32, 2])
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("rowData/width"));
    }

    #[test]
    fn rejects_invalid_strand() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("invalid-strand.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        let strands = [
            VarLenUnicode::from_str("+").unwrap(),
            VarLenUnicode::from_str("?").unwrap(),
        ];
        file.group("rowData")
            .unwrap()
            .dataset("strand")
            .unwrap()
            .write_raw(&strands)
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("Invalid rowData/strand"));
    }

    #[test]
    fn rejects_duplicate_sample_ids() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("duplicate-samples.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        let duplicate_ids = [
            VarLenUnicode::from_str("duplicate").unwrap(),
            VarLenUnicode::from_str("duplicate").unwrap(),
        ];
        let column_data = file.group("colData").unwrap();
        column_data
            .dataset("sample_id")
            .unwrap()
            .write_raw(&duplicate_ids)
            .unwrap();
        column_data
            .dataset("sample_name")
            .unwrap()
            .write_raw(&duplicate_ids)
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("Duplicate sample ID"));
    }

    #[test]
    fn rejects_invalid_finite_beta_value() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("invalid-beta.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        file.dataset("beta")
            .unwrap()
            .write_raw(&[1.25f32, f32::NAN, 0.75, 1.0])
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("Invalid beta value"));
    }

    #[test]
    fn rejects_beta_coverage_missingness_mismatch() {
        let directory = tempdir().unwrap();
        let path = directory.path().join("contradictory.h5");
        write_valid_fixture(&path);
        let file = hdf5::File::open_rw(&path).unwrap();
        file.dataset("cov")
            .unwrap()
            .write_raw(&[0u32, 8, 0, 2])
            .unwrap();
        file.flush().unwrap();
        drop(file);

        let error = validate_custom_hdf5(&path).unwrap_err();
        assert!(error.to_string().contains("beta/cov missingness mismatch"));
    }
}
