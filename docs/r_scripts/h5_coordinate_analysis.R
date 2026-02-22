#!/usr/bin/env Rscript
# 直接从 HDF5 分析结构

library(rhdf5)

h5_file <- "testdata/mCall/rust_output_20260222_112603_job36922017/assays.h5"

cat("==========================================\n")
cat("HDF5 文件结构与 CpG 坐标信息\n")
cat("==========================================\n\n")

cat("文件:", h5_file, "\n")
cat("大小:", format(file.info(h5_file)$size, units = "auto"), "\n\n")

# 完整结构
cat("文件结构:\n")
h5ls(h5_file, recursive = TRUE)

# 读取坐标数据
chr_data <- h5read(h5_file, "/rowData/chr")
start_data <- h5read(h5_file, "/rowData/start")
end_data <- h5read(h5_file, "/rowData/end")
strand_data <- h5read(h5_file, "/rowData/strand")

cat("\n\n==========================================\n")
cat("CpG 位点坐标信息\n")
cat("==========================================\n\n")

cat("位点总数:", length(chr_data), "\n")
cat("染色体:", paste(unique(chr_data), collapse = ", "), "\n")
cat("坐标范围: [", min(start_data), ", ", max(start_data), "]\n", sep = "")
cat("坐标系统: 0-based (HDF5内部)\n\n")

cat("前10个位点示例:\n")
cat("----------------------------------------\n")
cat("Index\tChr\tStart(0-based)\tStart(1-based)\tEnd\tStrand\tCpG_ID\n")
for (i in 1:min(10, length(chr_data))) {
  cpg_id <- paste0(chr_data[i], ":", start_data[i] + 1)
  cat(i, "\t", chr_data[i], "\t", start_data[i], "\t", start_data[i] + 1, "\t",
      end_data[i], "\t", strand_data[i], "\t", cpg_id, "\n")
}

cat("\n==========================================\n")
cat("CpG ID 生成方案\n")
cat("==========================================\n\n")

cat("当前 HDF5 结构:\n")
cat("  ❌ 没有专门的 cpg_id 字段\n")
cat("  ✓ 有 chr, start, end, strand 字段\n\n")

cat("推荐方案 - 创建唯一 CpG ID:\n")
cat("----------------------------------------\n\n")

cat("方案 1: 简单行索引\n")
cat("  cpg_id <- 1:nrow(se)\n")
cat("  结果: 1, 2, 3, ..., 80028\n")
cat("  优点: 简单、唯一\n")
cat("  缺点: 不含位置信息\n\n")

cat("方案 2: 染色体:坐标 (推荐) ⭐\n")
cat("  cpg_id <- paste(chr, start + 1, sep = \":\")\n")
cat("  结果: \"chr1:133165\", \"chr1:133180\", ...\n")
cat("  优点: 包含位置信息、唯一、标准格式\n")
cat("  示例: \"", paste(chr_data[1], start_data[1] + 1, sep = ":"), "\"\n\n")

cat("方案 3: 染色体:坐标:链 (最完整)\n")
cat("  cpg_id <- paste(chr, start + 1, strand, sep = \":\")\n")
cat("  结果: \"chr1:10469:+\", \"chr1:10470:+\", ...\n")
cat("  优点: 信息最完整\n")
cat("  示例: \"", paste(chr_data[1], start_data[1] + 1, strand_data[1], sep = ":"), "\"\n\n")

cat("方案 4: GenomicRanges 格式 (Bioconductor 标准)\n")
cat("  GRanges(chr, IRanges(start+1, end))\n")
cat("  优点: R/Bioconductor 标准格式\n")
cat("  说明: 自动处理坐标转换\n\n")

cat("==========================================\n")
cat("坐标系统总结\n")
cat("==========================================\n\n")

cat("1. FASTA 提取时:\n")
cat("   - 坐标系: 0-based\n")
cat("   - 示例: 序列位置 2 → CpGSite{start: 2, end: 4}\n\n")

cat("2. HDF5 存储时:\n")
cat("   - 坐标系: 0-based\n")
cat("   - 存储值: start = 2\n")
cat("   - 读取后: R 中需要 +1 → position = 3\n\n")

cat("3. 一致性: ✅ 完全一致\n")
cat("   - FASTA 提取 → 内部处理 → HDF5 存储: 全部 0-based\n")
cat("   - 只有在读取到 R 时才转换为 1-based\n\n")
