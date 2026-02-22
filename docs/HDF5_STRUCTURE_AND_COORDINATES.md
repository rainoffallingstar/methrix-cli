# HDF5 文件结构与 CpG 坐标信息完整说明

## 📋 版本说明

**当前版本**: 使用标准化的数据集名称 `beta` 和 `cov`

**版本历史**:
- **v1.0 (旧)**: 使用 `assay001` 和 `assay002`
- **v2.0 (当前)**: 使用 `beta` 和 `cov` (与 R methrix 包命名一致)

## 📁 HDF5 文件结构

### 完整目录结构

```
assays.h5 (7.2 MB)
├── /beta              FLOAT [80028 x 2]  - beta 矩阵 (甲基化值)
├── /cov               INTEGER [80028 x 2] - coverage 矩阵 (覆盖度)
├── /colData/
│   └── sample_id      STRING [2]         - 样本 ID
├── /rowData/
│   ├── chr            STRING [80028]     - 染色体
│   ├── start          INTEGER [80028]     - 起始位置 (0-based)
│   ├── end            INTEGER [80028]     - 结束位置
│   └── strand         STRING [80028]     - 链 (+/-)
└── /metadata/
    ├── genome         INTEGER [4]        - 基因组名称 ("hg19")
    └── is_h5          ENUM [1]           - HDF5 格式标志
```

### 字段详细说明

| 字段路径 | 类型 | 维度 | 说明 | 示例值 |
|---------|------|------|------|--------|
| `/beta` | FLOAT | 80028 x 2 | 甲基化值矩阵 (CpG位点 x 样本) | 0.98, NA, 0.05 |
| `/cov` | INTEGER | 80028 x 2 | 覆盖度矩阵 (CpG位点 x 样本) | 51, 101, 0 |
| `/rowData/chr` | STRING | 80028 | 染色体名称 | "chr1", "chr2", ... |
| `/rowData/start` | INTEGER | 80028 | 起始位置 (0-based) | 133164, 133179, ... |
| `/rowData/end` | INTEGER | 80028 | 结束位置 | 133166, 133181, ... |
| `/rowData/strand` | STRING | 80028 | 链信息 | "+", "-" |
| `/colData/sample_id` | STRING | 2 | 样本 ID | "sample1.bismark.cov" |
| `/metadata/genome` | INTEGER | 4 | 基因组名称 (ASCII) | 104, 103, 49, 57 ("hg19") |
| `/metadata/is_h5` | ENUM | 1 | HDF5 格式标志 | TRUE |

## 📍 坐标系统详解

### 各阶段坐标系统

```
1. FASTA 文件 CpG 提取
   └─> 坐标系: 0-based
   └─> 示例: 序列 "ATCGATCGAA" 中
        - 索引: 0 1 2 3 4 5 6 7 8
        - 序列: A T C G A T C G A A
        - CG1: 索引 2-3 → CpGSite{chr: "chr1", start: 2, end: 4}
        - CG2: 索引 6-7 → CpGSite{chr: "chr1", start: 6, end: 8}

2. Bismark .cov 文件
   └─> 坐标系: 1-based (标准格式)
   └─> 示例: "chr1	3	4	0	10	CG"
        └─> 读取时转换: start = 3 - 1 = 2 (0-based)

3. 内部处理
   └─> 坐标系: 0-based
   └─> 所有数据对齐使用 0-based
        CpG位点: start=2
        Bismark记录: start=2 (已转换)

4. HDF5 存储
   └─> 坐标系: 0-based
   └─> /rowData/start = 2 (原样存储)

5. R 读取
   └─> 坐标系: 1-based (R 标准)
   └─> 转换: position = start + 1
        例如: HDF5 start=2 → R position=3
```

### 实际数据示例

从分析结果看，前10个CpG位点：

| Index | Chr | Start (0-based) | Start (1-based) | End | Strand | CpG_ID |
|-------|-----|-----------------|-----------------|-----|--------|---------|
| 1 | chr1 | 133164 | 133165 | 133166 | + | chr1:133165 |
| 2 | chr1 | 133179 | 133180 | 133181 | + | chr1:133180 |
| 3 | chr1 | 799120 | 799121 | 799122 | + | chr1:799121 |
| 4 | chr1 | 839434 | 839435 | 839436 | + | chr1:839435 |
| 5 | chr1 | 839507 | 839508 | 839509 | + | chr1:839508 |

