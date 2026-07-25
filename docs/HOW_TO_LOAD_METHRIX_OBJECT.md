# 如何从 Rust 生成的 HDF5 文件创建 methrix 对象

## 📚 概述

Rust methx 生成的 HDF5 文件可以通过多种方式在 R 中加载和使用。本文档提供了完整的方法指南。

## 方法对比

| 方法 | 难度 | 灵活性 | methrix兼容 | 推荐度 |
|------|------|--------|-------------|--------|
| 1. 直接读取HDF5 | ⭐ | ⭐⭐⭐⭐⭐ | ❌ | ⭐⭐⭐⭐⭐ |
| 2. 使用se.rds | ⭐ | ⭐⭐⭐⭐ | ⚠️ | ⭐⭐⭐⭐⭐ |
| 3. 转换为bedgraph | ⭐⭐⭐ | ⭐⭐⭐ | ✅ | ⭐⭐ |
| 4. 手动创建methrix | ⭐⭐⭐⭐ | ⭐⭐ | ✅ | ⭐⭐⭐ |

## 方法 1: 直接读取 HDF5（最推荐）✨

### 优点
- 最简单直接
- 完全控制数据
- 无需中间文件
- 内存效率高

### 使用方法

```r
library(rhdf5)

# 读取 HDF5 文件
h5_file <- "output/assays.h5"

# 读取 assays
beta <- h5read(h5_file, "/assay001")
cov <- h5read(h5_file, "/assay002")

# 读取 rowData
chr <- h5read(h5_file, "/rowData/chr")
start <- h5read(h5_file, "/rowData/start")
end <- h5read(h5_file, "/rowData/end")
strand <- h5read(h5_file, "/rowData/strand")

# 读取 colData
sample_ids <- h5read(h5_file, "/colData/sample_id")

# 读取 metadata
genome_raw <- h5read(h5_file, "/metadata/genome")
genome <- intToUtf8(genome_raw)  # 转换为字符串

# 现在可以使用数据了
dim(beta)  # 80028 x 2
dim(cov)   # 80028 x 2
```

### 数据分析示例

```r
# 基本统计
mean_beta <- colMeans(beta, na.rm = TRUE)
mean_cov <- colMeans(cov)

# 覆盖度过滤
covered_sites <- rowSums(cov > 0) >= 1  # 至少一个样本覆盖
beta_filtered <- beta[covered_sites, ]
cov_filtered <- cov[covered_sites, ]

# 提取特定区域
chr1_sites <- chr == "chr1"
beta_chr1 <- beta[chr1_sites, ]

# 差异甲基化分析
# 比较两个样本
diff <- beta[, 1] - beta[, 2]
top_diff <- order(abs(diff), decreasing = TRUE)[1:100]
```

## 方法 2: 使用 se.rds 文件

### 生成 se.rds

```r
# 使用提供的脚本
source("load_h5_as_methrix.R")
se <- load_h5_as_methrix("output/")

# 或手动创建
library(SummarizedExperiment)
library(GenomicRanges)

se <- SummarizedExperiment(
  assays = list(beta = beta, cov = cov),
  rowRanges = GRanges(
    seqnames = chr,
    ranges = IRanges(start = start + 1, end = end),
    strand = strand
  ),
  colData = DataFrame(sample_id = sample_ids),
  metadata = list(genome = genome, is_h5 = FALSE)
)

# 保存
saveRDS(se, "output/se.rds")
```

### 加载和使用

```r
# 加载
se <- readRDS("output/se.rds")

# 访问数据
assays(se)           # beta 和 cov
rowData(se)          # CpG 位点
colData(se)          # 样本信息
metadata(se)         # 元数据

# 子集操作
se[, 1]              # 第一个样本
se[1:1000, ]         # 前1000个位点
se[seqnames(se) == "chr1", ]  # chr1 上的位点
```

## 方法 3: 转换为 methrix 对象

### 为什么难创建 methrix 对象？

methrix 是一个 S4 类，其构造函数 `new("methrix", ...)` 不是导出的，无法直接调用。

### 解决方案：使用 bedgraph 文件

```r
library(methrix)

# 步骤1: 将 HDF5 数据写入 bedgraph 文件
# 为每个样本创建两个文件：*_beta.bdg 和 *_cov.bdg

# 示例：创建第一个样本的 bedgraph
sample1_beta <- data.frame(
  chr = chr,
  start = start,
  end = end,
  beta = beta[, 1]
)
sample1_beta <- sample1_beta[!is.na(sample1_beta$beta), ]
write.table(sample1_beta, "sample1_beta.bdg",
            sep = "\t", row.names = FALSE, col.names = FALSE, quote = FALSE)

sample1_cov <- data.frame(
  chr = chr,
  start = start,
  end = end,
  coverage = cov[, 1]
)
sample1_cov <- sample1_cov[sample1_cov$coverage > 0, ]
write.table(sample1_cov, "sample1_cov.bdg",
            sep = "\t", row.names = FALSE, col.names = FALSE, quote = FALSE)

# 步骤2: 使用 read_bedgraphs 读取
m <- read_bedgraphs(
  files = c("sample1_beta.bdg", "sample2_beta.bdg"),
  ref_build = "hg19",
  n_threads = 2
)

# 现在 m 是真正的 methrix 对象
class(m)  # "methrix" "RangedSummarizedExperiment"

# 可以使用所有 methrix 函数
stats <- get_stats(m)
m_filt <- coverage_filter(m, cov_thr = 10, min_samples = 2)
```

