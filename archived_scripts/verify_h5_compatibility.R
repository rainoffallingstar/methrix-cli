#!/usr/bin/env Rscript

#===========================================================
# 验证 Rust 生成的 HDF5 文件与 R methrix 包的兼容性
#===========================================================

cat("==========================================")
cat("Methrix CLI HDF5 R 兼容性验证")
cat("==========================================")
cat("开始时间:", Sys.time(), "\n")

#===========================================================
# 加载必要的库
#===========================================================

cat(">>> 加载 R 库...\n")

suppressMessages({
  if (!require("BiocManager", quietly = TRUE)) {
    install.packages("BiocManager")
  }

  # 安装/加载 methrix 和相关包
  if (!require("methrix", quietly = TRUE)) {
    BiocManager::install("methrix")
  }
  if (!require("HDF5Array", quietly = TRUE)) {
    BiocManager::install("HDF5Array")
  }
})

library(methrix)
library(HDF5Array)

cat("✓ 库加载成功\n\n")

#===========================================================
# 设置路径
#===========================================================

# 使用最新的Rust生成的assays.h5
rust_output_dirs <- list.files("testdata/mCall", pattern = "rust_output_.*", full.names = TRUE)

if (length(rust_output_dirs) == 0) {
  cat("错误: 未找到 Rust 输出目录\n")
  cat("请先运行 SLURM 任务生成 HDF5 文件\n")
  quit(status = 1)
}

# 按修改时间排序，使用最新的输出目录
output_dir <- rust_output_dirs[which.max(file.mtime(rust_output_dirs))]
h5_file <- file.path(output_dir, "methrix_data.h5")

cat(">>> 文件路径\n")
cat("输出目录:", output_dir, "\n")
cat("HDF5 文件:", h5_file, "\n\n")

#===========================================================
# 检查文件存在性
#===========================================================

if (!file.exists(h5_file)) {
  cat("错误: HDF5 文件不存在:", h5_file, "\n")
  quit(status = 1)
}

cat("✓ HDF5 文件存在\n")
cat("  文件大小:", format(file.info(h5_file)$size, units = "auto"), "\n\n")

#===========================================================
# 测试 1: 加载 HDF5 文件
#===========================================================

cat(">>> 测试 1: 加载 HDF5 文件...\n")
cat("==========================================\n")

tryCatch({
  # 方法1: 尝试使用 load_HDF5_methrix (methrix包函数)
  cat("方法1: 使用 load_HDF5_methrix...\n")
  m <- load_HDF5_methrix(output_dir)
  cat("✓ HDF5 文件加载成功!\n")
  cat("  对象类型:", class(m)[1], "\n")
  cat("  样本数:", ncol(m), "\n")
  cat("  CpG 位点数:", nrow(m), "\n")
  cat("  基因组:", m@metadata$genome, "\n")
  cat("  格式:", ifelse(m@metadata$is_h5, "HDF5", "其他"), "\n")
}, error = function(e) {
  cat("✗ load_HDF5_methrix 失败:\n")
  cat("  错误:", conditionMessage(e), "\n\n")

  # 方法2: 尝试使用 HDF5Array::loadHDF5SummarizedExperiment
  cat("方法2: 使用 HDF5Array::loadHDF5SummarizedExperiment...\n")
  tryCatch({
    se <- HDF5Array::loadHDF5SummarizedExperiment(h5_file)
    cat("✓ 使用 HDF5Array 加载成功!\n")
    cat("  对象类型:", class(se)[1], "\n")

    # 转换为 methrix 对象
    m <- methrix::as_methrix(se)
    cat("✓ 转换为 methrix 对象成功!\n")
    cat("  样本数:", ncol(m), "\n")
    cat("  CpG 位点数:", nrow(m), "\n")
    cat("  基因组:", m@metadata$genome, "\n")
    cat("  格式:", ifelse(m@metadata$is_h5, "HDF5", "其他"), "\n")
  }, error = function(e2) {
    cat("✗ HDF5Array 加载也失败:\n")
    cat("  错误:", conditionMessage(e2), "\n")
    quit(status = 1)
  })
})

cat("\n")

#===========================================================
# 测试 2: 数据结构验证
#===========================================================

cat(">>> 测试 2: 数据结构验证...\n")
cat("==========================================\n")

# 检查 assays
cat("assays (甲基化和覆盖度矩阵):\n")
cat("  - beta 维度:", dim(assays(m)$beta), "\n")
cat("  - cov 维度:", dim(assays(m)$cov), "\n")

