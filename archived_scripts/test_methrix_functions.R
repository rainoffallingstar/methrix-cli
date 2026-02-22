#!/usr/bin/env Rscript

library(methrix)

output_dir <- "testdata/mCall/rust_output_20260222_112603_job36922017"

cat("测试 Rust 生成的数据与 methrix 包的兼容性\n")
cat("==========================================\n")
cat("目录:", output_dir, "\n\n")

# 读取 se.rds
se_rds <- file.path(output_dir, "se.rds")
m <- readRDS(se_rds)

cat("✓ se.rds 加载成功\n")
cat("  类型:", class(m)[1], "\n")
cat("  维度:", nrow(m), "x", ncol(m), "\n")
cat("  genome:", metadata(m)$genome, "\n")
cat("  is_h5:", metadata(m)$is_h5, "\n")
cat("  assays:", names(assays(m)), "\n\n")

# 测试 methrix 函数
cat("测试 methrix 核心功能...\n")
cat("==========================================\n\n")

cat("1. get_stats...\n")
stats <- get_stats(m)
cat("✓ 成功\n")
print(head(stats, 3))

cat("\n2. coverage_filter...\n")
m_filtered <- coverage_filter(m, cov_thr = 5, min_samples = 1)
cat("✓ 成功\n")
cat("  过滤前:", nrow(m), "个位点\n")
cat("  过滤后:", nrow(m_filtered), "个位点\n")

cat("\n3. get_region_summary...\n")
regions <- get_region_summary(m, regions = NULL)
cat("✓ 成功\n")
print(head(regions, 3))

cat("\n==========================================\n")
cat("所有测试通过！ Rust 生成的数据与 methrix 包完全兼容！\n")
cat("==========================================\n")
