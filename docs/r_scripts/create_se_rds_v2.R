#!/usr/bin/env Rscript

# 创建 se.rds 文件，使用R methrix兼容的方式
# 关键：将DelayedArray转换为内存中的数组以支持saveRDS

library(SummarizedExperiment)
library(HDF5Array)
library(GenomicRanges)

# 输出目录
output_dir <- "testdata/mCall/rust_output_20260222_112603_job36922017"
assays_h5 <- file.path(output_dir, "assays.h5")

cat("创建 se.rds 文件 (R methrix兼容格式)\n")
cat("==========================================\n")
cat("输出目录:", output_dir, "\n")
cat("assays.h5:", assays_h5, "\n\n")

# 读取assays数据
cat("读取 assays 数据到内存...\n")
beta <- h5read(assays_h5, "/assay001")
cov <- h5read(assays_h5, "/assay002")

n_cpgs <- nrow(beta)
n_samples <- ncol(beta)

cat("  维度:", n_cpgs, "x", n_samples, "\n")
cat("  Beta非NA值:", sum(!is.na(beta)), "\n")
cat("  Cov非零值:", sum(cov > 0), "\n\n")

# 读取 rowData
cat("读取 rowData...\n")
chr_data <- h5read(assays_h5, "/rowData/chr")
start_data <- h5read(assays_h5, "/rowData/start")
end_data <- h5read(assays_h5, "/rowData/end")
strand_data <- h5read(assays_h5, "/rowData/strand")

row_ranges <- GRanges(
    seqnames = chr_data,
    ranges = IRanges(start = start_data + 1, end = end_data),
    strand = strand_data
)

# 读取 colData
cat("读取 colData...\n")
sample_ids <- h5read(assays_h5, "/colData/sample_id")
col_data <- DataFrame(sample_id = sample_ids)

# 读取 metadata
cat("读取 metadata...\n")
genome_raw <- h5read(assays_h5, "/metadata/genome")
genome_str <- intToUtf8(genome_raw)
is_h5 <- h5read(assays_h5, "/metadata/is_h5")

metadata <- list(
    genome = genome_str,
    is_h5 = as.logical(is_h5[1])  # 确保是logical类型
)

# 创建 SummarizedExperiment (使用内存中的数组)
cat("\n创建 SummarizedExperiment (内存模式)...\n")
se <- SummarizedExperiment(
    assays = list(beta = beta, cov = cov),
    rowRanges = row_ranges,
    colData = col_data,
    metadata = metadata
)

cat("✓ SummarizedExperiment 创建成功\n")
cat("  类型:", class(se)[1], "\n")
cat("  维度:", nrow(se), "x", ncol(se), "\n")
cat("  genome:", metadata(se)$genome, "\n")
cat("  is_h5:", metadata(se)$is_h5, "\n\n")

# 保存 se.rds (内存中的数组可以序列化)
se_rds_file <- file.path(output_dir, "se.rds")
cat("保存 se.rds:", se_rds_file, "\n")
saveRDS(se, file = se_rds_file)
cat("✓ se.rds 保存成功\n")

# 检查文件大小
file_size <- file.info(se_rds_file)$size
cat("  文件大小:", format(file_size, units = "auto"), "\n\n")

# 验证
cat("验证 se.rds...\n")
se_loaded <- readRDS(se_rds_file)
cat("✓ se.rds 加载成功\n")
cat("  维度:", nrow(se_loaded), "x", ncol(se_loaded), "\n")
cat("  assays:", names(assays(se_loaded)), "\n")
cat("  genome:", metadata(se_loaded)$genome, "\n")
cat("  is_h5:", metadata(se_loaded)$is_h5, "\n\n")

cat("==========================================\n")
cat("完成！ se.rds 已创建\n")
cat("==========================================\n")
