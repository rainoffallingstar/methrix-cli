# 快速指南：从 Rust HDF5 加载数据

## 🚀 最简单的方法

```r
library(rhdf5)

# 读取数据
h5_file <- "output/assays.h5"
beta <- h5read(h5_file, "/assay001")
cov <- h5read(h5_file, "/assay002")

# 使用数据
dim(beta)  # 80028 x 2
summary(cov)

# 基本分析
mean_meth <- colMeans(beta, na.rm = TRUE)
hist(beta[, 1], main = "Methylation Distribution")
```

## 📦 使用 SummarizedExperiment

```r
# 1. 使用提供的脚本
Rscript load_h5_as_methrix.R output/

# 2. 加载
library(SummarizedExperiment)
se <- readRDS("output/se.rds")

# 3. 使用
assays(se)       # beta 和 cov
rowData(se)      # CpG 位点
colData(se)      # 样本
```

## 🎯 创建 methrix 对象

由于 methrix 类的构造函数不是导出的，最简单的方法是通过 bedgraph 文件：

```r
library(methrix)

# 1. 转换 HDF5 到 bedgraph 格式
# (需要为每个样本创建 *_beta.bdg 文件)

# 2. 读取
m <- read_bedgraphs(
  files = list.files(pattern = "_beta.bdg$"),
  ref_build = "hg19"
)

# 3. 使用 methrix 函数
stats <- get_stats(m)
m_filt <- coverage_filter(m, cov_thr = 10, min_samples = 2)
```

## 💡 推荐方案

**对于大多数用户**：直接读取 HDF5 ✨
```r
library(rhdf5)
beta <- h5read("assays.h5", "/assay001")
cov <- h5read("assays.h5", "/assay002")
```

**需要 SE 功能**：使用 se.rds
```r
se <- readRDS("se.rds")
```

**需要 methrix 函数**：转换为 bedgraph
```r
m <- read_bedgraphs(files = beta_files, ref_build = "hg19")
```

## 📚 完整文档

详见：`HOW_TO_LOAD_METHRIX_OBJECT.md`
