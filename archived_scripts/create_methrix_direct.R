#!/usr/bin/env Rscript
# create_methrix_direct.R
# 直接从 HDF5 创建 methrix 对象（不使用 bedgraph 中间步骤）

suppressMessages({
  library(methrix)
  library(rhdf5)
})

args <- commandArgs(trailingOnly = TRUE)

if (length(args) == 0) {
  cat("用法: Rscript create_methrix_direct.R <assays.h5路径或目录>\n\n")
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
cat("从 HDF5 直接创建 methrix 对象\n")
cat("==========================================\n")
cat("HDF5 文件:", assays_h5, "\n\n")

# 步骤 1: 读取 HDF5 数据
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
cat("    维度:", nrow(beta), "x", ncol(beta), "\n\n")

# 步骤 2: 创建 SummarizedExperiment
suppressMessages({
  library(SummarizedExperiment)
  library(GenomicRanges)
})

cat("步骤 2: 创建 SummarizedExperiment...\n")

row_ranges <- GRanges(
  seqnames = chr,
  ranges = IRanges(start = start + 1, end = end),
  strand = strand
)

col_data <- DataFrame(sample_id = sample_ids)
rownames(col_data) <- sample_ids

# 重要：添加必要的 metadata 以兼容 methrix
meta_data <- list(
  genome = genome,
  is_h5 = FALSE  # 内存模式
)

se <- SummarizedExperiment(
  assays = list(beta = beta, cov = cov),
  rowRanges = row_ranges,
  colData = col_data,
  metadata = meta_data
)

cat("  ✓ SummarizedExperiment 创建成功\n")
cat("    维度:", nrow(se), "x", ncol(se), "\n\n")

# 步骤 3: 设置正确的类名
cat("步骤 3: 设置为 methrix 对象...\n")

# 方法：使用 S4 系统直接修改类名
# 注意：这不是"官方"方法，但在功能上是等效的
class(se) <- c("methrix", class(se))

cat("  ✓ 类名设置完成\n")
cat("    新类型:", class(se)[1], "\n")
cat("    所有类:", paste(class(se), collapse = ", "), "\n\n")

# 步骤 4: 验证对象
cat("步骤 4: 验证 methrix 对象...\n")

cat("  检查 is_methrix 函数...\n")
tryCatch({
  result <- is_methrix(se)
  cat("  ✓ is_methrix:", result, "\n")
}, error = function(e) {
  cat("  ⚠ is_methrix 函数不存在或失败\n")
})

cat("  检查 slots...\n")
cat("    Slots:", paste(slotNames(se), collapse = ", "), "\n")

cat("  检查 metadata...\n")
cat("    genome:", se@metadata$genome, "\n")
cat("    is_h5:", se@metadata$is_h5, "\n\n")

# 步骤 5: 测试功能
cat("步骤 5: 测试核心功能...\n")
cat("==========================================\n\n")

cat("1. 访问 assays\n")
assay_data <- assays(se)
cat("  ✓ 成功\n")
cat("    beta 维度:", dim(assay_data$beta), "\n")
cat("    cov 维度:", dim(assay_data$cov), "\n")

cat("\n2. 访问 rowData\n")
row_data <- rowData(se)
cat("  ✓ 成功\n")
cat("    行数:", nrow(row_data), "\n")
cat("    前3行:\n")
print(head(row_data, 3))

cat("\n3. 访问 colData\n")
col_data <- colData(se)
cat("  ✓ 成功\n")
print(col_data)

cat("\n4. 子集操作\n")
test_sub <- se[, 1]
cat("  ✓ 成功\n")
cat("    维度:", dim(test_sub), "\n")

cat("\n==========================================\n")
cat("步骤 6: 保存对象\n")
cat("==========================================\n\n")

m_rds <- file.path(output_dir, "methrix_object.rds")
cat("保存文件:", m_rds, "\n")
saveRDS(se, file = m_rds)
cat("✓ 保存成功\n")
file_size <- file.info(m_rds)$size
cat("  大小:", format(file_size, units = "auto"), "\n\n")

# 验证重新加载
cat("验证重新加载...\n")
se_loaded <- readRDS(m_rds)
cat("  ✓ 重新加载成功\n")
cat("    类型:", class(se_loaded)[1], "\n")
cat("    维度:", nrow(se_loaded), "x", ncol(se_loaded), "\n\n")

# 使用示例
cat("==========================================\n")
cat("使用示例\n")
cat("==========================================\n\n")

cat("# 重新加载对象:\n")
cat("m <- readRDS('", m_rds, "')\n\n", sep="")

cat("# 基本操作:\n")
cat("# 注意: 虽然对象标记为 'methrix' 类,\n")
cat("# 但某些 methrix 特定函数可能需要真正的 methrix 对象\n")
cat("# 大多数操作使用 SummarizedExperiment 方法即可\n\n")

cat("# 获取数据:\n")
cat("beta <- assays(m)$beta\n")
cat("cov <- assays(m)$cov\n\n")

cat("# 样本信息:\n")
cat("colData(m)\n\n")

cat("# 位点信息:\n")
cat("rowData(m)\n\n")

cat("# 子集:\n")
cat("m[, 1]           # 第一个样本\n")
cat("m[1:1000, ]      # 前1000个位点\n")
cat("m[m$start < 1000, ]  # 特定区域\n\n")

cat("# 统计:\n")
cat("rowMeans(assays(m)$cov, na.rm = TRUE)  # 平均覆盖度\n")
cat("colMeans(assays(m)$beta, na.rm = TRUE)  # 平均甲基化\n\n")

cat("==========================================\n")
cat("完成！\n")
cat("==========================================\n")
