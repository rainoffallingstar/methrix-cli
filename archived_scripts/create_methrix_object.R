#!/usr/bin/env Rscript

# 从 Rust 生成的数据创建完整的 methrix 对象

library(methrix)
library(rhdf5)

output_dir <- "testdata/mCall/rust_output_20260222_112603_job36922017"
assays_h5 <- file.path(output_dir, "assays.h5")

cat("创建 methrix 对象 (从 Rust 数据)\n")
cat("==========================================\n\n")

# 读取数据到内存
cat("读取数据...\n")
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

cat("✓ 数据读取完成\n")
cat("  维度:", nrow(beta), "x", ncol(beta), "\n")
cat("  genome:", genome_str, "\n\n")

# 创建 methrix 对象
cat("创建 methrix 对象...\n")
m <- create_methrix(
    beta_matrix = beta,
    cov_matrix = cov,
    chr = chr_data,
    start = start_data,
    end = end_data,
    strand = strand_data,
    sample_names = sample_ids,
    genome = genome_str,
    is_h5 = FALSE  # 内存模式，不是H5模式
)

cat("✓ methrix 对象创建成功\n")
cat("  类型:", class(m)[1], "\n")
cat("  维度:", nrow(m), "x", ncol(m), "\n")
cat("  genome:", m@metadata$genome, "\n")
cat("  is_h5:", m@metadata$is_h5, "\n\n")

# 测试 methrix 功能
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

cat("\n3. extract_region_summary...\n")
region_summary <- extract_region_summary(m, chrom = "chr1", start = 1, end = 1000000)
cat("✓ 成功\n")
print(region_summary)

# 保存 methrix 对象
cat("\n==========================================\n")
m_rds_file <- file.path(output_dir, "methrix_object.rds")
cat("保存 methrix 对象:", m_rds_file, "\n")
saveRDS(m, file = m_rds_file)
cat("✓ 保存成功\n")

file_size <- file.info(m_rds_file)$size
cat("  文件大小:", format(file_size, units = "auto"), "\n")

cat("\n==========================================\n")
cat("完成！ methrix 对象已创建并测试\n")
cat("==========================================\n")
