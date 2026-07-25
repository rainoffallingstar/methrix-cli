# R 脚本目录

本目录包含用于分析和验证 methx 生成的 HDF5 文件的 R 脚本。

## 📋 脚本列表

### 核心分析脚本

#### `read_h5_with_new_names.R`
**用途**: 演示如何读取使用新命名 (beta/cov) 的 HDF5 文件

**功能**:
- 读取 HDF5 文件结构
- 使用 `rhdf5` 和 `HDF5Array` 读取数据
- 创建 SummarizedExperiment 对象
- 演示数据访问方法

**运行方式**:
```r
Rscript read_h5_with_new_names.R
```

或指定文件：
```r
Rscript read_h5_with_new_names.R /path/to/assays.h5
```

---

#### `verify_h5_structure.R`
**用途**: 验证 HDF5 文件结构和命名

**功能**:
- 检查文件结构
- 验证数据集命名 (v1.0 vs v2.0)
- 测试数据读取
- 检查结构完整性

**运行方式**:
```r
Rscript verify_h5_structure.R
```

或指定文件：
```r
Rscript verify_h5_structure.R /path/to/assays.h5
```

---

#### `visualize_h5_layers.R`
**用途**: 可视化 HDF5 文件的层级结构

**功能**:
- 显示 ASCII 艺术的文件结构
- 说明各层级的含义
- 解释数据组织方式

**运行方式**:
```r
Rscript visualize_h5_layers.R
```

---

#### `h5_coordinate_analysis.R`
**用途**: 分析 HDF5 文件中的坐标信息

**功能**:
- 读取坐标数据
- 显示 CpG 位点信息
- 生成 CpG ID
- 说明坐标系统

**运行方式**:
```r
Rscript h5_coordinate_analysis.R
```

---

#### `analyze_h5_simple.R`
**用途**: 简化的 HDF5 文件分析

**功能**:
- 快速查看文件结构
- 检查数据维度
- 验证坐标系统
- 示例数据访问

**运行方式**:
```r
Rscript analyze_h5_simple.R
```

---

### 数据导出脚本

#### `create_se_rds_v2.R`
**用途**: 从 HDF5 文件创建 SummarizedExperiment RDS 对象

**功能**:
- 读取 HDF5 数据
- 创建 SummarizedExperiment 对象
- 导出为 .rds 文件
- 与 R methrix 包兼容

**运行方式**:
```r
Rscript create_se_rds_v2.R
```

---

### 兼容性验证脚本

#### `verify_r_compatibility.R`
**用途**: 验证与 R methrix 包的兼容性

**功能**:
- 测试数据加载
- 验证数据类型
- 检查与 methrix 函数的兼容性
- 生成兼容性报告

**运行方式**:
```r
Rscript verify_r_compatibility.R
```

---

## 🎯 使用场景

### 场景 1: 快速检查 HDF5 文件
```bash
# 验证文件结构
Rscript verify_h5_structure.R /path/to/assays.h5

# 查看可视化结构
Rscript visualize_h5_layers.R

# 简单分析
Rscript analyze_h5_simple.R
```

### 场景 2: 在 R 中加载数据进行分析
```r
# 运行加载示例
source("read_h5_with_new_names.R")

# 或者直接使用脚本
Rscript read_h5_with_new_names.R /path/to/assays.h5
```

### 场景 3: 导出为 RDS 对象
```r
# 创建 RDS 对象用于后续分析
Rscript create_se_rds_v2.R
```

### 场景 4: 验证兼容性
```r
# 检查与 R methrix 的兼容性
Rscript verify_r_compatibility.R
```

## 📦 依赖要求

所有脚本都需要以下 R 包：

```r
# 核心包
install.packages("rhdf5")
install.packages("HDF5Array")

# Bioconductor 包
if (!requireNamespace("BiocManager", quietly = TRUE))
    install.packages("BiocManager")
BiocManager::install("SummarizedExperiment")
BiocManager::install("GenomicRanges")
BiocManager::install("S4Vectors")

# 可选：methrix 包
BiocManager::install("methrix")
```

## 🔧 自定义使用

### 修改文件路径

大多数脚本在开头定义了文件路径：

```r
# 默认路径
h5_file <- "testdata/mCall/rust_output_20260222_112603_job36922017/assays.h5"

# 修改为您的路径
h5_file <- "/path/to/your/assays.h5"
```

### 在 R 交互式环境中使用

```r
# 加载脚本内容
source("read_h5_with_new_names.R")

# 或者直接复制粘贴代码块到 R 控制台
```

## 📊 输出示例

### verify_h5_structure.R 输出
```
==========================================
HDF5 文件结构验证
==========================================

文件: assays.h5
大小: 7.2 MB

文件结构:
  name    group   type    dimentions
1 beta    /       FLOAT   80028 x 2
2 cov     /       INTEGER 80028 x 2
3 rowData rowData  -       -
4 chr     rowData STRING  80028
5 start   rowData INTEGER 80028
...

命名验证:
----------------------------------------
✓ 使用新命名 (v2.0)
  - beta: 存在
  - cov: 存在 ✓
```

### read_h5_with_new_names.R 输出
```
==========================================
读取使用新名称的 HDF5 文件
==========================================

beta 矩阵维度: 80028 2
前3行前2列:
          Sample1    Sample2
CpG1     0.9800000  0.9500000
CpG2     0.8500000  0.9000000
CpG3            NA  0.7800000
...
```

## 🐛 故障排除

### 问题: 找不到文件
```
错误: 文件不存在: assays.h5
```
**解决**: 修改脚本中的文件路径为实际路径

### 问题: 缺少依赖包
```
错误: 没有"HDF5Array"这个程序包
```
**解决**: 安装缺失的包（见"依赖要求"部分）

### 问题: HDF5 版本不兼容
```
错误: HDF5 版本不匹配
```
**解决**:
```r
# 检查 HDF5 版本
rhdf5::h5Version()

# 如需要，重新安装 rhdf5
BiocManager::install("rhdf5", force = TRUE)
```

## 📝 脚本开发规范

添加新脚本时，请遵循以下规范：

1. **文件命名**: 使用描述性名称，如 `analyze_feature.R`
2. **头部注释**: 说明用途、功能、运行方式
3. **依赖检查**: 检查必需的包是否已安装
4. **错误处理**: 使用 `tryCatch()` 处理错误
5. **输出格式**: 使用清晰的格式（表格、分隔线等）
6. **示例数据**: 提供示例输出或使用场景

示例模板：
```r
#!/usr/bin/env Rscript
# 脚本名称: script_name.R
# 用途: 简短描述
# 作者: Your Name
# 日期: YYYY-MM-DD

# 依赖检查
if (!require("package_name")) {
  install.packages("package_name")
}

# 主要代码
# ...

# 输出结果
cat("结果:\n")
print(result)
```

## 🔗 相关文档

- [HDF5_STRUCTURE_AND_COORDINATES.md](../HDF5_STRUCTURE_AND_COORDINATES.md)
- [HOW_TO_LOAD_METHRIX_OBJECT.md](../HOW_TO_LOAD_METHRIX_OBJECT.md)
- [R_METHRIX_COMPATIBILITY_GUIDE.md](../R_METHRIX_COMPATIBILITY_GUIDE.md)

---

**最后更新**: 2026-02-22
