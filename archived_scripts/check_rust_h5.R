#!/usr/bin/env Rscript

library(rhdf5)

assays_h5 <- "testdata/mCall/rust_output_20260222_112603_job36922017/assays.h5"

cat("检查 Rust 生成的 assays.h5 结构\n")
cat("==========================================\n")

h5ls(assays_h5)
