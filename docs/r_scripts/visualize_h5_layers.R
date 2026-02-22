#!/usr/bin/env Rscript
# 可视化 HDF5 文件的层级结构

library(rhdf5)

h5_file <- "testdata/mCall/rust_output_20260222_112603_job36922017/assays.h5"

cat("==========================================\n")
cat("HDF5 文件的层级结构\n")
cat("==========================================\n\n")

cat("📁 assays.h5 (根)\n")
cat("│\n")
cat("├── 📊 📁 assays/ (数据层 - 实际上没有这个组，数据直接在根下)\n")
cat("│   ├── 📄 beta     [FLOAT: 80028 x 2]  <- beta 甲基化矩阵\n")
cat("│   └── 📄 cov      [INTEGER: 80028 x 2] <- coverage 覆盖度矩阵\n")
cat("│\n")
cat("├── 📍 📁 rowData/ (行数据 - CpG位点信息)\n")
cat("│   ├── 📄 chr       [STRING: 80028]  <- 染色体名称\n")
cat("│   ├── 📄 start     [INTEGER: 80028]  <- 起始位置 (0-based)\n")
cat("│   ├── 📄 end       [INTEGER: 80028]  <- 结束位置\n")
cat("│   └── 📄 strand    [STRING: 80028]  <- 链方向 (+/-)\n")
cat("│\n")
cat("├── 🧪 📁 colData/ (列数据 - 样本信息)\n")
cat("│   └── 📄 sample_id [STRING: 2]  <- 样本ID\n")
cat("│\n")
cat("└── ℹ️  📁 metadata/ (元数据)\n")
cat("    ├── 📄 genome    [INTEGER: 4]     <- 基因组 \"hg19\"\n")
cat("    └── 📄 is_h5     [ENUM: 1]        <- HDF5格式标志\n")

cat("\n==========================================\n")
cat("关键点说明\n")
cat("==========================================\n\n")

cat("1. 这是单个 HDF5 文件 (assays.h5)\n")
cat("   大小: ~7.2 MB\n")
cat("   包含所有数据: 矩阵 + 坐标 + 样本 + 元数据\n\n")

cat("2. 采用 GROUP (组) 层级结构\n")
cat("   - rowData/: 一个组，包含4个数据集\n")
cat("   - colData/: 一个组，包含1个数据集\n")
cat("   - metadata/: 一个组，包含2个数据集\n")
cat("   - assay001/assay002: 直接在根级别，不在组内\n\n")

cat("3. 数据集 (Dataset) vs 组 (Group)\n")
cat("   - Dataset: 实际的数据矩阵 (如 assay001)\n")
cat("   - Group: 逻辑容器，包含其他数据集 (如 rowData)\n\n")

cat("4. 为什么 beta/cov 在根级别？\n")
cat("   - 这是 R methrix 的 HDF5 格式要求\n")
cat("   - 与 assays 组内的 rowData/colData 并列存储\n")
cat("   - 便于 R 包直接读取\n\n")

cat("5. 坐标信息存储在 rowData 组中\n")
cat("   - ✅ 有基因组坐标 (chr, start, end, strand)\n")
cat("   - ❌ 没有专门的 'cpg_id' 字段\n")
cat("   - 💡 可用 'chr:start+1' 作为唯一 ID\n\n")

cat("6. 维度说明\n")
cat("   - 80028: 过滤后的 CpG 位点数\n")
cat("   - 2: 样本数\n")
cat("   - 所有矩阵维度一致: 80028 行 x 2 列\n\n")

cat("==========================================\n")
