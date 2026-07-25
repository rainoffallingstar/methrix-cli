use anyhow::{Context, Result};
use hdf5::types::VarLenUnicode;
use hdf5::{File, Group};
use ndarray::{s, Array2};
use std::path::{Path, PathBuf};
use std::str::FromStr;

use crate::atomic_output::write_atomically;

pub const SCHEMA_NAME: &str = "methx.custom-hdf5";
pub const SCHEMA_VERSION: &str = "1.0.0";
pub const LOADER_COMPATIBILITY: &str = "rhdf5 direct schema access only; not compatible with HDF5Array::loadHDF5SummarizedExperiment or methrix::load_HDF5_methrix";
const TARGET_ASSAY_CHUNK_BYTES: usize = 1024 * 1024;

pub struct CustomHdf5Writer {
    output_path: PathBuf,
}

impl CustomHdf5Writer {
    pub fn new(output_path: String) -> Self {
        Self {
            output_path: PathBuf::from(output_path),
        }
    }

    /// Write the versioned methx custom HDF5 schema atomically.
    pub fn write_methrix_object(
        &self,
        methrix_data: &crate::cli::process::MethrixData,
    ) -> Result<()> {
        write_atomically(&self.output_path, |temporary_path| {
            Self::write_methrix_object_to_path(temporary_path, methrix_data)?;
            crate::hdf5::validate::validate_custom_hdf5(temporary_path)?;
            Ok(())
        })
    }

    pub(crate) fn write_methrix_object_to_path(
        output_path: &Path,
        methrix_data: &crate::cli::process::MethrixData,
    ) -> Result<()> {
        let file = File::create(output_path).context("Failed to create HDF5 file")?;

        Self::write_assay(&file, "beta", &methrix_data.beta_matrix)?;
        Self::write_assay(&file, "cov", &methrix_data.cov_matrix)?;

        let rowdata_group = file
            .create_group("rowData")
            .context("Failed to create rowData group")?;
        Self::write_rowdata(&rowdata_group, &methrix_data.cpg_locations)?;

        let coldata_group = file
            .create_group("colData")
            .context("Failed to create colData group")?;
        Self::write_coldata(&coldata_group, &methrix_data.sample_names)?;

        let metadata_group = file
            .create_group("metadata")
            .context("Failed to create metadata group")?;
        Self::write_metadata(&metadata_group, &methrix_data.genome)?;

        file.flush().context("Failed to flush HDF5 file")?;
        Ok(())
    }

    fn write_assay<T: hdf5::H5Type + Copy>(
        group: &Group,
        name: &str,
        data: &Array2<T>,
    ) -> Result<()> {
        let (n_cpgs, n_samples) = data.dim();
        if n_cpgs == 0 || n_samples == 0 {
            anyhow::bail!(
                "Cannot write assay {} with zero-sized dimensions {:?}",
                name,
                data.dim()
            );
        }

        // R/HDF5 uses column-major matrices. Store [sample, CpG] in HDF5 C
        // layout so direct R readers observe [CpG, sample]. Source columns are
        // strided, therefore copy only one bounded standard-layout chunk at a
        // time instead of materializing a complete transposed assay.
        let elements_per_chunk = (TARGET_ASSAY_CHUNK_BYTES / std::mem::size_of::<T>()).max(1);
        let cpgs_per_chunk = elements_per_chunk.min(n_cpgs);
        let dataset = group
            .new_dataset::<T>()
            .shape((n_samples, n_cpgs))
            .chunk((1, cpgs_per_chunk))
            .deflate(6)
            .create(name)
            .context("Failed to create dataset")?;

        for sample_idx in 0..n_samples {
            for cpg_start in (0..n_cpgs).step_by(cpgs_per_chunk) {
                let cpg_end = (cpg_start + cpgs_per_chunk).min(n_cpgs);
                let chunk_values = (cpg_start..cpg_end)
                    .map(|cpg_idx| data[(cpg_idx, sample_idx)])
                    .collect::<Vec<_>>();
                let chunk = Array2::from_shape_vec((1, cpg_end - cpg_start), chunk_values)
                    .context("Failed to shape bounded assay chunk")?;
                dataset
                    .write_slice(
                        chunk.view(),
                        s![sample_idx..sample_idx + 1, cpg_start..cpg_end],
                    )
                    .with_context(|| {
                        format!(
                            "Failed to write {} assay chunk for sample {} and CpGs {}..{}",
                            name, sample_idx, cpg_start, cpg_end
                        )
                    })?;
            }
        }

        Ok(())
    }

