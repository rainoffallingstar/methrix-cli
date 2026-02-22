#!/usr/bin/env Rscript

# 分析 methrix 类的内部结构，找出如何直接构造

library(methrix)
library(rhdf5)
library(methods)

# 创建一个简单的 methrix 对象用于分析
cat("创建测试用的 methrix 对象...\n")

# 创建临时数据
temp_beta <- matrix(c(0.5, 0.6, NA, 0.8), nrow = 2)
temp_cov <- matrix(c(10, 20, 0, 30), nrow = 2)

library(SummarizedExperiment)
library(GenomicRanges)

test_se <- SummarizedExperiment(
  assays = list(beta = temp_beta, cov = temp_cov),
  rowRanges = GRanges("chr1", IRanges(start = 1:2, end = 2:3), strand = "+"),
  colData = DataFrame(sample_id = c("s1", "s2")),
  metadata = list(genome = "hg19", is_h5 = FALSE)
)

cat("✓ 测试 SE 创建成功\n\n")

# 尝试转换为 methrix
cat("尝试转换为 methrix 对象...\n")

# 方法1: 检查 methrix 的 slot 结构
cat("\n=== methrix 类的 slots ===\n")
slot_names <- slotNames("methrix")
print(slot_names)

# 方法2: 尝试使用 as()
cat("\n=== 尝试 as() 方法 ===\n")
tryCatch({
  m <- as(test_se, "methrix")
  cat("✓ as() 方法成功!\n")
  cat("  类型:", class(m)[1], "\n")
}, error = function(e) {
  cat("✗ as() 失败:", conditionMessage(e), "\n")
})

# 方法3: 检查是否有转换函数
cat("\n=== 检查转换函数 ===\n")
methrix_funcs <- ls("package:methrix")
convert_funcs <- grep("convert|as_|to_", methrix_funcs, value = TRUE)
print(convert_funcs)

# 方法4: 查看某个简单 methrix 对象的结构
cat("\n=== 分析现有 methrix 对象 ===\n")
# 如果 R methrix 输出存在
r_se <- "testdata/mCall/methrixh5/se.rds"
if (file.exists(r_se)) {
  m_r <- readRDS(r_se)
  cat("✓ 加载 R methrix 对象成功\n")
  cat("  类型:", class(m_r), "\n")
  cat("  Slots:", slotNames(m_r), "\n")

  # 查看每个 slot 的内容
  for (sn in slotNames(m_r)) {
    slot_content <- slot(m_r, sn)
    cat("  Slot '", sn, "': ", class(slot_content)[1], "\n", sep = "")
  }

  # 检查是否可以直接复制结构
  cat("\n=== 尝试复制结构 ===\n")

  # 创建一个新的 methrix 对象，使用相同的结构
  new_m <- m_r
  # 替换 assays
  assays(new_m) <- assays(test_se)
  # 替换 rowData
  rowData(new_m) <- rowData(test_se)
  # 替换 colData
  colData(new_m) <- colData(test_se)

  cat("✓ 结构复制成功!\n")
  cat("  类型:", class(new_m)[1], "\n")
  cat("  维度:", nrow(new_m), "x", ncol(new_m), "\n")
} else {
  cat("R methrix 对象不存在，跳过\n")
}

# 方法5: 检查 methrix 包的内部函数
cat("\n=== 检查内部函数 ===\n")
all_funcs <- ls("package:methrix", all.names = TRUE)
internal_funcs <- grep("^\\.", all_funcs, value = TRUE)
print(head(internal_funcs, 20))

cat("\n=== 完成 ===\n")
