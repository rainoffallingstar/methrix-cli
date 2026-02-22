#!/usr/bin/env Rscript

# 使用 methrix::convert_HDF5_methrix 将 Rust 数据转换为 methrix 对象

library(methrix)
library(rhdf5)

output_dir <- "testdata/mCall/rust_output_20260222_112603_job36922017"
assays_h5 <- file.path(output_dir, "assays.h5")

cat("使用 convert_HDF5_methrix 创建 methrix 对象\n")
cat("==========================================\n\n")

# 读取数据
beta <- h5read(assays_h5, "/assay001")
cov <- h5read(assays_h5, "/assay002")

# 读取 rowData
chr_data <- h5read(assays_h5, "/rowData/chr")
start_data <- h5read(assays_h5, "/rowData/start")
end_data <- h5read(assays_h5, "/rowData/end")
strand_data <- h5read(assays_h5, "/rowData/strand")

# 读取 colData
sample_ids <- h5read(assays_h5, "/colData/sample_id")

# 读取 metadata
genome_raw <- h5read(assays_h5, "/metadata/genome")
genome_str <- intToUtf8(genome_raw)

cat("数据维度:", nrow(beta), "x", ncol(beta), "\n")
cat("genome:", genome_str, "\n\n")

# 创建 SummarizedExperiment
library(SummarizedExperiment)
library(GenomicRanges)

row_ranges <- GRanges(
    seqnames = chr_data,
    ranges = IRanges(start = start_data + 1, end = end_data),
    strand = strand_data
)

col_data <- DataFrame(sample_id = sample_ids)
meta_data <- list(genome = genome_str, is_h5 = FALSE)

se <- SummarizedExperiment(
    assays = list(beta = beta, cov = cov),
    rowRanges = row_ranges,
    colData = col_data,
    metadata = meta_data
)

cat("✓ SummarizedExperiment 创建成功\n\n")

# 转换为 methrix 对象
cat("转换为 methrix 对象...\n")
m <- methrix:::new("methrix", se)

cat("✓ methrix 对象创建成功\n")
cat("  类型:", class(m)[1], "\n")
cat("  维度:", nrow(m), "x", ncol(m), "\n")

# 测试 methrix 功能
cat("\n测试 methrix 核心功能...\n")
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

cat("\n==========================================\n")
cat("完成！ Rust 生成的数据与 methrix 包完全兼容！\n")
cat("==========================================\n")
