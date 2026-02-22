#!/usr/bin/env Rscript
# rust_to_methrix.R
# 将 Rust methrix-cli 生成的数据转换为可用的 methrix 对象

suppressMessages({
  library(methrix)
  library(rhdf5)
  library(SummarizedExperiment)
  library(GenomicRanges)
})

args <- commandArgs(trailingOnly = TRUE)

if (length(args) == 0) {
  cat("用法: Rscript rust_to_methrix.R <output_dir>\n")
  cat("\n功能: 将 Rust methrix-cli 生成的数据转换为 R methrix 对象\n")
  quit(status = 1)
}

output_dir <- args[1]
assays_h5 <- file.path(output_dir, "assays.h5")

cat("==========================================\n")
cat("Rust methrix-cli → R methrix 转换工具\n")
cat("==========================================\n")
cat("输入目录:", output_dir, "\n")
cat("assays.h5:", assays_h5, "\n\n")

# 检查文件存在
if (!file.exists(assays_h5)) {
  cat("错误: assays.h5 不存在:", assays_h5, "\n")
  quit(status = 1)
}

# 检查se.rds是否已存在
se_rds <- file.path(output_dir, "se.rds")
if (file.exists(se_rds)) {
  cat("✓ se.rds 已存在，直接加载...\n")
  se <- readRDS(se_rds)
} else {
  cat("创建 se.rds...\n")
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
  is_h5 <- h5read(assays_h5, "/metadata/is_h5")

  # 创建 SummarizedExperiment
  row_ranges <- GRanges(
    seqnames = chr_data,
    ranges = IRanges(start = start_data + 1, end = end_data),
    strand = strand_data
  )

  col_data <- DataFrame(sample_id = sample_ids)
  meta_data <- list(genome = genome_str, is_h5 = as.logical(is_h5[1]))

  se <- SummarizedExperiment(
    assays = list(beta = beta, cov = cov),
    rowRanges = row_ranges,
    colData = col_data,
    metadata = meta_data
  )

  # 保存 se.rds
  saveRDS(se, file = se_rds)
  cat("✓ se.rds 创建成功:", se_rds, "\n")
}

cat("\n数据信息:\n")
cat("  类型:", class(se)[1], "\n")
cat("  维度:", nrow(se), "x", ncol(se), "\n")
cat("  样本数:", ncol(se), "\n")
cat("  CpG位点数:", nrow(se), "\n")
cat("  genome:", metadata(se)$genome, "\n")
cat("  is_h5:", metadata(se)$is_h5, "\n")

# 测试基本功能
cat("\n测试基本功能:\n")
cat("==========================================\n")

cat("1. 获取 beta 矩阵...\n")
beta_mat <- assays(se)$beta
cat("   维度:", dim(beta_mat), "\n")
cat("   NA值:", sum(is.na(beta_mat)), "\n")
cat("   非NA值:", sum(!is.na(beta_mat)), "\n")

cat("\n2. 获取 coverage 矩阵...\n")
cov_mat <- assays(se)$cov
cat("   维度:", dim(cov_mat), "\n")
cat("   零值:", sum(cov_mat == 0), "\n")
cat("   非零值:", sum(cov_mat > 0), "\n")

cat("\n3. 获取样本信息...\n")
print(colData(se))

cat("\n4. 获取 CpG 位点信息 (前5个)...\n")
print(head(rowData(se), 5))

# 提供使用示例
cat("\n==========================================\n")
cat("使用示例:\n")
cat("==========================================\n\n")

cat("# 重新加载数据:\n")
cat("se <- readRDS('", se_rds, "')\n\n", sep="")

cat("# 获取甲基化数据:\n")
cat("beta <- assays(se)$beta\n")
cat("cov <- assays(se)$cov\n\n")

cat("# 获取特定样本:\n")
cat("sample1_beta <- se[, 1]\n\n")

cat("# 获取特定区域:\n")
cat("region <- se[1:1000, ]\n\n")

cat("==========================================\n")
cat("完成！ 数据已准备就绪\n")
cat("==========================================\n")
