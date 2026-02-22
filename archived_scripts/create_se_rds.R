#!/usr/bin/env Rscript

# 创建 se.rds 文件，支持 load_HDF5_methrix()

library(SummarizedExperiment)
library(HDF5Array)
library(GenomicRanges)

# 输出目录
output_dir <- "testdata/mCall/rust_output_20260222_112603_job36922017"
assays_h5 <- file.path(output_dir, "assays.h5")

cat("创建 se.rds 文件\n")
cat("==========================================\n")
cat("输出目录:", output_dir, "\n")
cat("assays.h5:", assays_h5, "\n\n")

# 读取assays数据以获取维度
beta <- h5read(assays_h5, "/assay001")
cov <- h5read(assays_h5, "/assay002")

n_cpgs <- nrow(beta)
n_samples <- ncol(beta)

cat("数据维度:", n_cpgs, "x", n_samples, "\n\n")

# 创建 DelayedMatrix assays
cat("创建 DelayedMatrix assays...\n")
beta_h5 <- HDF5Array(assays_h5, name = "/assay001")
cov_h5 <- HDF5Array(assays_h5, name = "/assay002")

# 读取 rowData 信息
cat("读取 rowData...\n")
# 现在rowData已经是过滤后的，维度匹配assays
chr_data <- h5read(assays_h5, "/rowData/chr")
start_data <- h5read(assays_h5, "/rowData/start")
end_data <- h5read(assays_h5, "/rowData/end")
strand_data <- h5read(assays_h5, "/rowData/strand")

# HDF5是0-based，R是1-based，需要转换start
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
genome_str <- intToUtf8(genome_raw)  # 转换整数向量为字符串
is_h5 <- h5read(assays_h5, "/metadata/is_h5")

metadata <- list(
    genome = genome_str,
    is_h5 = is_h5
)

# 创建 SummarizedExperiment
cat("\n创建 SummarizedExperiment...\n")
se <- SummarizedExperiment(
    assays = list(beta = beta_h5, cov = cov_h5),
    rowRanges = row_ranges,
    colData = col_data,
    metadata = metadata
)

cat("✓ SummarizedExperiment 创建成功\n")
cat("  类型:", class(se)[1], "\n")
cat("  维度:", nrow(se), "x", ncol(se), "\n")
cat("  genome:", metadata(se)$genome, "\n")
cat("  is_h5:", metadata(se)$is_h5, "\n\n")

# 保存为HDF5SummarizedExperiment格式
cat("保存 HDF5SummarizedExperiment...\n")

# saveHDF5SummarizedExperiment需要目录作为输出
# 我们使用输出目录作为基础
h5se_assays <- file.path(output_dir, "assays.h5")

# 直接保存，它会修改assays.h5文件
HDF5Array::saveHDF5SummarizedExperiment(se, h5se_assays, replace = TRUE)

cat("✓ HDF5SummarizedExperiment 保存成功\n")
cat("  文件:", h5se_assays, "\n\n")

# 为了向后兼容，也创建一个简单的se.rds引用
# 但这需要将DelayedArray转换为内存中的数组
# 由于数据量大，我们只保存metadata和维度信息
cat("创建 se_meta.rds (仅包含元数据)...\n")
se_meta <- list(
    nrow = nrow(se),
    ncol = ncol(se),
    genome = metadata(se)$genome,
    is_h5 = metadata(se)$is_h5,
    assays_h5 = file.path(h5se_dir, "assays.h5")
)
se_meta_file <- file.path(output_dir, "se_meta.rds")
saveRDS(se_meta, file = se_meta_file)
cat("✓ se_meta.rds 保存成功\n")

# 验证
cat("\n验证 HDF5SummarizedExperiment...\n")
se_loaded <- HDF5Array::loadHDF5SummarizedExperiment(h5se_assays)
cat("✓ HDF5SummarizedExperiment 加载成功\n")
cat("  类型:", class(se_loaded)[1], "\n")
cat("  维度:", nrow(se_loaded), "x", ncol(se_loaded), "\n")
cat("  genome:", metadata(se_loaded)$genome, "\n")
cat("  assays:", names(assays(se_loaded)), "\n")
cat("\n完成！\n")
