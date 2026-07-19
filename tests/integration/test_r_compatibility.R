#!/usr/bin/env Rscript

arguments <- commandArgs(trailingOnly = TRUE)
if (length(arguments) != 1L) {
  stop("Usage: Rscript tests/integration/test_r_compatibility.R <assays.h5>", call. = FALSE)
}

h5_path <- arguments[[1L]]
if (!file.exists(h5_path)) {
  stop(sprintf("HDF5 file does not exist: %s", h5_path), call. = FALSE)
}
if (!requireNamespace("rhdf5", quietly = TRUE)) {
  stop("The Bioconductor package 'rhdf5' is required", call. = FALSE)
}

assert_true <- function(condition, message) {
  if (!isTRUE(condition)) {
    stop(message, call. = FALSE)
  }
}

read_required_dataset <- function(path) {
  tryCatch(
    rhdf5::h5read(h5_path, path),
    error = function(error) {
      stop(sprintf("Unable to read required dataset %s: %s", path, conditionMessage(error)), call. = FALSE)
    }
  )
}

normalize_assay <- function(values, cpg_count, sample_count, assay_name) {
  assay_dimensions <- dim(values)
  assert_true(length(assay_dimensions) == 2L, sprintf("%s must be two-dimensional", assay_name))

  if (identical(as.integer(assay_dimensions), c(cpg_count, sample_count))) {
    return(values)
  }
  if (identical(as.integer(assay_dimensions), c(sample_count, cpg_count))) {
    return(t(values))
  }

  stop(
    sprintf(
      "%s dimensions %s do not match %d CpGs and %d samples",
      assay_name,
      paste(assay_dimensions, collapse = " x "),
      cpg_count,
      sample_count
    ),
    call. = FALSE
  )
}

root_attributes <- rhdf5::h5readAttributes(h5_path, "/")
assert_true(identical(as.integer(root_attributes$se_version), 2L), "Root attribute se_version must equal 2")

file_structure <- rhdf5::h5ls(h5_path, recursive = TRUE)
all_paths <- gsub("^/+", "", paste(file_structure$group, file_structure$name, sep = "/"))
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
  "metadata/is_h5"
)
missing_paths <- setdiff(required_paths, all_paths)
assert_true(length(missing_paths) == 0L, sprintf("Missing HDF5 paths: %s", paste(missing_paths, collapse = ", ")))

sequence_names <- enc2utf8(as.character(read_required_dataset("/rowData/seqnames")))
start_positions <- as.integer(read_required_dataset("/rowData/start"))
end_positions <- as.integer(read_required_dataset("/rowData/end"))
widths <- as.integer(read_required_dataset("/rowData/width"))
strands <- as.character(read_required_dataset("/rowData/strand"))
sample_names <- enc2utf8(as.character(read_required_dataset("/colData/sample_name")))
genome <- enc2utf8(as.character(read_required_dataset("/metadata/genome")))
is_h5 <- as.logical(read_required_dataset("/metadata/is_h5"))

cpg_count <- length(sequence_names)
sample_count <- length(sample_names)
assert_true(cpg_count > 0L, "rowData must contain at least one CpG")
assert_true(sample_count > 0L, "colData must contain at least one sample")
assert_true(length(start_positions) == cpg_count, "rowData/start length mismatch")
assert_true(length(end_positions) == cpg_count, "rowData/end length mismatch")
assert_true(length(widths) == cpg_count, "rowData/width length mismatch")
assert_true(length(strands) == cpg_count, "rowData/strand length mismatch")
assert_true(all(start_positions >= 1L), "rowData/start must use 1-based coordinates")
assert_true(all(end_positions >= start_positions), "rowData/end must not precede start")
assert_true(all(widths == end_positions - start_positions + 1L), "rowData/width violates the closed-coordinate contract")
assert_true(all(strands %in% c("+", "-", "*")), "rowData/strand contains unsupported values")
assert_true(length(genome) == 1L && nzchar(genome), "metadata/genome must be a non-empty scalar")
assert_true(length(is_h5) == 1L && isTRUE(is_h5), "metadata/is_h5 must be TRUE")

beta <- normalize_assay(read_required_dataset("/beta"), cpg_count, sample_count, "beta")
coverage <- normalize_assay(read_required_dataset("/cov"), cpg_count, sample_count, "cov")
assert_true(all(is.na(beta) | (is.finite(beta) & beta >= 0 & beta <= 1)), "beta contains values outside [0, 1]")
assert_true(all(is.finite(coverage) & coverage >= 0), "cov contains negative or non-finite values")
assert_true(all(coverage == floor(coverage)), "cov contains non-integral values")

if (
  requireNamespace("SummarizedExperiment", quietly = TRUE) &&
  requireNamespace("GenomicRanges", quietly = TRUE) &&
  requireNamespace("IRanges", quietly = TRUE) &&
  requireNamespace("S4Vectors", quietly = TRUE)
) {
  row_ranges <- GenomicRanges::GRanges(
    seqnames = sequence_names,
    ranges = IRanges::IRanges(start = start_positions, end = end_positions),
    strand = strands
  )
  column_data <- S4Vectors::DataFrame(sample_name = sample_names)
  summarized_experiment <- SummarizedExperiment::SummarizedExperiment(
    assays = list(beta = beta, cov = coverage),
    rowRanges = row_ranges,
    colData = column_data,
    metadata = list(genome = genome, is_h5 = is_h5)
  )

  assert_true(
    identical(as.integer(dim(summarized_experiment)), c(cpg_count, sample_count)),
    "SummarizedExperiment dimensions do not match the HDF5 contract"
  )
  assert_true(
    identical(SummarizedExperiment::assayNames(summarized_experiment), c("beta", "cov")),
    "SummarizedExperiment assay names are incorrect"
  )
}

cat(sprintf("PASS: validated %d CpGs and %d samples in %s\n", cpg_count, sample_count, h5_path))