## 方法 4: 实用工具脚本

### 提供的脚本

1. **`load_h5_as_methrix.R`** - 最简单
   - 直接读取 HDF5
   - 创建 SummarizedExperiment
   - 保存为 se.rds

2. **`rust_to_methrix.R`** - 自动化
   - 自动查找 assays.h5
   - 创建/加载 se.rds
   - 提供使用示例

3. **`create_methrix_from_h5.R`** - 完整功能
   - 读取 HDF5
   - 创建对象
   - 测试功能
   - 保存结果

### 使用示例

```bash
# 方法1: 直接读取
Rscript load_h5_as_methrix.R output/assays.h5

# 方法2: 指定目录
Rscript rust_to_methrix.R output/

# 在R中使用
R
> source("load_h5_as_methrix.R")
> se <- load_h5_as_methrix("output/")
> assays(se)
```

## 📊 数据结构对比

### Rust HDF5 格式
```
assays.h5
├── /assay001        # beta (80028 x 2)
├── /assay002        # cov (80028 x 2)
├── /rowData
│   ├── chr          # 染色体
│   ├── start        # 起始位置 (0-based)
│   ├── end          # 结束位置
│   └── strand       # 链
├── /colData
│   └── sample_id    # 样本ID
└── /metadata
    ├── genome       # "hg19"
    └── is_h5        # TRUE
```

### R methrix 对象
```r
SummarizedExperiment (或 methrix)
├── assays
│   ├── beta         # DelayedMatrix 或 matrix
│   └── cov          # DelayedMatrix 或 matrix
├── rowRanges        # GRanges object
├── colData          # DataFrame
└── metadata
    ├── genome       # "hg19"
    └── is_h5        # TRUE/FALSE
```

## 🎯 推荐工作流

### 日常分析

```r
# 1. 加载数据
library(rhdf5)
beta <- h5read("assays.h5", "/assay001")
cov <- h5read("assays.h5", "/assay002")

# 2. 分析
mean_meth <- colMeans(beta, na.rm = TRUE)
mean_cov <- colMeans(cov)

# 3. 可视化
hist(beta[, 1], main = "Sample 1 Methylation")
plot(beta[, 1], beta[, 2],
     xlab = "Sample 1", ylab = "Sample 2",
     main = "Methylation Comparison")
```

### 需要methrix功能时

```r
# 1. 转换为bedgraph
# (使用脚本或手动)

# 2. 创建methrix对象
library(methrix)
m <- read_bedgraphs(
  files = list.files(pattern = "_beta.bdg$"),
  ref_build = "hg19"
)

# 3. 使用methrix函数
stats <- get_stats(m)
m_filt <- coverage_filter(m, cov_thr = 10, min_samples = 2)
plot_stats(m_filt)
```

### 使用SummarizedExperiment（推荐）

```r
# 1. 创建SE对象
se <- readRDS("se.rds")

# 2. 使用Bioconductor生态
library(GenomicRanges)
library(SummarizedExperiment)

# 3. 分析
# 聚合到基因水平
gene_avg <- rowsum(assays(se)$beta, rowData(se)$gene)

# 差异分析
library(limma)
# ... 标准RNA-seq流程适用于甲基化数据

# 可视化
library(Gviz)
# ... 基因组浏览器可视化
```

## 💡 常见问题

### Q: 为什么不能直接创建methrix对象？
A: methrix类的构造函数不是导出的，只能通过`read_bedgraphs()`等函数创建。

### Q: SummarizedExperiment够用吗？
A: 对于大多数分析，是的！SummarizedExperiment是Bioconductor的基础类，支持所有标准操作。

### Q: 什么时候需要methrix对象？
A: 需要使用methrix特定的函数时，如`coverage_filter()`, `get_stats()`等。

### Q: 性能如何？
A:
- 直接读取HDF5：快，按需加载
- 内存矩阵：快，但占用内存
- DelayedArray：慢，但内存效率高

## 🚀 总结

**最推荐的方法**：
1. 日常使用：**直接读取HDF5**（方法1）
2. 需要SE功能：**使用se.rds**（方法2）
3. 需要methrix函数：**转换为bedgraph**（方法3）

**关键点**：
- Rust生成的HDF5格式完全兼容
- 数据可被R正确读取
- 虽然不能直接创建methrix对象，但可以通过bedgraph间接实现
- SummarizedExperiment提供更灵活的分析选项
