# SLURM 脚本目录

本目录包含用于在 SLURM 集群上运行 methrix-cli 的脚本。

## 🖥️ 关于 SLURM

SLURM (Simple Linux Utility for Resource Management) 是一个 Linux 集群管理和作业调度系统。

## 📋 脚本列表

### 任务提交脚本

#### `submit_test.sh`
**用途**: 交互式提交测试任务到 SLURM 集群

**功能**:
- 交互式选择测试类型
- 自动配置资源
- 提交任务并监控状态

**使用方法**:
```bash
./slurm_scripts/submit_test.sh
```

**测试选项**:
1. **快速测试**: 2个样本, ~30分钟, 8核32GB
2. **完整测试**: 12个样本, ~2小时, 16核64GB
3. **自定义配置**: 自定义资源

**示例输出**:
```
请选择测试类型:
  1) 快速测试 (2个样本, ~30分钟, 8核32GB)
  2) 完整测试 (12个样本, ~2小时, 16核64GB)
  3) 自定义配置

请输入选项 [1-3]: 1

任务已提交!
任务信息:
  Job ID: 12345
  脚本: run_quick_test.sbatch
```

---

### 任务监控脚本

#### `monitor_job.sh`
**用途**: 实时监控 SLURM 任务状态

**功能**:
- 实时显示任务状态
- 显示输出日志
- 自动刷新

**使用方法**:
```bash
# 监控特定任务
./slurm_scripts/monitor_job.sh 12345

# 监控最近的任务
./slurm_scripts/monitor_job.sh
```

**监控功能**:
- 任务状态 (RUNNING, PENDING, COMPLETED)
- 资源使用情况
- 实时日志输出
- 错误日志

---

## 🚀 快速开始

### 1. 提交任务
```bash
cd /path/to/methrix-cli
./slurm_scripts/submit_test.sh
```

### 2. 监控任务
```bash
# 实时监控
./slurm_scripts/monitor_job.sh

# 或使用 SLURM 命令
squeue -j <JOB_ID>
```

### 3. 查看日志
```bash
# 查看输出日志
tail -f logs/methrix_*_<JOB_ID>.out

# 查看错误日志
tail -f logs/methrix_*_<JOB_ID>.err
```

### 4. 取消任务
```bash
scancel <JOB_ID>
```

## 📊 SLURM 基础命令

### 任务管理
```bash
# 提交任务
sbatch script.sbatch

# 查看任务状态
squeue -u $USER

# 查看特定任务
squeue -j <JOB_ID>

# 取消任务
scancel <JOB_ID>

# 取消所有任务
scancel -u $USER
```

### 日志管理
```bash
# 查看输出
cat logs/methrix_*_<JOB_ID>.out

# 查看错误
cat logs/methrix_*_<JOB_ID>.err

# 实时监控
tail -f logs/methrix_*_<JOB_ID>.out
```

### 资源查看
```bash
# 查看任务详情
sacct -j <JOB_ID>

# 查看资源使用
sstat -j <JOB_ID>

# 查看集群状态
sinfo
```

## 📝 SBATCH 脚本模板

### 基本模板
```bash
#!/bin/bash
#SBATCH --job-name=methrix_test      # 任务名称
#SBATCH --output=logs/%j.out         # 输出日志
#SBATCH --error=logs/%j.err          # 错误日志
#SBATCH --cpus-per-task=8            # CPU 核心数
#SBATCH --mem=32G                    # 内存
#SBATCH --time=01:00:00              # 时间限制
#SBATCH --partition=normal           # 分区

# 你的命令
./target/release/methrix process \
    --input data/ \
    --output output/ \
    --genome hg38
```

### 高级模板
```bash
#!/bin/bash
#SBATCH --job-name=methrix_test
#SBATCH --output=logs/%j.out
#SBATCH --error=logs/%j.err
#SBATCH --cpus-per-task=16
#SBATCH --mem=64G
#SBATCH --time=04:00:00
#SBATCH --partition=highmem
#SBATCH --mail-type=FAIL,END         # 邮件通知
#SBATCH --mail-user=user@example.com

# 加载模块
module load hdf5

# 激活 conda 环境
source ~/miniconda/etc/profile.d/conda.sh
conda activate rust_build

# 设置环境变量
export HDF5_DIR=$CONDA_PREFIX

# 运行任务
echo "开始时间: $(date)"

./target/release/methrix process \
    --input data/ \
    --output output/ \
    --genome hg38 \
    --threads $SLURM_CPUS_PER_TASK

echo "结束时间: $(date)"
```

## 🔧 配置优化

### 快速测试 (小数据)
```bash
#SBATCH --cpus-per-task=8
#SBATCH --mem=32G
#SBATCH --time=00:30:00
```

### 完整测试 (大数据)
```bash
#SBATCH --cpus-per-task=16
#SBATCH --mem=64G
#SBATCH --time=04:00:00
```

### 高内存任务
```bash
#SBATCH --cpus-per-task=8
#SBATCH --mem=128G
#SBATCH --partition=highmem
#SBATCH --time=08:00:00
```

## 🐛 故障排除

### 问题: 任务提交失败
```
错误: Job submit failed
```
**解决**: 检查分区名称和资源限制
```bash
sinfo  # 查看可用分区
```

### 问题: 任务被杀死
```
状态: KILLED
```
**解决**: 增加内存或时间限制
```bash
#SBATCH --mem=64G        # 增加内存
#SBATCH --time=04:00:00  # 增加时间
```

### 问题: 任务一直等待
```
状态: PENDING
```
**解决**: 检查集群负载或调整分区
```bash
squeue  # 查看队列
sinfo   # 查看分区状态
```

### 问题: 找不到模块
```
错误: module: command not found
```
**解决**: 初始化 SLURM 环境
```bash
source /etc/profile.d/modules.sh
```

## 📚 资源估算

### 数据规模 vs 资源需求

| 样本数 | CpG 位点 | CPU | 内存 | 时间 |
|--------|---------|-----|------|------|
| 2 | ~50K | 8 | 32GB | 30min |
| 5 | ~50K | 8 | 32GB | 1h |
| 10 | ~50K | 16 | 64GB | 2h |
| 20 | ~50K | 16 | 64GB | 4h |
| 50 | ~50K | 32 | 128GB | 8h |

### 内存估算
```
基础内存 = 2GB
每样本内存 = 500MB × 样本数
每100K CpG = 1GB

总内存 ≈ 基础内存 + 样本内存 + CpG内存
```

## 🔗 相关资源

- **SLURM 文档**: https://slurm.schedmd.com/documentation.html
- **集群测试指南**: [docs/SLURM_TEST_GUIDE.md](../docs/SLURM_TEST_GUIDE.md)
- **构建脚本**: [../scripts/](../scripts/)
- **测试文档**: [docs/TESTING_QUICK_REF.md](../docs/TESTING_QUICK_REF.md)

## 📞 获取帮助

### 本地帮助
```bash
sbatch --help     # sbatch 帮助
squeue --help     # squeue 帮助
sacct --help      # sacct 帮助
```

### 集群管理员
- 查看集群配置: `sinfo`
- 查看任务优先级: `sprio`
- 查看队列限制: `scontrol show config`

---

**最后更新**: 2026-02-22
**适用版本**: SLURM 20.02+
