#!/usr/bin/env Rscript
# load_h5_as_methrix.R
# 简单版：从 HDF5 创建可用的 methrix-like 对象

suppressMessages({
  library(methrix)
  library(rhdf5)
})

args <- commandArgs(trailingOnly = TRUE)

if (length(args) == 0) {
  cat("用法: Rscript load_h5_as_methrix.R <assays.h5路径或目录>\n\n")
  cat("示例:\n")
  cat("  Rscript load_h5_as_methrix.R output/assays.h5\n")
  cat("  Rscript load_h5_as_methrix.R output/\n\n")
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
cat("从 HDF5 加载甲基化数据\n")
cat("==========================================\n")
cat("HDF5 文件:", assays_h5, "\n\n")

# 读取数据
cat("读取数据...\n")
beta <- h5read(assays_h5, "/assay001")
cov <- h5read(assays_h5, "/assay002")
chr <- h5read(assays_h5, "/rowData/chr")
start <- h5read(assays_h5, "/rowData/start")
end <- h5read(assays_h5, "/rowData/end")
strand <- h5read(assays_h5, "/rowData/strand")
sample_ids <- h5read(assays_h5, "/colData/sample_id")
genome_raw <- h5read(assays_h5, "/metadata/genome")
genome <- intToUtf8(genome_raw)

cat("  ✓ 完成\n")
cat("    维度:", nrow(beta), "x", ncol(beta), "\n")
cat("    样本:", paste(sample_ids, collapse = ", "), "\n\n")

# 创建 SummarizedExperiment
suppressMessages({
  library(SummarizedExperiment)
  library(GenomicRanges)
})

cat("创建 SummarizedExperiment 对象...\n")
row_ranges <- GRanges(
  seqnames = chr,
  ranges = IRanges(start = start + 1, end = end),
  strand = strand
)

col_data <- DataFrame(sample_id = sample_ids)
rownames(col_data) <- sample_ids

meta_data <- list(
  genome = genome,
  is_h5 = FALSE
)

se <- SummarizedExperiment(
  assays = list(beta = beta, cov = cov),
  rowRanges = row_ranges,
  colData = col_data,
  metadata = meta_data
)

cat("✓ SummarizedExperiment 创建成功\n")
cat("  类型:", class(se)[1], "\n")
cat("  维度:", nrow(se), "x", ncol(se), "\n\n")

# 由于无法直接创建methrix对象，我们将其保存为se.rds
# 用户可以通过SummarizedExperiment接口使用
se_rds <- file.path(output_dir, "se.rds")
cat("保存 SummarizedExperiment:\n")
cat("  文件:", se_rds, "\n")
saveRDS(se, file = se_rds)
cat("✓ 保存成功\n")
cat("  大小:", format(file.info(se_rds)$size, units = "auto"), "\n\n")

# 演示功能
cat("==========================================\n")
cat("数据访问示例\n")
cat("==========================================\n\n")

cat("# 加载数据:\n")
cat("se <- readRDS('", se_rds, "')\n\n", sep="")

cat("# 获取 beta 矩阵:\n")
cat("beta <- assays(se)$beta\n")
cat("dim(beta)  # ", dim(beta), "\n\n", sep="")

cat("# 获取 coverage 矩阵:\n")
cat("cov <- assays(se)$cov\n")
cat("dim(cov)  # ", dim(cov), "\n\n", sep="")

cat("# 获取样本信息:\n")
cat("colData(se)\n")
cat("# DataFrame with ", ncol(se), " rows and 1 columns\n\n", sep="")

cat("# 获取位点信息:\n")
cat("rowData(se)\n")
cat("# DataFrame with ", nrow(se), " rows\n\n", sep="")

cat("# 获取特定样本:\n")
cat("sample1 <- se[, 1]\n")
cat("sample2 <- se[, 2]\n\n")

cat("# 获取特定区域:\n")
cat("region <- se[1:1000, ]\n")
cat("region_data <- assays(region)$beta\n\n")

cat("# 计算统计:\n")
cat("# 平均甲基化水平\n")
cat("mean_meth <- colMeans(assays(se)$beta, na.rm = TRUE)\n")
mean_meth_val <- colMeans(beta, na.rm = TRUE)
cat("# ", paste(round(mean_meth_val, 4), collapse = ", "), "\n\n", sep="")

cat("# 平均覆盖度\n")
cat("mean_cov <- colMeans(assays(se)$cov)\n")
mean_cov_val <- colMeans(cov)
cat("# ", paste(round(mean_cov_val, 1), collapse = ", "), "\n\n", sep="")

cat("# 位点统计\n")
cat("n_covered <- rowSums(assays(se)$cov > 0) > 0\n")
cat("sum(n_covered)  # ", sum(rowSums(cov) > 0), "\n\n", sep="")

cat("==========================================\n")
cat("完成！\n")
cat("==========================================\n\n")

cat("注意: 虽然 se 是 SummarizedExperiment 而不是 methrix 对象,\n")
cat("但可以通过 assays(), rowData(), colData() 等方法访问所有数据。\n")
cat("大多数甲基化分析都可以在 SummarizedExperiment 上进行。\n\n")
