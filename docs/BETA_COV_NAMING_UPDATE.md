# Beta/Cov 命名更新说明

## 📋 更新概述

从 **v2.0** 开始，methrix-cli 生成的 HDF5 文件使用标准化的数据集名称：
- `beta` (甲基化值矩阵)
- `cov` (覆盖度矩阵)

这取代了之前的命名：
- ~~`assay001`~~ → `beta`
- ~~`assay002`~~ → `cov`

## ✅ 更新的文件

### 源代码
- `src/hdf5/se_compat.rs`: 将数据集名称从 `assay001/assay002` 修改为 `beta/cov`

### 文档
- `HDF5_STRUCTURE_VISUAL.md`: 更新可视化图和说明
- `HDF5_STRUCTURE_AND_COORDINATES.md`: 更新结构说明和示例代码

### R 脚本
- `visualize_h5_layers.R`: 更新可视化脚本
- `h5_coordinate_analysis.R`: 更新坐标分析脚本
- `analyze_h5_simple.R`: 更新简单分析脚本
- `read_h5_with_new_names.R`: 新增示例脚本，演示如何使用新名称

## 🎯 更新的 HDF5 结构

### 新结构 (v2.0)
```
assays.h5
├── /beta              FLOAT [80028 x 2]  - 甲基化值矩阵
├── /cov               INTEGER [80028 x 2] - 覆盖度矩阵
├── /rowData/
│   ├── chr            STRING [80028]
│   ├── start          INTEGER [80028]
│   ├── end            INTEGER [80028]
│   └── strand         STRING [80028]
├── /colData/
│   └── sample_id      STRING [2]
└── /metadata/
    ├── genome         STRING
    └── is_h5          ENUM
```

### 旧结构 (v1.0)
```
assays.h5
├── /assay001          FLOAT [80028 x 2]
├── /assay002          INTEGER [80028 x 2]
├── /rowData/
│   ├── chr            STRING [80028]
│   ├── start          INTEGER [80028]
│   ├── end            INTEGER [80028]
│   └── strand         STRING [80028]
├── /colData/
│   └── sample_id      STRING [2]
└── /metadata/
    ├── genome         STRING
    └── is_h5          ENUM
```

## 💡 使用方法

### 在 R 中读取 (使用新名称)

#### 方法 1: 直接读取
```r
library(rhdf5)

h5_file <- "assays.h5"

# 使用新名称读取
beta <- h5read(h5_file, "/beta")
cov <- h5read(h5_file, "/cov")
```

#### 方法 2: 延迟加载 (推荐)
```r
library(HDF5Array)

h5_file <- "assays.h5"

# 创建 HDF5Matrix 对象
beta_h5 <- HDF5Array(h5_file, "beta")
cov_h5 <- HDF5Array(h5_file, "cov")

# 创建 SummarizedExperiment
library(SummarizedExperiment)
se <- SummarizedExperiment(
  assays = list(beta = beta_h5, cov = cov_h5),
  rowRanges = GRanges(chr, IRanges(start + 1, end), strand),
  colData = DataFrame(sample_id = sample_names)
)
```

### 检查现有 HDF5 文件的数据集名称

```r
library(rhdf5)

h5_file <- "assays.h5"

# 查看文件结构
structure <- h5ls(h5_file, recursive = TRUE)
print(structure)

# 检查数据集名称
dataset_names <- structure$name
if ("beta" %in% dataset_names) {
  cat("使用新名称 (v2.0): beta, cov\n")
} else if ("assay001" %in% dataset_names) {
  cat("使用旧名称 (v1.0): assay001, assay002\n")
} else {
  cat("未知的命名格式\n")
}
```

## 🔄 兼容性说明

### 与 R methrix 包的兼容性

✅ **完全兼容** - 新名称与 R methrix 包的命名约定一致：

```r
# methrix 包内部使用的数据集名称
assay_names <- c("beta", "cov")

# 这与我们的新命名一致
```

### 向后兼容性

如果您有使用旧名称的 R 代码，需要进行简单的更新：

```r
# 旧代码 (v1.0)
beta <- h5read(h5_file, "/assay001")
cov <- h5read(h5_file, "/assay002")

# 新代码 (v2.0)
beta <- h5read(h5_file, "/beta")
cov <- h5read(h5_file, "/cov")
```

### 检测和适配旧文件

如果需要处理旧版本的 HDF5 文件，可以使用这个辅助函数：

```r
read_methrix_h5 <- function(h5_file) {
  library(rhdf5)
  library(HDF5Array)

  # 检查数据集名称
  structure <- h5ls(h5_file, recursive = TRUE)
  dataset_names <- structure$name

  # 根据版本选择名称
  if ("beta" %in% dataset_names) {
    # 新版本
    beta_name <- "beta"
    cov_name <- "cov"
  } else if ("assay001" %in% dataset_names) {
    # 旧版本
    beta_name <- "assay001"
    cov_name <- "assay002"
  } else {
    stop("未知的 HDF5 格式")
  }

  # 使用适当的名称读取
  beta_h5 <- HDF5Array(h5_file, beta_name)
  cov_h5 <- HDF5Array(h5_file, cov_name)

  return(list(beta = beta_h5, cov = cov_h5))
}

# 使用示例
data <- read_methrix_h5("assays.h5")
beta <- data$beta
cov <- data$cov
```

## 🎁 优势

使用 `beta` 和 `cov` 命名的优势：

1. **📖 更具描述性**: 名称直接反映数据内容
2. **🔗 标准化**: 与 R methrix 包和 Bioconductor 生态系统的命名一致
3. **🎯 更易理解**: 新用户可以立即理解每个数据集的用途
4. **✅ 更好的兼容性**: 与 methrix 包的内部命名约定完全一致
5. **📝 更清晰的代码**: 代码中 `assay(se, "beta")` 比 `assay(se, "assay001")` 更清晰

## 📝 迁移清单

如果您正在从 v1.0 迁移到 v2.0：

- [ ] 更新 R 脚本中的数据集名称
- [ ] 更新文档和注释
- [ ] 测试现有分析流程
- [ ] 重新生成 HDF5 文件 (使用新版本的 methrix-cli)
- [ ] 验证与 R methrix 包的兼容性

## 🚀 未来计划

- 保持 `beta` 和 `cov` 作为标准命名
- 添加更多可选的 assay 类型 (如 `M`, `MN` 等) 时将遵循类似的命名约定
- 确保所有新功能都与标准命名一致

---

**版本**: 2.0
**更新日期**: 2026-02-22
**相关文件**:
- `src/hdf5/se_compat.rs`
- `HDF5_STRUCTURE_VISUAL.md`
- `HDF5_STRUCTURE_AND_COORDINATES.md`
- `read_h5_with_new_names.R`