# 检查 rowData
cat("\nrowData (CpG 位点信息):\n")
cat("  - 行数:", nrow(rowData(m)), "\n")
cat("  - 列数:", ncol(rowData(m)), "\n")
cat("  - 列名:", paste(colnames(rowData(m)), "\n")

# 检查 colData
cat("\ncolData (样本信息):\n")
cat("  - 样本数:", nrow(colData(m)), "\n")
cat("  - 样本名:\n")
print(colData(m)$sample_id)

# 检查 metadata
cat("\nmetadata (元数据):\n")
cat("  - genome:", m@metadata$genome, "\n")
cat("  - is_h5:", m@metadata$is_h5, "\n"

# 检查 HDF5 文件结构
cat("\nHDF5 文件结构:\n")
tryCatch({
  h5ls(h5_file)
})

cat("\n")

#===========================================================
# 测试 3: 数据完整性检查
#===========================================================

cat(">>> 测试 3: 数据完整性...\n")
cat("==========================================\n")

# 检查 NA 值比例
beta_mat <- assays(m)$beta
cov_mat <- assays(m)$cov

total_cells <- length(beta_mat)
na_beta <- sum(is.na(beta_mat))
na_cov <- sum(is.na(cov_mat))

cat("数据完整性:\n")
cat("  - 总单元格:", total_cells, "\n")
cat("  - beta NA 数量:", na_beta,
      sprintf("(%.2f%%)", 100 * na_beta / total_cells), "\n")
cat("  - cov NA 数量:", na_cov,
      sprintf("(%.2f%%)", 100 * na_cov / total_cells), "\n")

# 检查数据范围
valid_beta <- beta_mat[!is.na(beta_mat)]
cat("\nBeta 值范围:\n")
cat("  - 最小值:", min(valid_beta), "\n")
cat("  - 最大值:", max(valid_beta), "\n")
cat("  - 平均值:", mean(valid_beta), "\n")

valid_cov <- cov_mat[!is.na(cov_mat)]
cat("\n覆盖度范围:\n")
cat("  - 最小值:", min(valid_cov), "\n")
cat("  - 最大值:", max(valid_cov), "\n")
cat("  - 平均值:", mean(valid_cov), "\n")

cat("\n")

#===========================================================
# 测试 4: 统计信息
#===========================================================

cat(">>> 测试 4: 统计信息...\n")
cat("==========================================\n")

tryCatch({
  stats <- get_stats(m)
  cat("✓ 统计信息获取成功\n\n")
  print(stats)
}, error = function(e) {
  cat("✗ 统计信息获取失败\n")
  cat("  错误:", conditionMessage(e), "\n")
})

cat("\n")

#===========================================================
# 测试 5: 与 R methrix 输出对比
#===========================================================

cat(">>> 测试 5: 与 R methrix 输出对比...\n")
cat("==========================================\n")

r_output_dir <- "testdata/mCall/methrixh5"
r_cov_xlsx <- file.path(r_output_dir, "CpG_coverage.xlsx")

if (dir.exists(r_output_dir)) {
  cat("R methrix 输出目录存在:", r_output_dir, "\n")

  # 尝试加载 R methrix 数据
  r_se_file <- file.path(r_output_dir, "se.rds")

  if (file.exists(r_se_file)) {
    cat("\n发现 R methrix 对象:", r_se_file, "\n")

    tryCatch({
      m_r <- readRDS(r_se_file)

      cat("\nR methrix 对象信息:\n")
      cat("  样本数:", ncol(m_r), "\n")
      cat("  CpG 位点数:", nrow(m_r), "\n")

      # 对比维度
      if (nrow(m) == nrow(m_r) && ncol(m) == ncol(m_r)) {
        cat("\n✓ 维度匹配！\n")
      } else {
        cat("\n✗ 维度不匹配:\n")
        cat("  Rust:", nrow(m), "x", ncol(m), "\n")
        cat("  R:", nrow(m_r), "x", ncol(m_r), "\n")
      }

      # 对比样本名称
      rust_samples <- colData(m)$sample_id
      r_samples <- colData(m_r)$sample_id

      cat("\n样本名称对比:\n")
      cat("  Rust 样本:", paste(rust_samples, collapse = ", "), "\n")
      cat("  R 样本:   ", paste(r_samples, collapse = ", "), "\n")

      if (all(sort(rust_samples) == sort(r_samples))) {
        cat("\n✓ 样本名称匹配！\n")
      } else {
        cat("\n✗ 样本名称不匹配\n")
      }

    }, error = function(e) {
      cat("无法加载 R 对象:", conditionMessage(e), "\n")
    })
  }

  # 检查 QC 报告
  if (file.exists(r_cov_xlsx)) {
    cat("\nR QC 报告:", r_cov_xlsx, "\n")
  }
} else {
  cat("R methrix 输出目录不存在:", r_output_dir, "\n")
}

cat("\n")

#===========================================================
# 测试 6: 功能测试
#===========================================================

cat(">>> 测试 6: methrix 核心功能...\n")
cat("==========================================\n")

# 测试覆盖度过滤
cat("测试覆盖度过滤 (cov_thr = 5, min_samples = 1):\n")
tryCatch({
  m_filtered <- coverage_filter(m, cov_thr = 5, min_samples = 1)
  cat("✓ 过滤后 CpG 数:", nrow(m_filtered), "\n")
  cat("  原始 CpG 数:", nrow(m), "\n")
}, error = function(e) {
  cat("✗ 覆盖度过滤失败:", conditionMessage(e), "\n")
})

cat("\n")

#===========================================================
# 保存验证结果
#===========================================================

cat("==========================================\n")
cat("验证完成!")
cat("==========================================\n")
cat("结束时间:", Sys.time(), "\n")
cat("HDF5 文件:", h5_file, "\n")
cat("输出目录:", output_dir, "\n")

# 保存结果到文件
result_file <- file.path(output_dir, "r_verification_results.txt")
sink(result_file)
cat("Methrix CLI R 兼容性验证结果\n")
cat("==========================================\n\n")
cat("验证时间:", Sys.time(), "\n\n")
cat("HDF5 文件:", h5_file, "\n\n")
cat("基本信息:\n")
cat("  样本数:", ncol(m), "\n")
cat("  CpG 位点数:", nrow(m), "\n")
cat("  基因组:", m@metadata$genome, "\n\n")

stats <- get_stats(m)
cat("统计信息:\n")
print(stats)
sink()

cat("\n验证结果已保存:", result_file, "\n")
cat("==========================================\n")
