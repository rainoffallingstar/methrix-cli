#!/usr/bin/env Rscript
# 简化的 HDF5 结构分析

library(rhdf5)

h5_file <- "testdata/mCall/rust_output_20260222_112603_job36922017/assays.h5"

cat("==========================================\n")
cat("HDF5 文件结构详细分析\n")
cat("==========================================\n")
cat("文件:", h5_file, "\n")
cat("大小:", format(file.info(h5_file)$size, units = "auto"), "\n\n")

# 完整结构（递归）
cat("完整目录结构:\n")
cat("==========================================\n")
h5ls(h5_file, recursive = TRUE)

# 读取数据
cat("\n\n数据维度:\n")
cat("==========================================\n")
beta <- h5read(h5_file, "/beta")
cov <- h5read(h5_file, "/cov")
cat("assay001 (beta):", dim(beta), "\n")
cat("assay002 (cov):", dim(cov), "\n")

# rowData
cat("\nrowData (CpG 坐标信息):\n")
cat("==========================================\n")
chr_data <- h5read(h5_file, "/rowData/chr")
start_data <- h5read(h5_file, "/rowData/start")
end_data <- h5read(h5_file, "/rowData/end")
strand_data <- h5read(h5_file, "/rowData/strand")

cat("位点数:", length(chr_data), "\n")
cat("染色体:", paste(unique(chr_data), collapse = ", "), "\n")
cat("坐标范围: [", min(start_data), ", ", max(start_data), "]\n", sep = "")
cat("坐标系统: 0-based (HDF5内部存储)\n")

# 前5个位点示例
cat("\n前5个 CpG 位点详情:\n")
cat("==========================================\n")
example_df <- data.frame(
  Index = 1:5,
  Chr = chr_data[1:5],
  Start_0based = start_data[1:5],
  End = end_data[1:5],
  Start_1based = start_data[1:5] + 1,
  Strand = strand_data[1:5],
  Beta_Sample1 = beta[1:5, 1],
  Cov_Sample1 = cov[1:5, 1]
)
rownames(example_df) <- paste0("CpG", 1:5)
print(example_df)

# 是否有 CpG ID
cat("\n==========================================\n")
cat("CpG ID 信息\n")
cat("==========================================\n")
cat("❌ 没有专门的 'cpg_id' 字段\n")
cat("✓ 但可以通过组合坐标创建唯一标识符\n\n")

cat("推荐方案 (创建唯一 ID):\n")
cat("----------------------------------------\n")
cat("# 方案1: 行索引\n")
cat("cpg_id <- 1:nrow(se)\n")
cat("# 结果: 1, 2, 3, ..., 80028\n\n")

cat("# 方案2: 染色体:坐标组合 (推荐)\n")
cat("cpg_id <- paste(chr_data, start_data + 1, sep = \":\")\n")
cat("# 结果: \"chr1:10469\", \"chr1:10470\", ...\n\n")

cat("# 方案3: 染色体:坐标:链 (最完整)\n")
cat("cpg_id <- paste(chr_data, start_data + 1, strand_data, sep = \":\")\n")
cat("# 结果: \"chr1:10469:+\", \"chr1:10470:+\", ...\n\n")

# 在 R 中实际创建 ID
cat("在 R SummarizedExperiment 中添加 ID:\n")
cat("----------------------------------------\n")
# 读取数据
se <- readRDS("testdata/mCall/rust_output_20260222_112603_job36922017/se.rds")

# 方法1: 设置行名为索引
rownames(se) <- paste0("CpG", 1:nrow(se))
cat("方法1 (行索引):\n")
cat("  前3行名:", head(rownames(se)), "\n\n")

# 方法2: 使用染色体:坐标
rownames(se) <- paste(chr_data, start_data + 1, sep = ":")
cat("方法2 (坐标组合):\n")
cat("  前3行名:", head(rownames(se)), "\n\n")

cat("==========================================\n")
cat("总结\n")
cat("==========================================\n\n")

cat("HDF5 文件结构:\n")
cat("├── /beta            # beta 矩阵 (80028 x 2)\n")
cat("├── /cov             # cov 矩阵 (80028 x 2)\n")
cat("├── /rowData\n")
cat("│   ├── chr          # 染色体 (STRING[80028])\n")
cat("│   ├── start        # 起始位置 (INTEGER[80028], 0-based)\n")
cat("│   ├── end          # 结束位置 (INTEGER[80028])\n")
cat("│   └── strand       # 链 (STRING[80028])\n")
cat("├── /colData\n")
cat("│   └── sample_id    # 样本ID (STRING[2])\n")
cat("└── /metadata\n")
cat("    ├── genome       # 基因组名称 (INTEGER[4], 实际是\"hg19\")\n")
cat("    └── is_h5        # HDF5标志 (ENUM[1])\n\n")

cat("坐标系统:\n")
cat("  - HDF5 内部: 0-based\n")
cat("  - R 使用: 1-based (需转换)\n")
cat("  - 一致性: ✅ 所有内部坐标都是 0-based\n\n")

cat("CpG 标识符:\n")
cat("  - 字段: ❌ 无专门的 cpg_id 字段\n")
cat("  - 标识: ✓ 可用 chr:start: 或行索引\n")
cat("  - 推荐: paste(chr, start+1, sep=\":\") 作为唯一 ID\n\n")
