#!/usr/bin/env Rscript
# analyze_h5_structure.R
# 详细分析 HDF5 文件的完整结构

library(rhdf5)

h5_file <- "testdata/mCall/rust_output_20260222_112603_job36922017/assays.h5"

cat("==========================================\n")
cat("HDF5 文件完整结构分析\n")
cat("==========================================\n")
cat("文件:", h5_file, "\n")
cat("大小:", format(file.info(h5_file)$size, units = "auto"), "\n\n")

# 1. 顶层结构
cat("1. 顶层结构\n")
cat("==========================================\n")
top_level <- h5ls(h5_file)
print(top_level)

# 2. assays 数据
cat("\n2. assays 数据\n")
cat("==========================================\n")
cat("/assay001 (beta 矩阵):\n")
beta_dims <- h5read(h5_file, "assay001") %>% dim()
cat("  维度:", paste(beta_dims, collapse = " x "), "\n")
cat("  说明: 甲基化值矩阵 (CpG位点 x 样本)\n")

cat("\n/assay002 (coverage 矩阵):\n")
cov_dims <- h5read(h5_file, "assay002") %>% dim()
cat("  维度:", paste(cov_dims, collapse = " x "), "\n")
cat("  说明: 覆盖度矩阵 (CpG位点 x 样本)\n")

# 3. rowData (CpG 位点信息)
cat("\n3. rowData (CpG 位点坐标信息)\n")
cat("==========================================\n")
cat("维度: 80028 个 CpG 位点\n\n")

chr_data <- h5read(h5_file, "/rowData/chr")
start_data <- h5read(h5_file, "/rowData/start")
end_data <- h5read(h5_file, "/rowData/end")
strand_data <- h5read(h5_file, "/rowData/strand")

cat("字段详情:\n")
cat("  /rowData/chr:\n")
cat("    类型: 字符串数组\n")
cat("    长度:", length(chr_data), "\n")
cat("    唯一值:", unique(chr_data), "\n")
cat("    示例:", head(chr_data, 5), "\n")

cat("\n  /rowData/start:\n")
cat("    类型: 整数数组\n")
cat("    长度:", length(start_data), "\n")
cat("    范围: [", min(start_data), ", ", max(start_data), "]\n", sep = "")
cat("    坐标系: 0-based (HDF5内部)\n")
cat("    示例:", head(start_data, 5), "\n")

cat("\n  /rowData/end:\n")
cat("    类型: 整数数组\n")
cat("    长度:", length(end_data), "\n")
cat("    范围: [", min(end_data), ", ", max(end_data), "]\n", sep = "")
cat("    说明: CpG 位点结束位置\n")
cat("    示例:", head(end_data, 5), "\n")

cat("\n  /rowData/strand:\n")
cat("    类型: 字符串数组\n")
cat("    长度:", length(strand_data), "\n")
cat("    唯一值:", unique(strand_data), "\n")
cat("    示例:", head(strand_data, 5), "\n")

# 4. colData (样本信息)
cat("\n4. colData (样本信息)\n")
cat("==========================================\n")
sample_ids <- h5read(h5_file, "/colData/sample_id")
cat("样本数:", length(sample_ids), "\n")
cat("样本ID:\n")
for (i in seq_along(sample_ids)) {
  cat("  样本", i, ":", sample_ids[i], "\n")
}

# 5. metadata (元数据)
cat("\n5. metadata (元数据)\n")
cat("==========================================\n")
genome_raw <- h5read(h5_file, "/metadata/genome")
cat("/metadata/genome:\n")
cat("  原始值:", genome_raw, "\n")
cat("  解码后:", intToUtf8(genome_raw), "\n")
cat("  说明: 参考基因组名称\n")

cat("\n/metadata/is_h5:\n")
is_h5 <- h5read(h5_file, "/metadata/is_h5")
cat("  值:", is_h5, "\n")
cat("  说明: 是否为 HDF5 格式\n")

# 6. 数据示例
cat("\n6. 数据示例 (前5个 CpG 位点)\n")
cat("==========================================\n")
example_df <- data.frame(
  chr = chr_data[1:5],
  start_0based = start_data[1:5],
  end = end_data[1:5],
  start_1based = start_data[1:5] + 1,  # 转换为1-based
  strand = strand_data[1:5],
  beta_sample1 = h5read(h5_file, "assay001")[1:5, 1],
  cov_sample1 = h5read(h5_file, "assay002")[1:5, 1]
)
rownames(example_df) <- paste0("CpG", 1:5)
print(example_df)

# 7. 坐标系说明
cat("\n7. 坐标系说明\n")
cat("==========================================\n")
cat("HDF5 中存储的坐标: 0-based\n")
cat("  - /rowData/start: 0-based 起始位置\n")
cat("  - /rowData/end: 结束位置 (excluded)\n\n")

cat("R 中的坐标: 1-based\n")
cat("  - 转换: start_1based = start_0based + 1\n")
cat("  - 例如: start=2 (0-based) → position=3 (1-based)\n\n")

cat("Bismark 文件格式: 1-based\n")
cat("  - 输入时转换: start_internal = start_bismark - 1\n")
cat("  - 输出时转换: start_bismark = start_internal + 1\n\n")

cat("完整流程:\n")
cat("  FASTA 提取 (0-based)\n")
cat("  → Bismark 文件 (1-based)\n")
cat("  → 读取转换 (0-based)\n")
cat("  → HDF5 存储 (0-based)\n")
cat("  → R 读取 (1-based, 需转换)\n")

# 8. 是否有 CpG ID？
cat("\n8. CpG ID 信息\n")
cat("==========================================\n")
cat("❌ 没有单独的 CpG ID 字段\n")
cat("✓ 但可以通过以下方式标识 CpG 位点:\n\n")

cat("方法 1: 使用染色体 + 坐标组合\n")
cat("  示例: \"chr1:10468\" (chromosome:start)\n\n")

cat("方法 2: 使用行索引\n")
cat("  HDF5 中行索引从 0 开始\n")
cat("  示例: 第100个 CpG → 索引 99\n\n")

cat("方法 3: 在 R 中创建唯一 ID\n")
cat("  rownames(se) <- paste0(\"CpG\", 1:nrow(se))\n")
cat("  或\n")
cat("  rownames(se) <- paste0(chr, \":\", start + 1)\n\n")

cat("推荐方案:\n")
cat("  rownames(se) <- paste(chr_data, start_data + 1, sep = \":\")\n")
cat("  结果示例: \"chr1:10469\", \"chr1:10470\", ...\n")

cat("\n==========================================\n")
cat("分析完成\n")
cat("==========================================\n")
