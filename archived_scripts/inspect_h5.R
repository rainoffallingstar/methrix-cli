#!/usr/bin/env Rscript

library(rhdf5)

h5_file <- "testdata/mCall/rust_output_20260222_104621_job36921990/methrix_data.h5"

cat("检查 HDF5 文件结构:", h5_file, "\n")
cat("==========================================\n\n")

cat("根级别对象:\n")
print(h5ls(h5_file))

cat("\n\nassays/beta 维度:\n")
beta <- h5read(h5_file, "assays/beta")
cat("dim:", dim(beta), "\n")
cat("前几个值:", head(beta), "\n")

cat("\nrowData/chr 前5个:\n")
chr <- h5read(h5_file, "rowData/chr")
print(head(chr))

cat("\ncolData/sample_id:\n")
sample_id <- h5read(h5_file, "colData/sample_id")
print(sample_id)

cat("\nmetadata/genome:\n")
genome <- h5read(h5_file, "metadata/genome")
print(genome)
