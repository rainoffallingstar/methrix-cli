#!/usr/bin/env Rscript

assert_true <- function(condition, message) {
  if (!isTRUE(condition)) {
    stop(message, call. = FALSE)
  }
}

require_conversion_packages <- function() {
  required_packages <- c(
    "HDF5Array",
    "methrix",
    "rhdf5",
    "S4Vectors",
    "SummarizedExperiment"
  )
  missing_packages <- required_packages[
    !vapply(required_packages, requireNamespace, logical(1), quietly = TRUE)
  ]
  assert_true(
    length(missing_packages) == 0L,
    sprintf(
      "Missing required R packages: %s",
      paste(missing_packages, collapse = ", ")
    )
  )
}

read_required_dataset <- function(h5_path, dataset_path) {
  tryCatch(
    rhdf5::h5read(h5_path, dataset_path),
    error = function(error) {
      stop(
        sprintf(
          "Unable to read required Methx dataset %s from %s: %s",
          dataset_path,
          h5_path,
          conditionMessage(error)
        ),
        call. = FALSE
      )
    }
  )
}

validate_methx_schema <- function(h5_path) {
  h5_entries <- rhdf5::h5ls(h5_path, recursive = TRUE)
  h5_paths <- gsub("^/+", "", paste(h5_entries$group, h5_entries$name, sep = "/"))
  required_paths <- c(
    "beta",
    "cov",
    "rowData/seqnames",
    "rowData/start",
    "rowData/end",
    "rowData/width",
    "rowData/strand",
    "colData/sample_name",
    "metadata/genome",
    "metadata/schema_name",
    "metadata/schema_version"
  )
  missing_paths <- setdiff(required_paths, h5_paths)
  assert_true(
    length(missing_paths) == 0L,
    sprintf(
      "Methx HDF5 file is missing required datasets: %s",
      paste(missing_paths, collapse = ", ")
    )
  )

  schema_name <- as.character(read_required_dataset(h5_path, "/metadata/schema_name"))
  assert_true(
    identical(schema_name, "methx.custom-hdf5"),
    sprintf(
      "Unsupported Methx HDF5 schema %s; expected methx.custom-hdf5",
      paste(schema_name, collapse = ", ")
    )
  )
}

compare_assay_blocks <- function(
  source_assay,
  exported_assay,
  assay_name,
  tolerance,
  block_size_rows
) {
  assert_true(
    identical(dim(source_assay), dim(exported_assay)),
    sprintf("%s dimensions differ after export", assay_name)
  )

  source_row_count <- nrow(source_assay)
  for (block_start in seq.int(1L, source_row_count, by = block_size_rows)) {
    block_end <- min(block_start + block_size_rows - 1L, source_row_count)
    row_indices <- block_start:block_end
    source_values <- as.matrix(source_assay[row_indices, , drop = FALSE])
    exported_values <- as.matrix(exported_assay[row_indices, , drop = FALSE])

    source_missing <- is.na(source_values) | is.nan(source_values)
    exported_missing <- is.na(exported_values) | is.nan(exported_values)
    assert_true(
      isTRUE(all(source_missing == exported_missing)),
      sprintf(
        "%s missing-value mask differs in exported rows %d-%d",
        assay_name,
        block_start,
        block_end
      )
    )

    comparable_values <- !source_missing
    absolute_differences <- abs(
      source_values[comparable_values] - exported_values[comparable_values]
    )
    assert_true(
      all(absolute_differences <= tolerance),
      sprintf(
        "%s values differ beyond tolerance %g in exported rows %d-%d",
        assay_name,
        tolerance,
        block_start,
        block_end
      )
    )
  }
}

validate_exported_methrix_object <- function(
  source_beta,
  source_coverage,
  source_row_data,
  source_col_data,
  output_directory,
  beta_tolerance,
  block_size_rows
) {
  exported_object <- methrix::load_HDF5_methrix(output_directory)
  assert_true(
    methods::is(exported_object, "methrix"),
    "Methrix did not load the exported HDF5 directory as a methrix object"
  )

  exported_row_data <- as.data.frame(
    SummarizedExperiment::rowData(exported_object),
    stringsAsFactors = FALSE
  )
  exported_col_data <- as.data.frame(
    SummarizedExperiment::colData(exported_object),
    stringsAsFactors = FALSE
  )
  required_row_columns <- c("chr", "start", "end", "width", "strand")
  required_col_columns <- c("sample_id", "sample_name")
  assert_true(
    all(required_row_columns %in% names(exported_row_data)),
    "Exported methrix object is missing required CpG row metadata"
  )
  assert_true(
    all(required_col_columns %in% names(exported_col_data)),
    "Exported methrix object is missing required sample metadata"
  )
  assert_true(
    identical(as.character(exported_row_data$chr), as.character(source_row_data$chr)) &&
      identical(as.integer(exported_row_data$start), as.integer(source_row_data$start)) &&
      identical(as.integer(exported_row_data$end), as.integer(source_row_data$end)) &&
      identical(as.integer(exported_row_data$width), as.integer(source_row_data$width)) &&
      identical(as.character(exported_row_data$strand), as.character(source_row_data$strand)),
    "CpG coordinates differ after Methrix HDF5 export"
  )
  assert_true(
    identical(as.character(exported_col_data$sample_id), as.character(source_col_data$sample_id)) &&
      identical(as.character(exported_col_data$sample_name), as.character(source_col_data$sample_name)),
    "Sample metadata differs after Methrix HDF5 export"
  )

  compare_assay_blocks(
    source_beta,
    SummarizedExperiment::assay(exported_object, "beta"),
    assay_name = "beta",
    tolerance = beta_tolerance,
    block_size_rows = block_size_rows
  )
  compare_assay_blocks(
    source_coverage,
    SummarizedExperiment::assay(exported_object, "cov"),
    assay_name = "coverage",
    tolerance = 0,
    block_size_rows = block_size_rows
  )

  invisible(exported_object)
}

