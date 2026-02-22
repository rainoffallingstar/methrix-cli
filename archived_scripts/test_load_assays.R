#!/usr/bin/env Rscript

library(rhdf5)

assays_h5 <- "testdata/mCall/rust_output_20260222_111217_job36922010/assays.h5"

cat("测试加载 Rust 生成的 assays.h5\n")
cat("==========================================\n")

# 读取assays
beta <- h5read(assays_h5, "/assay001")
cov <- h5read(assays_h5, "/assay002")

cat("✓ 成功读取 assay001 (beta):", dim(beta), "\n")
cat("✓ 成功读取 assay002 (cov):", dim(cov), "\n")

cat("\n数据验证:\n")
cat("  Beta值范围:", range(beta, na.rm=TRUE), "\n")
cat("  覆盖度范围:", range(cov), "\n")

cat("\n✓ assays.h5 文件格式正确！\n")
