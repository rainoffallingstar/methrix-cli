use anyhow::{Context, Result};
use hdf5::types::{VarLenAscii, VarLenUnicode};
use hdf5::{File, Group};
use ndarray::Array2;

pub struct SummarizedExperimentWriter {
    output_path: String,
}

impl SummarizedExperimentWriter {
    pub fn new(output_path: String) -> Self {
        Self { output_path }
    }

    /// Write methrix object as H5 format - compatible with R::load_HDF5_summarized_experiment
    pub fn write_methrix_object(
        &self,
        methrix_data: &crate::cli::process::MethrixData,
    ) -> Result<()> {
        let file = File::create(&self.output_path).context("Failed to create HDF5 file")?;
        self.write_se_attributes(&file)?;

        // 1. Write assays (required for both direct reading and se.rds)
        self.write_assay(&file, "beta", &methrix_data.beta_matrix)?;
        self.write_assay(&file, "cov", &methrix_data.cov_matrix)?;

        // 2. Write rowData (for se.rds creation support)
        let rowdata_group = file
            .create_group("rowData")
            .context("Failed to create rowData group")?;
        self.write_rowdata(&rowdata_group, &methrix_data.cpg_locations)?;

        // 3. Write colData (for se.rds creation support)
        let coldata_group = file
            .create_group("colData")
            .context("Failed to create colData group")?;
        self.write_coldata(&coldata_group, &methrix_data.sample_names)?;

        // 4. Write metadata (for se.rds creation support)
        let metadata_group = file
            .create_group("metadata")
            .context("Failed to create metadata group")?;
        self.write_metadata(&metadata_group, &methrix_data.genome)?;

        Ok(())
    }

    fn write_assay<T: hdf5::H5Type + Copy>(
        &self,
        group: &Group,
        name: &str,
        data: &Array2<T>,
    ) -> Result<()> {
        let (n_cpgs, n_samples) = data.dim(); // data is [n_cpgs, n_samples] in row-major

        // R/HDF5 uses column-major: matrix[cpg, sample]
        // We'll create HDF5 dataset with shape [n_samples, n_cpgs] in C layout
        // This will appear in R as [n_cpgs, n_samples] after automatic transpose
        //
        // Data order in Vec: [cpg1_s1, cpg2_s1, ..., cpgN_s1, cpg1_s2, cpg2_s2, ...]
        // Shape [n_samples, n_cpgs] C-layout stores as:
        //   row 0: [cpg1_s1, cpg2_s1, ..., cpgN_s1]
        //   row 1: [cpg1_s2, cpg2_s2, ..., cpgN_s2]
        // Which is exactly our Vec order!
        let mut col_major_data: Vec<T> = Vec::with_capacity(n_cpgs * n_samples);
        for sample_idx in 0..n_samples {
            for cpg_idx in 0..n_cpgs {
                col_major_data.push(data[(cpg_idx, sample_idx)]);
            }
        }

        // Create C-layout 2D array with shape [n_samples, n_cpgs]
        use ndarray::Array2;
        let reshaped = Array2::from_shape_vec((n_samples, n_cpgs), col_major_data)
            .context("Failed to reshape assay data")?;

        // Write the 2D array - it's contiguous in C layout.
        // Use .view() to get ArrayView2<T> which pins D=Ix2 for type inference.
        let builder = group.new_dataset_builder();
        let _dataset = builder
            .with_data(reshaped.view())
            .deflate(6)
            .create(name)
            .context("Failed to create dataset")?;

        Ok(())
    }

    fn write_rowdata(
        &self,
        group: &Group,
        cpg_locations: &[crate::genome::cpg::CpGSite],
    ) -> Result<()> {
        let chr: Vec<VarLenAscii> = cpg_locations
            .iter()
            .map(|cpg| VarLenAscii::from_ascii(&cpg.chr).unwrap())
            .collect();
        let start: Vec<u32> = cpg_locations.iter().map(|cpg| cpg.start + 1).collect();
        let end: Vec<u32> = cpg_locations.iter().map(|cpg| cpg.end).collect();
        let strand: Vec<VarLenAscii> = cpg_locations
            .iter()
            .map(|cpg| VarLenAscii::from_ascii(&cpg.strand.to_string()).unwrap())
            .collect();

        group
            .new_dataset_builder()
            .with_data(&chr)
            .create("chr")
            .context("Failed to create chr dataset")?;

        group
            .new_dataset_builder()
            .with_data(&chr)
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
            .map(|(start_position, end_position)| end_position - start_position + 1)
            .collect();
        group
            .new_dataset_builder()
            .with_data(&width)
            .create("width")
            .context("Failed to create width dataset")?;

        group
            .new_dataset_builder()
            .with_data(&strand)
            .create("strand")
            .context("Failed to create strand dataset")?;

        Ok(())
    }

