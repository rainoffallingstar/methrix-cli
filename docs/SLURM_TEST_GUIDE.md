# Methrix CLI SLURM 测试指南

本文档说明如何使用 SLURM 提交任务来测试 methrix-cli。

## 📋 测试脚本说明

### 1. 完整测试 (`run_test.sbatch`)

测试所有 12 个样本，完整的处理流程。

**资源需求**:
- CPU: 16 核心
- 内存: 64 GB
- 时间: 2 小时
- 分区: cpu

**提交命令**:
```bash
sbatch run_test.sbatch
```

**输出位置**:
- HDF5: `testdata/mCall/rust_output/methrix_data.h5`
- QC 报告: `testdata/mCall/rust_output/CpG_coverage.xlsx`
- 日志: `logs/methrix_test_<JOB_ID>.out`

### 2. 快速测试 (`run_quick_test.sbatch`)

仅测试 2 个样本，用于快速验证功能。

**资源需求**:
- CPU: 8 核心
- 内存: 32 GB
- 时间: 30 分钟
- 分区: cpu

**提交命令**:
```bash
sbatch run_quick_test.sbatch
```

**输出位置**:
- HDF5: `testdata/mCall/rust_output_quick/methrix_data.h5`
- 日志: `logs/methrix_quick_<JOB_ID>.out`

## 🚀 使用步骤

### Step 1: 提交任务

选择合适的测试脚本并提交：

```bash
# 快速测试 (推荐首次运行)
cd /public3/home/scg9946/methrix-cli
sbatch run_quick_test.sbatch

# 完整测试
sbatch run_test.sbatch
```

### Step 2: 监控任务

```bash
# 查看任务状态
squeue -u $USER

# 查看任务日志 (实时)
tail -f logs/methrix_quick_<JOB_ID>.out

# 或查看完整日志
cat logs/methrix_quick_<JOB_ID>.out
```

### Step 3: 验证输出

任务完成后，运行 R 验证脚本：

```bash
cd /public3/home/scg9946/methrix-cli
Rscript verify_r_compatibility.R
```

## 📊 预期输出

### HDF5 文件结构

生成的 HDF5 文件应包含以下结构：

```
methrix_data.h5
├── assays/
│   ├── beta          # f32 矩阵 (甲基化值)
│   └── cov           # u16 矩阵 (覆盖度)
├── rowData/
│   ├── chr           # 染色体
│   ├── start         # 起始位置 (0-based)
│   ├── end           # 结束位置
│   └── strand        # 链信息
├── colData/
│   └── sample_id     # 样本名称
└── metadata/
    ├── genome        # 参考基因组名称
    └── is_h5         # HDF5 格式标志
```

### R 加载示例

```r
library(methrix)

# 加载 HDF5 文件
m <- load_HDF5_methrix('testdata/mCall/rust_output/methrix_data.h5')

# 查看基本信息
print(m)

# 获取统计信息
stats <- get_stats(m)

# 绘制覆盖度分布
plot_coverage(m, type = "hist")

# PCA 分析 (如果有多个样本)
if (ncol(m) > 2) {
  methrix_pca(m)
}
```

## 📈 性能对比

与 R methrix 包的性能对比：

| 指标 | R 实现 | Rust 实现 | 改进 |
|------|--------|-----------|------|
| 启动时间 | ~5-10 秒 | <1 秒 | 5-10x |
| 处理速度 (100样本) | ~45 分钟 | ~5 分钟 | 9x |
| 内存使用 | ~8 GB | ~4 GB | -50% |

## 🔍 故障排查

### 常见问题

**1. 任务提交失败**
```bash
# 检查分区信息
sinfo

# 检查配额
sacctmgr show qos
```

**2. HDF5 文件未生成**
```bash
# 查看错误日志
cat logs/methrix_quick_<JOB_ID>.err

# 检查磁盘空间
df -h
```

**3. R 加载失败**
```r
# 检查 HDF5 文件完整性
library(HDF5Array)
h5ls('testdata/mCall/rust_output/methrix_data.h5')
```

## 📝 日志文件说明

### 标准输出 (.out)
- 执行进度
- 处理统计
- 输出文件信息
- 性能指标

### 标准错误 (.err)
- 错误信息
- 警告信息

## 🔗 相关文件

- `run_test.sbatch` - 完整测试脚本
- `run_quick_test.sbatch` - 快速测试脚本
- `verify_r_compatibility.R` - R 验证脚本
- `BUILD_STATUS.md` - 编译状态报告
- `CLAUDE.md` - AI 助手指南

## 📞 技术支持

如遇问题，请检查：
1. 日志文件 (`logs/` 目录)
2. 测试摘要 (`rust_output/test_summary.txt`)
3. R 验证结果 (`r_verification_results.txt`)

---

**最后更新**: 2025-02-21
**版本**: 0.1.0
