#!/usr/bin/env Rscript
# 验证 HDF5 文件是否使用新的 beta/cov 命名

library(rhdf5)

# 命令行参数
args <- commandArgs(trailingOnly = TRUE)
if (length(args) == 0) {
  h5_file <- "testdata/mCall/rust_output_20260222_112603_job36922017/assays.h5"
} else {
  h5_file <- args[1]
}

cat("==========================================\n")
cat("HDF5 文件结构验证\n")
cat("==========================================\n\n")

if (!file.exists(h5_file)) {
  cat("错误: 文件不存在:", h5_file, "\n")
  quit(status = 1)
}

cat("文件:", h5_file, "\n")
cat("大小:", format(file.info(h5_file)$size, units = "auto"), "\n\n")

# 读取文件结构
structure <- h5ls(h5_file, recursive = TRUE)
cat("文件结构:\n")
cat("----------------------------------------\n")
print(structure)
cat("\n")

# 检查数据集名称
dataset_names <- structure$name
cat("检测到的数据集:\n")
cat("----------------------------------------\n")
print(dataset_names)
cat("\n")

# 验证命名
cat("命名验证:\n")
cat("----------------------------------------\n")

if ("beta" %in% dataset_names) {
  cat("✓ 使用新命名 (v2.0)\n")
  cat("  - beta: 存在\n")
  cat("  - cov: ", ifelse("cov" %in% dataset_names, "存在 ✓", "不存在 ✗"), "\n")

  # 检查是否还有旧名称
  if ("assay001" %in% dataset_names || "assay002" %in% dataset_names) {
    cat("⚠ 警告: 同时检测到旧命名 (assay001/assay002)\n")
  }
} else if ("assay001" %in% dataset_names) {
  cat("✗ 使用旧命名 (v1.0)\n")
  cat("  - assay001: 存在\n")
  cat("  - assay002: ", ifelse("assay002" %in% dataset_names, "存在", "不存在"), "\n")
  cat("建议: 更新到新版本的 methx\n")
} else {
  cat("✗ 未知的命名格式\n")
  cat("无法检测到 beta/cov 或 assay001/assay002\n")
  quit(status = 1)
}

cat("\n")

# 尝试读取数据
cat("数据读取测试:\n")
cat("----------------------------------------\n")

if ("beta" %in% dataset_names) {
  tryCatch({
    beta <- h5read(h5_file, "/beta")
    cov <- h5read(h5_file, "/cov")

    cat("✓ 成功读取 beta 矩阵\n")
    cat("  维度:", dim(beta), "\n")
    cat("  类型:", typeof(beta), "\n")
    cat("  前3个值:", head(as.vector(beta), 3), "\n\n")

    cat("✓ 成功读取 cov 矩阵\n")
    cat("  维度:", dim(cov), "\n")
    cat("  类型:", typeof(cov), "\n")
    cat("  前3个值:", head(as.vector(cov), 3), "\n\n")
  }, error = function(e) {
    cat("✗ 读取数据时出错:", e$message, "\n")
  })
}

# 检查必需的组和数据集
cat("结构完整性检查:\n")
cat("----------------------------------------\n")

required_groups <- c("rowData", "colData", "metadata")
required_datasets <- c(
  "rowData/chr", "rowData/start", "rowData/end", "rowData/strand",
  "colData/sample_id",
  "metadata/genome", "metadata/is_h5"
)

all_paths <- paste(structure$group, structure$name, sep = "/")
all_paths <- gsub("^/", "", all_paths)

for (group in required_groups) {
  if (group %in% structure$group) {
    cat("✓ 组", group, ": 存在\n")
  } else {
    cat("✗ 组", group, ": 不存在\n")
  }
}

for (dataset in required_datasets) {
  if (dataset %in% all_paths) {
    cat("✓ 数据集", dataset, ": 存在\n")
  } else {
    cat("✗ 数据集", dataset, ": 不存在\n")
  }
}

cat("\n==========================================\n")
cat("验证完成\n")
cat("==========================================\n")