    fn write_coldata(&self, group: &Group, sample_names: &[String]) -> Result<()> {
        // Convert to VarLenAscii
        let names: Vec<VarLenAscii> = sample_names
            .iter()
            .map(|s| VarLenAscii::from_ascii(s).unwrap())
            .collect();

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

    fn write_metadata(&self, group: &Group, genome: &str) -> Result<()> {
        // genome - use VarLenUnicode for proper string representation in R
        let genome_str = unsafe { VarLenUnicode::from_str_unchecked(genome) };
        group
            .new_dataset_builder()
            .with_data(&genome_str)
            .create("genome")
            .context("Failed to create genome dataset")?;

        // is_h5
        group
            .new_dataset_builder()
            .with_data(&[true])
            .create("is_h5")
            .context("Failed to create is_h5 dataset")?;

        Ok(())
    }

    fn write_se_attributes(&self, file: &File) -> Result<()> {
        // HDF5SummarizedExperiment required attributes
        use hdf5::types::VarLenUnicode;

        let attr = file
            .new_attr::<u32>()
            .create("se_version")
            .context("Failed to create se_version attribute")?;
        attr.write_scalar(&2)?;

        let delayed_array_str = unsafe { VarLenUnicode::from_str_unchecked("HDF5Array") };
        let attr = file
            .new_attr::<VarLenUnicode>()
            .create("delayed_array_type")
            .context("Failed to create delayed_array_type attribute")?;
        attr.write_scalar(&delayed_array_str)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::SummarizedExperimentWriter;
    use crate::cli::process::MethrixData;
    use crate::genome::cpg::CpGSite;
    use hdf5::types::VarLenAscii;
    use ndarray::Array2;
    use tempfile::tempdir;

    #[test]
    fn writes_schema_v2_with_u32_coverage_and_r_coordinates() {
        let temporary_directory = tempdir().unwrap();
        let output_path = temporary_directory.path().join("assays.h5");
        let methrix_data = MethrixData {
            beta_matrix: Array2::from_shape_vec((2, 2), vec![0.25, 0.5, 0.75, 1.0]).unwrap(),
            cov_matrix: Array2::from_shape_vec((2, 2), vec![70_000, 2, 3, 4]).unwrap(),
            cpg_locations: vec![
                CpGSite {
                    chr: "chr1".to_string(),
                    start: 9,
                    end: 11,
                    strand: '+',
                },
                CpGSite {
                    chr: "chr2".to_string(),
                    start: 19,
                    end: 21,
                    strand: '+',
                },
            ],
            sample_names: vec!["sample_a".to_string(), "sample_b".to_string()],
            genome: "hg38".to_string(),
        };

        SummarizedExperimentWriter::new(output_path.to_string_lossy().into_owned())
            .write_methrix_object(&methrix_data)
            .unwrap();

        let file = hdf5::File::open(&output_path).unwrap();
        assert_eq!(
            file.attr("se_version")
                .unwrap()
                .read_scalar::<u32>()
                .unwrap(),
            2
        );

        let coverage_dataset = file.dataset("cov").unwrap();
        assert_eq!(coverage_dataset.shape(), vec![2, 2]);
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
                .read_raw::<VarLenAscii>()
                .unwrap()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["chr1", "chr2"]
        );

        let column_data = file.group("colData").unwrap();
        assert_eq!(
            column_data
                .dataset("sample_name")
                .unwrap()
                .read_raw::<VarLenAscii>()
                .unwrap()
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            vec!["sample_a", "sample_b"]
        );
    }
}
