#!/usr/bin/env Rscript
# create_methrix_from_h5.R
# 直接从 HDF5 创建 methrix 对象的完整解决方案

suppressMessages({
  library(methrix)
  library(rhdf5)
})

args <- commandArgs(trailingOnly = TRUE)

if (length(args) == 0) {
  cat("用法: Rscript create_methrix_from_h5.R <assays.h5路径或目录>\n\n")
  quit(status = 1)
}

# 处理输入路径
input_path <- args[1]
if (dir.exists(input_path)) {
  assays_h5 <- file.path(input_path, "assays.h5")
  output_dir <- input_path
} else if (file.exists(input_path)) {
  assays_h5 <- input_path
  output_dir <- dirname(input_path)
} else {
  cat("错误: 找不到:", input_path, "\n")
  quit(status = 1)
}

cat("==========================================\n")
cat("从 HDF5 创建 methrix 对象\n")
cat("==========================================\n")
cat("HDF5 文件:", assays_h5, "\n\n")

# 读取 HDF5 数据
cat("步骤 1: 读取 HDF5 数据...\n")
beta <- h5read(assays_h5, "/assay001")
cov <- h5read(assays_h5, "/assay002")
chr <- h5read(assays_h5, "/rowData/chr")
start <- h5read(assays_h5, "/rowData/start")
end <- h5read(assays_h5, "/rowData/end")
strand <- h5read(assays_h5, "/rowData/strand")
sample_ids <- h5read(assays_h5, "/colData/sample_id")
genome_raw <- h5read(assays_h5, "/metadata/genome")
genome <- intToUtf8(genome_raw)

cat("  ✓ 数据读取完成\n")
cat("    维度:", nrow(beta), "x", ncol(beta), "\n")
cat("    样本:", paste(sample_ids, collapse = ", "), "\n\n")

# 方法：使用 read_bedgraphs，但先创建临时 bedgraph 文件
cat("步骤 2: 创建临时 bedgraph 文件...\n")
suppressMessages({
  library(SummarizedExperiment)
  library(GenomicRanges)
})

temp_dir <- tempdir()
cat("  临时目录:", temp_dir, "\n")

# 为每个样本创建 bedgraph 文件
bedgraph_files <- character(0)
cov_files <- character(0)

for (i in seq_along(sample_ids)) {
  sample_id <- sample_ids[i]
  # 清理文件名
  safe_name <- gsub("[^A-Za-z0-9._-]", "_", sample_id)

  # 创建 beta 数据的 bedgraph
  beta_df <- data.frame(
    chr = chr,
    start = start,
    end = end,
    beta = beta[, i]
  )
  # 只保留非NA的行
  beta_df <- beta_df[!is.na(beta_df$beta), ]

  if (nrow(beta_df) > 0) {
    beta_file <- file.path(temp_dir, paste0(safe_name, ".bdg"))
    write.table(beta_df, beta_file,
                sep = "\t", row.names = FALSE,
                col.names = FALSE, quote = FALSE)
    bedgraph_files <- c(bedgraph_files, beta_file)
    cat("  创建:", basename(beta_file), "-", nrow(beta_df), "行\n")
  }

  # 创建 coverage 数据的 bedgraph
  cov_df <- data.frame(
    chr = chr,
    start = start,
    end = end,
    cov = cov[, i]
  )
  # 只保留有覆盖度的行
  cov_df <- cov_df[cov_df$cov > 0, ]

  if (nrow(cov_df) > 0) {
    cov_file <- file.path(temp_dir, paste0(safe_name, ".cov.bdg"))
    write.table(cov_df, cov_file,
                sep = "\t", row.names = FALSE,
                col.names = FALSE, quote = FALSE)
    cov_files <- c(cov_files, cov_file)
    cat("  创建:", basename(cov_file), "-", nrow(cov_df), "行\n")
  }
}

cat("\n步骤 3: 使用 read_bedgraphs 创建 methrix 对象...\n")
cat("  这可能需要几分钟...\n")

# 调用 read_bedgraphs
m <- read_bedgraphs(
  files = bedgraph_files,
  coldata = data.frame(sample_id = sample_ids),
  ref_build = genome,
  n_threads = 2,
  verbose = FALSE
)

cat("\n✓ methrix 对象创建成功！\n")
cat("  类型:", class(m)[1], "\n")
cat("  维度:", nrow(m), "x", ncol(m), "\n")
cat("  样本:", paste(colData(m)$sample_id, collapse = ", "), "\n")
cat("  genome:", m@metadata$genome, "\n")
cat("  is_h5:", m@metadata$is_h5, "\n\n")

# 清理临时文件
cat("步骤 4: 清理临时文件...\n")
unlink(c(bedgraph_files, cov_files))
cat("  ✓ 临时文件已清理\n\n")

# 测试 methrix 函数
cat("步骤 5: 测试 methrix 核心功能...\n")
cat("==========================================\n\n")

cat("1. get_stats()\n")
stats <- get_stats(m)
cat("  ✓ 成功\n")
print(head(stats, 3))

cat("\n2. coverage_filter()\n")
m_filtered <- coverage_filter(m, cov_thr = 5, min_samples = 1)
cat("  ✓ 成功\n")
cat("  过滤前:", nrow(m), "个位点\n")
cat("  过滤后:", nrow(m_filtered), "个位点\n")

cat("\n3. get_matrix()\n")
test_mat <- get_matrix(m, type = "beta", as_matrix = TRUE)
cat("  ✓ 成功\n")
cat("  矩阵维度:", dim(test_mat), "\n")

cat("\n4. extract_region_summary()\n")
region_summary <- extract_region_summary(m, chrom = "chr1", start = 1, end = 1000000)
cat("  ✓ 成功\n")
print(region_summary)

# 保存 methrix 对象
cat("\n==========================================\n")
cat("保存 methrix 对象\n")
cat("==========================================\n")

m_rds <- file.path(output_dir, "methrix_object.rds")
cat("文件:", m_rds, "\n")
saveRDS(m, file = m_rds)
cat("✓ 保存成功\n")
file_size <- file.info(m_rds)$size
cat("  大小:", format(file_size, units = "auto"), "\n\n")

# 使用示例
cat("==========================================\n")
cat("使用示例\n")
cat("==========================================\n\n")

cat("# 重新加载 methrix 对象:\n")
cat("m <- readRDS('", m_rds, "')\n\n", sep="")

cat("# 获取统计信息:\n")
cat("stats <- get_stats(m)\n")
cat("print(stats)\n\n")

cat("# 覆盖度过滤:\n")
cat("m_filt <- coverage_filter(m, cov_thr = 10, min_samples = 2)\n\n")

cat("# 提取矩阵:\n")
cat("beta_mat <- get_matrix(m, type = 'beta', as_matrix = TRUE)\n")
cat("cov_mat <- get_matrix(m, type = 'cov', as_matrix = TRUE)\n\n")

cat("# 区域汇总:\n")
cat("region_sum <- get_region_summary(m, 'chr1', 1, 1000000)\n\n")

cat("# PCA 分析:\n")
cat("pca_res <- methrix_pca(m, n_pc = 2)\n")
cat("plot_pca(pca_res)\n\n")

cat("# 绘图:\n")
cat("plot_stats(m)\n")
cat("plot_coverage(m)\n\n")

cat("==========================================\n")
cat("完成！ methrix 对象已创建并保存\n")
cat("==========================================\n")