    fn write_rowdata(group: &Group, cpg_locations: &[crate::genome::cpg::CpGSite]) -> Result<()> {
        let sequence_names = cpg_locations
            .iter()
            .enumerate()
            .map(|(row_index, cpg)| {
                VarLenUnicode::from_str(&cpg.chr).with_context(|| {
                    format!(
                        "Invalid chromosome name at row {}: {:?}",
                        row_index, cpg.chr
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let start: Vec<u32> = cpg_locations
            .iter()
            .enumerate()
            .map(|(row_index, cpg)| {
                cpg.start.checked_add(1).with_context(|| {
                    format!(
                        "CpG start overflows 1-based coordinates at row {}",
                        row_index
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;
        let end: Vec<u32> = cpg_locations.iter().map(|cpg| cpg.end).collect();
        let strands = cpg_locations
            .iter()
            .enumerate()
            .map(|(row_index, cpg)| {
                VarLenUnicode::from_str(&cpg.strand.to_string()).with_context(|| {
                    format!("Invalid strand at row {}: {:?}", row_index, cpg.strand)
                })
            })
            .collect::<Result<Vec<_>>>()?;

        group
            .new_dataset_builder()
            .with_data(&sequence_names)
            .create("chr")
            .context("Failed to create chr dataset")?;

        group
            .new_dataset_builder()
            .with_data(&sequence_names)
            .create("seqnames")
            .context("Failed to create seqnames dataset")?;

        group
            .new_dataset_builder()
            .with_data(&start)
            .create("start")
            .context("Failed to create start dataset")?;

        group
            .new_dataset_builder()
            .with_data(&end)
            .create("end")
            .context("Failed to create end dataset")?;

        let width: Vec<u32> = start
            .iter()
            .zip(end.iter())
            .enumerate()
            .map(|(row_index, (start_position, end_position))| {
                end_position
                    .checked_sub(*start_position)
                    .and_then(|difference| difference.checked_add(1))
                    .with_context(|| format!("Invalid CpG coordinates at row {}", row_index))
            })
            .collect::<Result<Vec<_>>>()?;
        group
            .new_dataset_builder()
            .with_data(&width)
            .create("width")
            .context("Failed to create width dataset")?;

        group
            .new_dataset_builder()
            .with_data(&strands)
            .create("strand")
            .context("Failed to create strand dataset")?;

        Ok(())
    }

    fn write_coldata(group: &Group, sample_names: &[String]) -> Result<()> {
        let names = sample_names
            .iter()
            .enumerate()
            .map(|(sample_index, sample_name)| {
                VarLenUnicode::from_str(sample_name).with_context(|| {
                    format!(
                        "Invalid sample name at column {}: {:?}",
                        sample_index, sample_name
                    )
                })
            })
            .collect::<Result<Vec<_>>>()?;

        group
            .new_dataset_builder()
            .with_data(&names)
            .create("sample_id")
            .context("Failed to create sample_id dataset")?;

        group
            .new_dataset_builder()
            .with_data(&names)
            .create("sample_name")
            .context("Failed to create sample_name dataset")?;

        Ok(())
    }

    fn write_metadata(group: &Group, genome: &str) -> Result<()> {
        Self::write_string_metadata(group, "genome", genome)?;
        Self::write_string_metadata(group, "schema_name", SCHEMA_NAME)?;
        Self::write_string_metadata(group, "schema_version", SCHEMA_VERSION)?;
        Self::write_string_metadata(group, "loader_compatibility", LOADER_COMPATIBILITY)?;

        group
            .new_dataset_builder()
            .with_data(&[true])
            .create("is_h5")
            .context("Failed to create is_h5 dataset")?;

        Ok(())
    }

    fn write_string_metadata(group: &Group, name: &str, value: &str) -> Result<()> {
        let encoded_value = VarLenUnicode::from_str(value)
            .with_context(|| format!("Metadata field {} contains an invalid string", name))?;
        let dataset = group
            .new_dataset::<VarLenUnicode>()
            .shape(())
            .create(name)
            .with_context(|| format!("Failed to create {} metadata dataset", name))?;
        dataset
            .write_scalar(&encoded_value)
            .with_context(|| format!("Failed to write {} metadata dataset", name))?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::{CustomHdf5Writer, LOADER_COMPATIBILITY, SCHEMA_NAME, SCHEMA_VERSION};
    use crate::cli::process::MethrixData;
    use crate::genome::cpg::CpGSite;
    use hdf5::types::VarLenUnicode;
    use ndarray::Array2;
    use tempfile::tempdir;

    #[test]
    fn writes_versioned_custom_schema_with_u32_coverage_and_r_coordinates() {
        let temporary_directory = tempdir().unwrap();
        let output_path = temporary_directory.path().join("assays.h5");
        let methrix_data = MethrixData {
            beta_matrix: Array2::from_shape_vec((2, 2), vec![0.25, 0.5, 0.75, 1.0]).unwrap(),
            cov_matrix: Array2::from_shape_vec((2, 2), vec![70_000, 2, 3, 4]).unwrap(),
            cpg_locations: vec![
                CpGSite {
                    chr: "chr一".to_string(),
                    start: 9,
                    end: 11,
                    strand: '+',
                },
                CpGSite {
                    chr: "chrÉ".to_string(),
                    start: 19,
                    end: 21,
                    strand: '+',
                },
            ],
            sample_names: vec!["样本甲".to_string(), "échantillon_b".to_string()],
            genome: "人类-hg38".to_string(),
        };

        CustomHdf5Writer::new(output_path.to_string_lossy().into_owned())
            .write_methrix_object(&methrix_data)
            .unwrap();

        let file = hdf5::File::open(&output_path).unwrap();
        let metadata = file.group("metadata").unwrap();
        assert_eq!(
            metadata
                .dataset("schema_name")
                .unwrap()
                .read_scalar::<VarLenUnicode>()
                .unwrap()
                .as_str(),
            SCHEMA_NAME
        );
        assert_eq!(
            metadata
                .dataset("schema_version")
                .unwrap()
                .read_scalar::<VarLenUnicode>()
                .unwrap()
                .as_str(),
            SCHEMA_VERSION
        );
        assert_eq!(
            metadata
                .dataset("loader_compatibility")
                .unwrap()
                .read_scalar::<VarLenUnicode>()
                .unwrap()
                .as_str(),
            LOADER_COMPATIBILITY
        );
        assert!(file.attr("se_version").is_err());
        assert!(file.attr("delayed_array_type").is_err());

        let coverage_dataset = file.dataset("cov").unwrap();
        assert_eq!(coverage_dataset.shape(), vec![2, 2]);
        assert_eq!(coverage_dataset.chunk(), Some(vec![1, 2]));
        assert_eq!(file.dataset("beta").unwrap().chunk(), Some(vec![1, 2]));
        assert_eq!(
            coverage_dataset.read_raw::<u32>().unwrap(),
            vec![70_000, 3, 2, 4]
        );

        let row_data = file.group("rowData").unwrap();
        assert_eq!(
            row_data
                .dataset("start")
                .unwrap()
                .read_raw::<u32>()
                .unwrap(),
            vec![10, 20]
        );
        assert_eq!(
            row_data.dataset("end").unwrap().read_raw::<u32>().unwrap(),
            vec![11, 21]
        );
        assert_eq!(
            row_data
                .dataset("width")
                .unwrap()
                .read_raw::<u32>()
                .unwrap(),
            vec![2, 2]
        );
        assert_eq!(
            row_data
                .dataset("seqnames")
                .unwrap()
                .read_raw::<VarLenUnicode>()
                .unwrap()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["chr一", "chrÉ"]
        );

        let column_data = file.group("colData").unwrap();
        assert_eq!(
            column_data
                .dataset("sample_name")
                .unwrap()
                .read_raw::<VarLenUnicode>()
                .unwrap()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["样本甲", "échantillon_b"]
        );
    }
}
