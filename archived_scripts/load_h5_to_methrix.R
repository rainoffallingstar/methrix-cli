#!/usr/bin/env Rscript
# load_h5_to_methrix.R
# 从 Rust methrix-cli 生成的 HDF5 文件创建 methrix 对象

suppressMessages({
  library(methrix)
  library(rhdf5)
})

args <- commandArgs(trailingOnly = TRUE)

if (length(args) == 0) {
  cat("用法: Rscript load_h5_to_methrix.R <assays.h5路径>\n")
  cat("\n功能: 从 Rust methrix-cli 生成的 HDF5 文件创建 methrix 对象\n\n")
  cat("示例:\n")
  cat("  Rscript load_h5_to_methrix.R output/assays.h5\n")
  cat("  Rscript load_h5_to_methrix.R output/\n")
  quit(status = 1)
}

# 处理输入路径
input_path <- args[1]
if (dir.exists(input_path)) {
  # 是目录，查找 assays.h5
  assays_h5 <- file.path(input_path, "assays.h5")
} else if (file.exists(input_path)) {
  # 是文件
  assays_h5 <- input_path
} else {
  cat("错误: 找不到文件或目录:", input_path, "\n")
  quit(status = 1)
}

cat("==========================================\n")
cat("从 HDF5 创建 methrix 对象\n")
cat("==========================================\n")
cat("HDF5 文件:", assays_h5, "\n\n")

# 检查文件是否存在
if (!file.exists(assays_h5)) {
  cat("错误: HDF5 文件不存在:", assays_h5, "\n")
  quit(status = 1)
}

# 读取 HDF5 数据
cat("正在读取 HDF5 文件...\n")

# 1. 读取 assays
beta <- h5read(assays_h5, "/assay001")
cov <- h5read(assays_h5, "/assay002")

cat("  ✓ assay001 (beta):", dim(beta), "\n")
cat("  ✓ assay002 (cov):", dim(cov), "\n")

# 2. 读取 rowData
chr <- h5read(assays_h5, "/rowData/chr")
start <- h5read(assays_h5, "/rowData/start")
end <- h5read(assays_h5, "/rowData/end")
strand <- h5read(assays_h5, "/rowData/strand")

cat("  ✓ rowData:", length(chr), "个位点\n")

# 3. 读取 colData
sample_ids <- h5read(assays_h5, "/colData/sample_id")

cat("  ✓ colData:", length(sample_ids), "个样本\n")

# 4. 读取 metadata
genome_raw <- h5read(assays_h5, "/metadata/genome")
genome <- intToUtf8(genome_raw)
is_h5_raw <- h5read(assays_h5, "/metadata/is_h5")
is_h5 <- as.logical(is_h5_raw[1])

cat("  ✓ metadata: genome =", genome, "\n\n")

# 创建 methrix 对象
cat("创建 methrix 对象...\n")
cat("----------------------------------------\n")

# 方法1: 使用 new("methrix") 直接创建
# 首先创建 SummarizedExperiment
suppressMessages({
  library(SummarizedExperiment)
  library(GenomicRanges)
})

# 创建 GRanges 对象
row_ranges <- GRanges(
  seqnames = chr,
  ranges = IRanges(start = start + 1, end = end),  # HDF5是0-based，R是1-based
  strand = strand
)

# 创建 DataFrame
col_data <- DataFrame(sample_id = sample_ids)
rownames(col_data) <- sample_ids

# 创建 metadata
meta_data <- list(
  genome = genome,
  is_h5 = FALSE  # 内存模式，不是H5模式
)

# 创建 SummarizedExperiment
se <- SummarizedExperiment(
  assays = list(beta = beta, cov = cov),
  rowRanges = row_ranges,
  colData = col_data,
  metadata = meta_data
)

cat("✓ SummarizedExperiment 创建成功\n")
cat("  类型:", class(se)[1], "\n")
cat("  维度:", nrow(se), "x", ncol(se), "\n")
cat("  样本:", paste(colData(se)$sample_id, collapse = ", "), "\n\n")

# 转换为 methrix 对象
cat("转换为 methrix 对象...\n")

# 使用 methrix 包的内部构造函数
m <- methrix:::new("methrix", se)

cat("✓ methrix 对象创建成功！\n")
cat("  类型:", class(m)[1], "\n")
cat("  维度:", nrow(m), "x", ncol(m), "\n")
cat("  genome:", m@metadata$genome, "\n")
cat("  is_h5:", m@metadata$is_h5, "\n\n")

# 测试 methrix 函数
cat("测试 methrix 核心功能...\n")
cat("==========================================\n\n")

cat("1. get_stats() - 获取统计信息\n")
stats <- get_stats(m)
cat("  ✓ 成功\n")
print(head(stats, 3))

cat("\n2. coverage_filter() - 覆盖度过滤\n")
m_filtered <- coverage_filter(m, cov_thr = 5, min_samples = 1)
cat("  ✓ 成功\n")
cat("  过滤前:", nrow(m), "个位点\n")
cat("  过滤后:", nrow(m_filtered), "个位点\n")

cat("\n3. get_matrix() - 获取矩阵\n")
test_mat <- get_matrix(m, type = "beta", as_matrix = TRUE)
cat("  ✓ 成功\n")
cat("  矩阵维度:", dim(test_mat), "\n")

cat("\n==========================================\n")
cat("完成！ methrix 对象已创建\n")
cat("==========================================\n\n")

# 提供保存选项
cat("是否保存 methrix 对象？\n")
cat("对象将保存到:", file.path(dirname(assays_h5), "methrix_object.rds"), "\n")

# 保存 methrix 对象
output_rds <- file.path(dirname(assays_h5), "methrix_object.rds")
saveRDS(m, file = output_rds)
cat("✓ methrix 对象已保存:", output_rds, "\n")
cat("  大小:", format(file.info(output_rds)$size, units = "auto"), "\n\n")

# 使用示例
cat("==========================================\n")
cat("使用示例:\n")
cat("==========================================\n\n")

cat("# 重新加载 methrix 对象:\n")
cat("m <- readRDS('", output_rds, "')\n\n", sep="")

cat("# 获取统计信息:\n")
cat("stats <- get_stats(m)\n\n")

cat("# 覆盖度过滤:\n")
cat("m_filt <- coverage_filter(m, cov_thr = 10, min_samples = 2)\n\n")

cat("# 获取特定样本:\n")
cat("sample1 <- m[, 1]\n\n")

cat("# 获取特定区域:\n")
cat("region <- m[1:1000, ]\n\n")

cat("# 提取矩阵:\n")
cat("beta_mat <- get_matrix(m, type = 'beta', as_matrix = TRUE)\n")
cat("cov_mat <- get_matrix(m, type = 'cov', as_matrix = TRUE)\n\n")

cat("==========================================\n")
