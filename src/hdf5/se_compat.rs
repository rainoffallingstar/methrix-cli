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

        // Write the 2D array - it's contiguous in C layout
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
        let start: Vec<u32> = cpg_locations.iter().map(|cpg| cpg.start).collect();
        let strand: Vec<VarLenAscii> = cpg_locations
            .iter()
            .map(|cpg| VarLenAscii::from_ascii(&cpg.strand.to_string()).unwrap())
            .collect();
        let end: Vec<u32> = start.iter().map(|s| s + 2).collect(); // CpG length is 2

        group
            .new_dataset_builder()
            .with_data(&chr)
            .create("chr")
            .context("Failed to create chr dataset")?;

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
        attr.write_scalar(&1)?;

        let delayed_array_str = unsafe { VarLenUnicode::from_str_unchecked("HDF5Array") };
        let attr = file
            .new_attr::<VarLenUnicode>()
            .create("delayed_array_type")
            .context("Failed to create delayed_array_type attribute")?;
        attr.write_scalar(&delayed_array_str)?;

        Ok(())
    }
}
