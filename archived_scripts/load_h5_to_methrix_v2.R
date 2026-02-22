#!/usr/bin/env Rscript
# load_h5_to_methrix_v2.R
# 从 Rust methrix-cli 生成的 HDF5 文件创建 methrix 对象

suppressMessages({
  library(methrix)
  library(rhdf5)
})

args <- commandArgs(trailingOnly = TRUE)

if (length(args) == 0) {
  cat("用法: Rscript load_h5_to_methrix_v2.R <assays.h5路径或目录>\n")
  cat("\n示例:\n")
  cat("  Rscript load_h5_to_methrix_v2.R output/assays.h5\n")
  cat("  Rscript load_h5_to_methrix_v2.R output/\n")
  quit(status = 1)
}

# 处理输入路径
input_path <- args[1]
if (dir.exists(input_path)) {
  assays_h5 <- file.path(input_path, "assays.h5")
} else if (file.exists(input_path)) {
  assays_h5 <- input_path
} else {
  cat("错误: 找不到:", input_path, "\n")
  quit(status = 1)
}

cat("==========================================\n")
cat("从 HDF5 创建 methrix 对象\n")
cat("==========================================\n")
cat("HDF5 文件:", assays_h5, "\n\n")

# 读取 HDF5 数据
cat("读取 HDF5 数据...\n")
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
cat("    样本:", paste(sample_ids, collapse = ", "), "\n")
cat("    genome:", genome, "\n\n")

# 方法：使用 read_bedgraphs 读取临时 bedgraph 文件
# 这是最可靠的方法来创建 methrix 对象
cat("创建 methrix 对象...\n")
cat("----------------------------------------\n")

# 创建临时目录
temp_dir <- tempdir()
cat("临时目录:", temp_dir, "\n")

# 将数据写入临时 bedgraph 文件
cat("写入临时 bedgraph 文件...\n")
for (i in seq_along(sample_ids)) {
  sample_id <- sample_ids[i]
  # 清理样本名（移除特殊字符）
  safe_name <- gsub("[^A-Za-z0-9._-]", "_", sample_id)

  # 写入 beta 值的 bedgraph
  beta_file <- file.path(temp_dir, paste0(safe_name, "_beta.bdg"))
  df <- data.frame(
    chr = chr,
    start = start,
    end = end,
    beta = beta[, i]
  )
  # 只写入非NA的行
  df <- df[!is.na(df$beta), ]
  if (nrow(df) > 0) {
    write.table(df, beta_file, sep = "\t", row.names = FALSE, col.names = FALSE, quote = FALSE)
    cat("  写入:", beta_file, "-", nrow(df), "行\n")
  }

  # 写入覆盖度的 bedgraph
  cov_file <- file.path(temp_dir, paste0(safe_name, "_cov.bdg"))
  df_cov <- data.frame(
    chr = chr,
    start = start,
    end = end,
    coverage = cov[, i]
  )
  # 只写入有覆盖度的行
  df_cov <- df_cov[df_cov$coverage > 0, ]
  if (nrow(df_cov) > 0) {
    write.table(df_cov, cov_file, sep = "\t", row.names = FALSE, col.names = FALSE, quote = FALSE)
    cat("  写入:", cov_file, "-", nrow(df_cov), "行\n")
  }
}

# 使用 read_bedgraphs 创建 methrix 对象
cat("\n使用 read_bedgraphs 创建 methrix 对象...\n")
m <- read_bedgraphs(
  bedgraphs = list.files(temp_dir, pattern = "_beta.bdg$", full.names = TRUE),
  bedgraphs_cov = list.files(temp_dir, pattern = "_cov.bdg$", full.names = TRUE),
  genome = genome,
  strand_collapse = FALSE,
  n_threads = 2
)

cat("✓ methrix 对象创建成功！\n")
cat("  类型:", class(m)[1], "\n")
cat("  维度:", nrow(m), "x", ncol(m), "\n")
cat("  样本:", paste(colData(m)$sample_id, collapse = ", "), "\n")
cat("  genome:", m@metadata$genome, "\n\n")

# 测试 methrix 函数
cat("测试 methrix 核心功能...\n")
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

# 清理临时文件
cat("\n清理临时文件...\n")
unlink(list.files(temp_dir, pattern = "\\.bdg$", full.names = TRUE))
cat("✓ 临时文件已清理\n\n")

# 保存 methrix 对象
cat("==========================================\n")
output_dir <- dirname(assays_h5)
m_rds <- file.path(output_dir, "methrix_object.rds")
cat("保存 methrix 对象:\n")
cat("  文件:", m_rds, "\n")
saveRDS(m, file = m_rds)
cat("✓ 保存成功\n")
cat("  大小:", format(file.info(m_rds)$size, units = "auto"), "\n\n")

# 使用示例
cat("==========================================\n")
cat("使用示例:\n")
cat("==========================================\n\n")

cat("# 重新加载 methrix 对象:\n")
cat("m <- readRDS('", m_rds, "')\n\n", sep="")

cat("# 获取统计信息:\n")
cat("stats <- get_stats(m)\n")
cat("print(head(stats))\n\n")

cat("# 覆盖度过滤:\n")
cat("m_filt <- coverage_filter(m, cov_thr = 10, min_samples = 2)\n\n")

cat("# 获取矩阵:\n")
cat("beta_mat <- get_matrix(m, type = 'beta', as_matrix = TRUE)\n")
cat("cov_mat <- get_matrix(m, type = 'cov', as_matrix = TRUE)\n\n")

cat("# 区域汇总:\n")
cat("region_summary <- get_region_summary(m, chrom = 'chr1', start = 1, end = 1000000)\n\n")

cat("# PCA 分析:\n")
cat("pca_result <- methrix_pca(m, n_pc = 2)\n")
cat("plot_pca(pca_result)\n\n")

cat("==========================================\n")
cat("完成！\n")
cat("==========================================\n")
