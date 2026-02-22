#!/usr/bin/env Rscript

library(rhdf5)

assays_h5 <- "testdata/mCall/rust_output_20260222_111217_job36922010/assays.h5"

cat("检查 assay001 (beta):\n")
beta <- h5read(assays_h5, "/assay001")
cat("  维度:", dim(beta), "\n")
cat("  类型:", class(beta), "\n")
cat("  NA数量:", sum(is.na(beta)), "\n")
cat("  非NA数量:", sum(!is.na(beta)), "\n")
cat("  前10个值:\n")
print(head(beta[,1], 10))

cat("\n检查 assay002 (cov):\n")
cov <- h5read(assays_h5, "/assay002")
cat("  维度:", dim(cov), "\n")
cat("  类型:", class(cov), "\n")
cat("  NA数量:", sum(is.na(cov)), "\n")
cat("  非NA数量:", sum(!is.na(cov)), "\n")
cat("  前10个值:\n")
print(head(cov[,1], 10))
