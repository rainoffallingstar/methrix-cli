use methx::hdf5::validate_custom_hdf5;
use std::fs;
use std::process::Command;
use tempfile::tempdir;

#[test]
fn process_command_runs_minimal_pipeline_from_fasta() {
    let temporary_directory = tempdir().unwrap();
    let input_directory = temporary_directory.path().join("input");
    let output_directory = temporary_directory.path().join("output");
    fs::create_dir_all(&input_directory).unwrap();

    let genome_path = temporary_directory.path().join("mini.fa");
    fs::write(&genome_path, ">chr1\nAACGTTTCGAA\n").unwrap();
    fs::write(
        input_directory.join("sample.cov"),
        concat!(
            "chr1\t3\t3\t75.000000\t3\t1\n",
            "chr1\t8\t8\t25.000000\t1\t3\n"
        ),
    )
    .unwrap();

    let command_output = Command::new(env!("CARGO_BIN_EXE_methx"))
        .arg("process")
        .arg("--input")
        .arg(&input_directory)
        .arg("--output")
        .arg(&output_directory)
        .arg("--genome")
        .arg(&genome_path)
        .arg("--threads")
        .arg("2")
        .arg("--min-coverage")
        .arg("1")
        .arg("--skip-annotation")
        .output()
        .unwrap();

    assert!(
        command_output.status.success(),
        "methx process failed\nstdout:\n{}\nstderr:\n{}",
        String::from_utf8_lossy(&command_output.stdout),
        String::from_utf8_lossy(&command_output.stderr)
    );

    let assays_path = output_directory.join("assays.h5");
    let alias_path = output_directory.join("methrix_data.h5");
    let summary = validate_custom_hdf5(&assays_path).unwrap();
    assert_eq!(summary.sample_count, 1);
    assert_eq!(summary.cpg_count, 2);
    assert_eq!(summary.genome, "mini");
    assert_eq!(
        fs::read(&assays_path).unwrap(),
        fs::read(alias_path).unwrap()
    );
    assert!(output_directory.join("CpG_coverage.xlsx").is_file());
    assert!(!output_directory.join("CpG_annotation_report.xlsx").exists());
    assert!(!output_directory
        .join("CpG_annotation_details.tsv.gz")
        .exists());

    let file = hdf5::File::open(assays_path).unwrap();
    assert_eq!(file.dataset("cov").unwrap().shape(), vec![1, 2]);
    assert_eq!(
        file.dataset("cov").unwrap().read_raw::<u32>().unwrap(),
        vec![4, 4]
    );
    assert_eq!(
        file.group("rowData")
            .unwrap()
            .dataset("start")
            .unwrap()
            .read_raw::<u32>()
            .unwrap(),
        vec![3, 8]
    );
}