**关键点**：
- HDF5 存储：`start = 133164` (0-based)
- R 读取后：`position = 133165` (1-based)
- 两者指向同一个CpG位点！

## 🔍 CpG ID 信息

### 当前状态

**❌ 没有专门的 `cpg_id` 字段**

**✅ 但有完整的坐标信息可以创建唯一 ID**

### 推荐的 CpG ID 生成方案

#### 方案 1: 简单行索引
```r
cpg_id <- 1:nrow(se)
# 结果: 1, 2, 3, ..., 80028
```
- ✅ 简单、唯一
- ❌ 不含位置信息

#### 方案 2: 染色体:坐标 (推荐) ⭐
```r
cpg_id <- paste(chr, start + 1, sep = ":")
# 结果: "chr1:133165", "chr1:133180", ...
```
- ✅ 包含位置信息
- ✅ 唯一性
- ✅ 标准格式
- ✅ 人类可读

#### 方案 3: 染色体:坐标:链 (最完整)
```r
cpg_id <- paste(chr, start + 1, strand, sep = ":")
# 结果: "chr1:133165:+", "chr1:133180:+", ...
```
- ✅ 信息最完整
- ✅ 可区分正负链

#### 方案 4: GenomicRanges (Bioconductor 标准)
```r
gr <- GRanges(chr, IRanges(start + 1, end), strand)
```
- ✅ R/Bioconductor 标准格式
- ✅ 自动处理坐标转换

## 💡 使用建议

### 在 R 中读取和创建 CpG ID

```r
library(rhdf5)
library(HDF5Array)
library(SummarizedExperiment)

# 读取 HDF5
h5_file <- "assays.h5"

# 方法 1: 直接读取 (全部加载到内存)
beta <- h5read(h5_file, "/beta")
cov <- h5read(h5_file, "/cov")

# 方法 2: 延迟加载 (推荐用于大数据)
beta_h5 <- HDF5Array(h5_file, "beta")
cov_h5 <- HDF5Array(h5_file, "cov")

# 读取坐标数据
chr <- h5read(h5_file, "/rowData/chr")
start <- h5read(h5_file, "/rowData/start")
end <- h5read(h5_file, "/rowData/end")
strand <- h5read(h5_file, "/rowData/strand")

# 创建 CpG ID (推荐方案2)
cpg_ids <- paste(chr, start + 1, sep = ":")

# 查看示例
head(cpg_ids)
# [1] "chr1:133165" "chr1:133180" "chr1:799121" "chr1:839435" ...

# 创建 SummarizedExperiment
library(GenomicRanges)
gr <- GRanges(chr, IRanges(start + 1, end), strand)
coldata <- DataFrame(sample_id = h5read(h5_file, "/colData/sample_id"))

se <- SummarizedExperiment(
  assays = list(beta = beta_h5, cov = cov_h5),
  rowRanges = gr,
  colData = coldata
)
rownames(se) <- cpg_ids
```

### 数据查询示例

```r
# 查找特定 CpG 位点
cpg_id <- "chr1:133165"

# 解析坐标
parts <- strsplit(cpg_id, ":")[[1]]
chr <- parts[1]           # "chr1"
pos <- as.integer(parts[2])  # 133165 (1-based)

# 在 HDF5 中查找对应索引
# (需要先减1转换为0-based)
start_0based <- pos - 1
index <- which(start == start_0based & chr == chr_data)
```

## ✅ 总结

### 文件结构

1. **assays 数据**: `/beta` (甲基化值), `/cov` (覆盖度)
2. **CpG 坐标**: `/rowData/chr`, `/rowData/start`, `/rowData/end`, `/rowData/strand`
3. **样本信息**: `/colData/sample_id`
4. **元数据**: `/metadata/genome`, `/metadata/is_h5`

### CpG ID

- **当前**: 无专门的 `cpg_id` 字段
- **推荐**: 使用 `chr:start+1` 格式作为唯一 ID
- **示例**: `"chr1:133165"`, `"chr1:839435"`

### 坐标一致性

**✅ 完全一致！**

所有阶段都使用 0-based 坐标：
- FASTA 提取 (0-based)
- 内部处理 (0-based)
- HDF5 存储 (0-based)

只有在与外部系统交互时才转换：
- Bismark 读取: `start = start_bismark - 1` (1→0-based)
- R 导出: `position = start + 1` (0→1-based)