# Exports Methx custom-HDF5 output as a directory loadable by methrix::load_HDF5_methrix().
export_methx_h5_to_methrix <- function(
  methx_h5_path,
  output_directory,
  replace = FALSE,
  beta_tolerance = 1e-6,
  block_size_rows = 250000L,
  validate = TRUE
) {
  require_conversion_packages()
  assert_true(length(methx_h5_path) == 1L, "methx_h5_path must be a single path")
  assert_true(length(output_directory) == 1L, "output_directory must be a single path")
  assert_true(file.exists(methx_h5_path), sprintf("Methx HDF5 file does not exist: %s", methx_h5_path))
  assert_true(
    is.numeric(beta_tolerance) && length(beta_tolerance) == 1L &&
      is.finite(beta_tolerance) && beta_tolerance >= 0,
    "beta_tolerance must be one finite non-negative number"
  )
  assert_true(
    is.numeric(block_size_rows) && length(block_size_rows) == 1L &&
      is.finite(block_size_rows) && block_size_rows >= 1,
    "block_size_rows must be a positive number"
  )

  methx_h5_path <- normalizePath(methx_h5_path, mustWork = TRUE)
  output_directory <- normalizePath(output_directory, mustWork = FALSE)
  validate_methx_schema(methx_h5_path)

  sequence_names <- enc2utf8(as.character(read_required_dataset(methx_h5_path, "/rowData/seqnames")))
  start_positions <- as.integer(read_required_dataset(methx_h5_path, "/rowData/start"))
  end_positions <- as.integer(read_required_dataset(methx_h5_path, "/rowData/end"))
  widths <- as.integer(read_required_dataset(methx_h5_path, "/rowData/width"))
  strands <- as.character(read_required_dataset(methx_h5_path, "/rowData/strand"))
  sample_names <- enc2utf8(as.character(read_required_dataset(methx_h5_path, "/colData/sample_name")))
  genome_name <- enc2utf8(as.character(read_required_dataset(methx_h5_path, "/metadata/genome")))

  cpg_count <- length(sequence_names)
  sample_count <- length(sample_names)
  assert_true(cpg_count > 0L, "Methx HDF5 contains no CpG rows")
  assert_true(sample_count > 0L, "Methx HDF5 contains no samples")
  assert_true(
    length(start_positions) == cpg_count &&
      length(end_positions) == cpg_count &&
      length(widths) == cpg_count &&
      length(strands) == cpg_count,
    "Methx CpG row metadata lengths differ"
  )
  assert_true(
    all(start_positions >= 1L) &&
      all(end_positions >= start_positions) &&
      all(widths == end_positions - start_positions + 1L),
    "Methx CpG coordinates violate the 1-based closed-interval contract"
  )
  assert_true(
    all(strands %in% c("+", "-", "*")),
    "Methx HDF5 contains unsupported strand values"
  )
  assert_true(
    length(genome_name) == 1L && nzchar(genome_name),
    "Methx HDF5 metadata/genome must be a non-empty scalar"
  )

  beta_assay <- HDF5Array::HDF5Array(methx_h5_path, "beta")
  coverage_assay <- HDF5Array::HDF5Array(methx_h5_path, "cov")
  expected_dimensions <- c(cpg_count, sample_count)
  assert_true(
    identical(as.integer(dim(beta_assay)), expected_dimensions) &&
      identical(as.integer(dim(coverage_assay)), expected_dimensions),
    sprintf(
      "Methx assay dimensions must be %d CpGs x %d samples",
      cpg_count,
      sample_count
    )
  )

  source_row_data <- S4Vectors::DataFrame(
    chr = sequence_names,
    start = start_positions,
    end = end_positions,
    width = widths,
    strand = strands,
    row.names = NULL
  )
  source_col_data <- S4Vectors::DataFrame(
    sample_id = sample_names,
    sample_name = sample_names,
    row.names = sample_names
  )
  methrix_object <- getFromNamespace("create_methrix", "methrix")(
    beta_mat = beta_assay,
    cov_mat = coverage_assay,
    cpg_loci = source_row_data,
    is_hdf5 = TRUE,
    genome_name = genome_name,
    col_data = source_col_data,
    h5_dir = NULL,
    ref_cpg_dt = as.data.frame(source_row_data, stringsAsFactors = FALSE)
  )

  methrix::save_HDF5_methrix(
    m = methrix_object,
    dir = output_directory,
    replace = replace
  )

  if (validate) {
    validate_exported_methrix_object(
      source_beta = beta_assay,
      source_coverage = coverage_assay,
      source_row_data = source_row_data,
      source_col_data = source_col_data,
      output_directory = output_directory,
      beta_tolerance = beta_tolerance,
      block_size_rows = as.integer(block_size_rows)
    )
  }

  invisible(list(
    output_directory = output_directory,
    cpg_count = cpg_count,
    sample_count = sample_count,
    genome = genome_name,
    validated = validate
  ))
}
