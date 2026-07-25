#!/usr/bin/env Rscript

#===========================================================
# methx R 兼容性验证脚本
#===========================================================

cat("==========================================\n")
cat("methx R 兼容性验证\n")
cat("==========================================\n")
cat("开始时间:", Sys.time(), "\n\n")

#===========================================================
# 加载必要的库
#===========================================================

cat(">>> 加载 R 库...\n")
suppressPackageStartupMessages({
  if (!require("methrix", quietly = TRUE)) {
    stop("错误: methrix 包未安装。请运行: BiocManager::install('methrix')")
  }
  if (!require("HDF5Array", quietly = TRUE)) {
    stop("错误: HDF5Array 包未安装")
  }
})

cat("✓ 库加载成功\n\n")

#===========================================================
# 设置路径
#===========================================================

H5_FILE <- "testdata/mCall/rust_output/methrix_data.h5"
R_REFERENCE_DIR <- "testdata/mCall/methrixh5"

if (!file.exists(H5_FILE)) {
  cat("错误: HDF5 文件不存在:", H5_FILE, "\n")
  cat("请先运行 sbatch 脚本生成 HDF5 文件\n")
  stop(1)
}

cat(">>> 文件路径:\n")
cat("  HDF5 文件:", H5_FILE, "\n")
cat("  R 参考目录:", R_REFERENCE_DIR, "\n\n")

#===========================================================
# 测试 1: 加载 HDF5 文件
#===========================================================

cat(">>> 测试 1: 加载 HDF5 文件...\n")
cat("==========================================\n")

tryCatch({
  m <- load_HDF5_methrix(H5_FILE)
  cat("✓ HDF5 文件加载成功\n")
  cat("  样本数:", ncol(m), "\n")
  cat("  CpG 位点数:", nrow(m), "\n")
  cat("  基因组:", m@metadata$genome, "\n")
  cat("  格式:", ifelse(m@metadata$is_h5, "HDF5", "其他"), "\n\n")
}, error = function(e) {
  cat("✗ HDF5 文件加载失败\n")
  cat("  错误:", conditionMessage(e), "\n")
  stop(1)
})

#===========================================================
# 测试 2: 数据结构验证
#===========================================================

cat(">>> 测试 2: 数据结构验证...\n")
cat("==========================================\n")

# 检查 assays
cat("检查 assays:\n")
cat("  - beta: ", class(assays(m)$beta), "\n")
cat("  - cov: ", class(assays(m)$cov), "\n")

# 检查 rowData
cat("检查 rowData:\n")
cat("  - 行数:", nrow(rowData(m)), "\n")
cat("  - 列:", paste(colnames(rowData(m)), collapse=", "), "\n")

# 检查 colData
cat("检查 colData:\n")
cat("  - 样本:", paste(colData(m)$sample_id, collapse=", "), "\n")

# 检查 metadata
cat("检查 metadata:\n")
cat("  - genome:", m@metadata$genome, "\n")
cat("  - is_h5:", m@metadata$is_h5, "\n\n")

#===========================================================
# 测试 3: 统计信息
#===========================================================

cat(">>> 测试 3: 统计信息...\n")
cat("==========================================\n")

tryCatch({
  stats <- get_stats(m)
  cat("✓ 统计信息获取成功\n")
  print(stats)
  cat("\n")
}, error = function(e) {
  cat("✗ 统计信息获取失败\n")
  cat("  错误:", conditionMessage(e), "\n")
})

#===========================================================
# 测试 4: 覆盖度分布
#===========================================================

cat(">>> 测试 4: 覆盖度分布...\n")
cat("==========================================\n")

tryCatch({
  # 获取覆盖度矩阵
  cov_mat <- get_matrix(m, type = "cov")
  cat("覆盖度矩阵维度:", dim(cov_mat), "\n")

  # 计算每个样本的平均覆盖度
  mean_cov <- rowMeans(cov_mat, na.rm = TRUE)
  cat("平均覆盖度:\n")
  print(round(mean_cov, 2))
  cat("\n")
}, error = function(e) {
  cat("✗ 覆盖度计算失败\n")
  cat("  错误:", conditionMessage(e), "\n")
})

#===========================================================
# 测试 5: 与 R methrix 输出对比 (如果存在)
#===========================================================

if (file.exists(file.path(R_REFERENCE_DIR, "se.rds"))) {
  cat(">>> 测试 5: 与 R methrix 输出对比...\n")
  cat("==========================================\n")

  tryCatch({
    # 加载 R methrix 输出
    m_r <- readRDS(file.path(R_REFERENCE_DIR, "se.rds"))

    cat("R methrix 对象:\n")
    cat("  样本数:", ncol(m_r), "\n")
    cat("  CpG 位点数:", nrow(m_r), "\n")

    # 对比维度
    if (nrow(m) == nrow(m_r) && ncol(m) == ncol(m_r)) {
      cat("✓ 维度匹配\n")
    } else {
      cat("✗ 维度不匹配\n")
      cat("  Rust:", nrow(m), "x", ncol(m), "\n")
      cat("  R:", nrow(m_r), "x", ncol(m_r), "\n")
    }

    # 对比样本名称
    rust_samples <- colData(m)$sample_id
    r_samples <- colData(m_r)$sample_id

    if (all(sort(rust_samples) == sort(r_samples))) {
      cat("✓ 样本名称匹配\n")
    } else {
      cat("✗ 样本名称不匹配\n")
    }

    cat("\n")
  }, error = function(e) {
    cat("✗ R 对象加载失败\n")
    cat("  错误:", conditionMessage(e), "\n")
  })
}

#===========================================================
# 测试 6: QC 报告验证
#===========================================================

cat(">>> 测试 6: QC 报告验证...\n")
cat("==========================================\n")

QC_XLSX <- "testdata/mCall/rust_output/CpG_coverage.xlsx"

if (file.exists(QC_XLSX)) {
  cat("✓ QC 报告已生成:", QC_XLSX, "\n")
  cat("  文件大小:", format(file.info(QC_XLSX)$size, units = "auto"), "\n")
} else {
  cat("✗ QC 报告不存在\n")
}

cat("\n")

#===========================================================
# 完成
#===========================================================

cat("==========================================\n")
cat("验证完成!\n")
cat("结束时间:", Sys.time(), "\n")
cat("==========================================\n")

# 保存验证结果
OUTPUT_FILE <- "testdata/mCall/rust_output/r_verification_results.txt"

sink(OUTPUT_FILE)
cat("methx R 兼容性验证结果\n")
cat("==========================================\n")
cat("验证时间:", Sys.time(), "\n\n")
cat("HDF5 文件:", H5_FILE, "\n\n")
cat("基本信息:\n")
cat("  样本数:", ncol(m), "\n")
cat("  CpG 位点数:", nrow(m), "\n")
cat("  基因组:", m@metadata$genome, "\n\n")

stats <- get_stats(m)
cat("统计信息:\n")
print(stats)
sink()

cat("验证结果已保存:", OUTPUT_FILE, "\n")
