#!/usr/bin/env Rscript

library(rhdf5)

assays_h5 <- "testdata/mCall/rust_output_20260222_110313_job36922006/assays.h5"

cat("检查 assays.h5 的详细结构\n")
cat("==========================================\n\n")

# assay001 (beta)
cat("assay001 (beta matrix):\n")

beta <- h5read(assays_h5, "/assays/assay001")
cat("维度:", dim(beta), "\n")
cat("前6个值 (线性索引):\n")
print(as.vector(beta)[1:6])

cat("\n\n第一行 (第一个样本):\n")
print(beta[1, 1:min(10, ncol(beta))])

cat("\n\n第一列 (第一个CpG位点):\n")
print(beta[1:min(10, nrow(beta)), 1])

# assay002 (cov)
cat("\n\nassay002 (coverage matrix):\n")
cov <- h5read(assays_h5, "/assays/assay002")
cat("维度:", dim(cov), "\n")
cat("前6个值:\n")
print(as.vector(cov)[1:6])
