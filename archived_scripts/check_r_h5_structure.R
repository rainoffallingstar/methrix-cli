#!/usr/bin/env Rscript

# 检查 R methrix 生成的 H5 文件结构

library(rhdf5)
library(methrix)

r_h5_dir <- "testdata/mCall/methrixh5"

if (!dir.exists(r_h5_dir)) {
  cat("R methrix 输出目录不存在:", r_h5_dir, "\n")
  quit(status = 1)
}

cat("检查 R methrix 输出目录:", r_h5_dir, "\n")
cat("==========================================\n\n")

cat("目录内容:\n")
print(list.files(r_h5_dir))

cat("\n")

# 检查是否有 .assays 文件
assays_files <- list.files(r_h5_dir, pattern = "\\.assays$", full.names = TRUE)
if (length(assays_files) > 0) {
  cat("发现 .assays 文件:\n")
  for (f in assays_files) {
    cat("  ", f, "\n")
    h5ls(f)
  }
}

# 检查 se.rds
se_file <- file.path(r_h5_dir, "se.rds")
if (file.exists(se_file)) {
  cat("\nse.rds 文件存在\n")
  se <- readRDS(se_file)
  cat("类型:", class(se)[1], "\n")
  cat("维度:", nrow(se), "x", ncol(se), "\n")
}
