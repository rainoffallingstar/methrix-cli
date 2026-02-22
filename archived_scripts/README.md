# 归档脚本目录

本目录包含已归档的 R 脚本，这些脚本主要用于开发、测试和调试过程。

## ⚠️ 使用说明

**这些脚本仅供参考和调试目的**，不建议用于生产环境。

- 大多数脚本已被 `docs/r_scripts/` 中的新版本替代
- 某些脚本包含重复功能或已过时的方法
- 保留这些脚本用于历史记录和故障排除

## 📋 归档脚本列表

### HDF5 结构分析脚本

| 脚本 | 状态 | 替代方案 |
|------|------|----------|
| `analyze_h5_structure.R` | 已归档 | `docs/r_scripts/analyze_h5_simple.R` |
| `analyze_h5_simple.R` | 已归档 | `docs/r_scripts/analyze_h5_simple.R` |
| `analyze_methrix_class.R` | 已归档 | `docs/r_scripts/verify_r_compatibility.R` |
| `analyze_se_rds.R` | 已归档 | `docs/r_scripts/create_se_rds_v2.R` |

### HDF5 检查脚本

| 脚本 | 状态 | 替代方案 |
|------|------|----------|
| `check_assays_h5.R` | 已归档 | `docs/r_scripts/verify_h5_structure.R` |
| `check_r_h5_structure.R` | 已归档 | `docs/r_scripts/verify_h5_structure.R` |
| `check_rust_h5.R` | 已归档 | `docs/r_scripts/verify_h5_structure.R` |
| `inspect_assays_detailed.R` | 已归档 | `docs/r_scripts/analyze_h5_simple.R` |
| `inspect_h5.R` | 已归档 | `docs/r_scripts/visualize_h5_layers.R` |

### Methrix 对象创建脚本

| 脚本 | 状态 | 替代方案 |
|------|------|----------|
| `convert_to_methrix.R` | 已归档 | `docs/r_scripts/read_h5_with_new_names.R` |
| `create_methrix_direct.R` | 已归档 | `docs/r_scripts/read_h5_with_new_names.R` |
| `create_methrix_from_h5.R` | 已归档 | `docs/r_scripts/read_h5_with_new_names.R` |
| `create_methrix_object.R` | 已归档 | `docs/r_scripts/read_h5_with_new_names.R` |

### HDF5 加载脚本

| 脚本 | 状态 | 替代方案 |
|------|------|----------|
| `load_h5_as_methrix.R` | 已归档 | `docs/r_scripts/read_h5_with_new_names.R` |
| `load_h5_to_methrix.R` | 已归档 | `docs/r_scripts/read_h5_with_new_names.R` |
| `load_h5_to_methrix_v2.R` | 已归档 | `docs/r_scripts/read_h5_with_new_names.R` |
| `rust_to_methrix.R` | 已归档 | `docs/r_scripts/read_h5_with_new_names.R` |

### RDS 创建脚本

| 脚本 | 状态 | 替代方案 |
|------|------|----------|
| `create_se_rds.R` | 已归档 | `docs/r_scripts/create_se_rds_v2.R` |

### 测试和验证脚本

| 脚本 | 状态 | 替代方案 |
|------|------|----------|
| `test_load_assays.R` | 已归档 | `docs/r_scripts/verify_r_compatibility.R` |
| `test_load_methrix.R` | 已归档 | `docs/r_scripts/verify_r_compatibility.R` |
| `test_methrix_functions.R` | 已归档 | `docs/r_scripts/verify_r_compatibility.R` |
| `verify_data.R` | 已归档 | `docs/r_scripts/verify_h5_structure.R` |
| `verify_h5_compatibility.R` | 已归档 | `docs/r_scripts/verify_r_compatibility.R` |

## 🔄 迁移指南

如果您正在使用这些归档脚本，建议迁移到新版本：

### 从 `load_h5_to_methrix*.R` 迁移
```r
# 旧方法
source("archived_scripts/load_h5_to_methrix_v2.R")

# 新方法
source("docs/r_scripts/read_h5_with_new_names.R")
```

### 从 `check_*_h5.R` 迁移
```r
# 旧方法
source("archived_scripts/check_assays_h5.R")

# 新方法
Rscript docs/r_scripts/verify_h5_structure.R /path/to/assays.h5
```

### 从 `create_se_rds.R` 迁移
```r
# 旧方法
source("archived_scripts/create_se_rds.R")

# 新方法
Rscript docs/r_scripts/create_se_rds_v2.R
```

## 📝 归档原因

### 主要原因

1. **功能重复**: 多个脚本实现相同功能
2. **命名变更**: HDF5 数据集名称从 `assay001/assay002` 改为 `beta/cov`
3. **代码优化**: 新版本更简洁、更易维护
4. **文档完善**: 新版本包含更好的文档和错误处理

### 时间线

- **2026-02-22**: 整理和归档旧脚本
- **v2.0**: 更新 HDF5 数据集命名
- **v1.0**: 初始实现

## 🔧 仍在维护的脚本

以下脚本仍在 `docs/r_scripts/` 中维护：

- ✅ `read_h5_with_new_names.R` - 主要的 HDF5 读取脚本
- ✅ `verify_h5_structure.R` - 结构验证脚本
- ✅ `visualize_h5_layers.R` - 可视化脚本
- ✅ `h5_coordinate_analysis.R` - 坐标分析脚本
- ✅ `analyze_h5_simple.R` - 简化分析脚本
- ✅ `create_se_rds_v2.R` - RDS 创建脚本
- ✅ `verify_r_compatibility.R` - 兼容性验证脚本

## 💡 如果需要使用归档脚本

如果您确实需要使用这些归档脚本：

1. **检查数据集名称**:
   - 旧脚本使用 `assay001/assay002`
   - 新 HDF5 文件使用 `beta/cov`
   - 可能需要修改脚本中的数据集名称

2. **更新文件路径**:
   - 修改脚本中的硬编码路径
   - 使用命令行参数或配置文件

3. **测试兼容性**:
   - 先在测试数据上运行
   - 验证输出结果
   - 检查是否有错误或警告

## 📚 相关资源

- **新脚本**: [docs/r_scripts/](../docs/r_scripts/)
- **文档**: [docs/INDEX.md](../docs/INDEX.md)
- **HDF5 结构**: [docs/HDF5_STRUCTURE_AND_COORDINATES.md](../docs/HDF5_STRUCTURE_AND_COORDINATES.md)

---

**归档日期**: 2026-02-22
**维护状态**: 不再维护，仅供参考
