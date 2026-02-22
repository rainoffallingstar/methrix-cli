#!/usr/bin/env Rscript

library(methrix)

output_dir <- "testdata/mCall/rust_output_20260222_112603_job36922017"

cat("测试 load_HDF5_methrix() 加载 Rust 生成的数据\n")
cat("==========================================\n")
cat("目录:", output_dir, "\n\n")

cat("方法1: 使用 load_HDF5_methrix (期望失败，因为需要特殊格式)...\n")
tryCatch({
  m <- methrix::load_HDF5_methrix(output_dir)
  cat("✓ 成功加载!\n")
  cat("  维度:", nrow(m), "x", ncol(m), "\n")
}, error = function(e) {
  cat("✗ 失败:", conditionMessage(e), "\n")
})

cat("\n方法2: 直接读取 se.rds...\n")
se_rds <- file.path(output_dir, "se.rds")
if (file.exists(se_rds)) {
  m <- readRDS(se_rds)
  cat("✓ se.rds 加载成功\n")
  cat("  类型:", class(m)[1], "\n")
  cat("  维度:", nrow(m), "x", ncol(m), "\n")
  cat("  genome:", metadata(m)$genome, "\n")
  cat("  is_h5:", metadata(m)$is_h5, "\n")
  cat("  assays:", names(assays(m)), "\n")

  cat("\n转换为 methrix 对象...\n")
  m2 <- methrix::as_methrix(m)
  cat("✓ 转换成功\n")
  cat("  类型:", class(m2)[1], "\n")

  cat("\n测试 methrix 功能...\n")
  stats <- methrix::get_stats(m2)
  cat("✓ get_stats 成功\n")
  print(head(stats, 3))
}

cat("\n==========================================\n")
cat("验证完成！\n")
cat("==========================================\n")
