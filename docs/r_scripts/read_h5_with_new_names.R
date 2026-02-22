#!/usr/bin/env Rscript
# 演示如何读取使用新名称 (beta/cov) 的 HDF5 文件

library(rhdf5)

h5_file <- "testdata/mCall/rust_output_20260222_112603_job36922017/assays.h5"

cat("==========================================\n")
cat("读取使用新名称的 HDF5 文件\n")
cat("==========================================\n\n")

cat("文件:", h5_file, "\n")
cat("大小:", format(file.info(h5_file)$size, units = "auto"), "\n\n")

# 1. 查看文件结构
cat("1. 文件结构:\n")
cat("----------------------------------------\n")
structure <- h5ls(h5_file, recursive = TRUE)
print(structure)
cat("\n")

# 2. 使用新名称读取数据
cat("2. 使用新名称读取数据:\n")
cat("----------------------------------------\n")

# 读取 beta 矩阵
beta <- h5read(h5_file, "/beta")
cat("beta 矩阵维度:", dim(beta), "\n")
cat("前3行前2列:\n")
print(beta[1:3, 1:2])
cat("\n")

# 读取 cov 矩阵
cov <- h5read(h5_file, "/cov")
cat("cov 矩阵维度:", dim(cov), "\n")
cat("前3行前2列:\n")
print(cov[1:3, 1:2])
cat("\n")

# 读取 rowData
chr_data <- h5read(h5_file, "/rowData/chr")
start_data <- h5read(h5_file, "/rowData/start")
end_data <- h5read(h5_file, "/rowData/end")
strand_data <- h5read(h5_file, "/rowData/strand")

cat("CpG 位点信息:\n")
cat("  位点数:", length(chr_data), "\n")
cat("  染色体:", paste(unique(chr_data), collapse = ", "), "\n")
cat("  坐标范围:", min(start_data), "-", max(start_data), "\n\n")

# 读取 colData
sample_names <- h5read(h5_file, "/colData/sample_id")
cat("样本信息:\n")
cat("  样本数:", length(sample_names), "\n")
cat("  样本名:", paste(sample_names, collapse = ", "), "\n\n")

# 3. 创建 SummarizedExperiment 对象
cat("3. 创建 SummarizedExperiment 对象:\n")
cat("----------------------------------------\n")

library(SummarizedExperiment)

# 创建 GRanges 对象用于 rowData
library(GenomicRanges)
gr <- GRanges(
  seqnames = chr_data,
  ranges = IRanges(start = start_data + 1, end = end_data),  # 转换为 1-based
  strand = strand_data
)

# 创建 DataFrame 用于 colData
library(S4Vectors)
coldata <- DataFrame(sample_id = sample_names)

# 创建 assays 列表
assays_list <- list(
  beta = beta,
  cov = cov
)

# 创建 SummarizedExperiment
se <- SummarizedExperiment(
  assays = assays_list,
  rowRanges = gr,
  colData = coldata
)

cat("创建的 SummarizedExperiment 对象:\n")
print(se)
cat("\n")

cat("assay 名称:", assayNames(se), "\n")
cat("维度:", dim(se), "\n\n")

# 4. 使用 HDF5Array 进行延迟加载 (推荐)
cat("4. 使用 HDF5Array 延迟加载 (推荐):\n")
cat("----------------------------------------\n")

library(HDF5Array)

# 创建 HDF5Matrix 对象 (不立即加载到内存)
beta_h5 <- HDF5Array(h5_file, "beta")
cov_h5 <- HDF5Array(h5_file, "cov")

cat("HDF5Matrix 对象:\n")
cat("  beta:", class(beta_h5), "\n")
cat("  维度:", dim(beta_h5), "\n")
cat("  延迟加载: TRUE (不会立即加载到内存)\n\n")

# 使用 HDF5Matrix 创建 SE
se_h5 <- SummarizedExperiment(
  assays = list(beta = beta_h5, cov = cov_h5),
  rowRanges = gr,
  colData = coldata
)

cat("使用 HDF5Matrix 的 SummarizedExperiment:\n")
print(se_h5)
cat("\n")

# 5. 数据访问示例
cat("5. 数据访问示例:\n")
cat("----------------------------------------\n")

# 访问特定位置的甲基化值
cat("第1个CpG位点在样本1的甲基化值:\n")
cat("  坐标:", as.character(gr[1]), "\n")
cat("  beta值:", assay(se, "beta")[1, 1], "\n")
cat("  覆盖度:", assay(se, "cov")[1, 1], "\n\n")

# 访问特定染色体的数据
cat("chr1 上的前3个位点:\n")
chr1_idx <- which(seqnames(gr) == "chr1")[1:3]
print(assay(se, "beta")[chr1_idx, ])
cat("\n")

# 6. 与 methrix 包的兼容性
cat("6. 与 methrix 包的兼容性:\n")
cat("----------------------------------------\n")

cat("✅ SummarizedExperiment 对象与 methrix 包兼容\n")
cat("✅ assay 名称为 'beta' 和 'cov'\n")
cat("✅ 可以使用 methrix 的大部分分析函数\n\n")

cat("示例操作:\n")
cat("  # 获取甲基化值\n")
cat("  beta_values <- assay(se, \"beta\")\n\n")
cat("  # 获取覆盖度\n")
cat("  coverage <- assay(se, \"cov\")\n\n")
cat("  # 筛选高覆盖度位点\n")
cat("  high_cov <- rowMeans(assay(se, \"cov\")) > 10\n")
cat("  se_filtered <- se[high_cov, ]\n\n")

cat("==========================================\n")
cat("总结\n")
cat("==========================================\n\n")

cat("✓ HDF5 文件使用 'beta' 和 'cov' 作为数据集名称\n")
cat("✓ 与 R methrix 包的命名约定一致\n")
cat("✓ 可以使用 h5read() 或 HDF5Array 读取\n")
cat("✓ 推荐使用 HDF5Array 进行延迟加载以节省内存\n")
cat("✓ 创建的 SummarizedExperiment 对象与 methrix 兼容\n\n")

cat("数据集名称对照:\n")
cat("  旧名称: assay001 → 新名称: beta\n")
cat("  旧名称: assay002 → 新名称: cov\n")
