# methx 测试快速参考

## 🚀 快速开始

### 方法 1: 使用提交助手 (推荐)

```bash
cd /public3/home/scg9946/methx
./submit_test.sh
```

### 方法 2: 直接提交

```bash
# 快速测试 (2样本, 30分钟)
sbatch run_quick_test.sbatch

# 完整测试 (12样本, 2小时)
sbatch run_test.sbatch
```

## 📊 监控任务

```bash
# 查看任务状态
squeue -u $USER

# 查看特定任务
squeue -j <JOB_ID>

# 实时查看日志
tail -f logs/methx_quick_<JOB_ID>.out

# 查看完整日志
cat logs/methx_quick_<JOB_ID>.out
```

## ✅ 验证结果

```bash
# R 验证
cd /public3/home/scg9946/methx
Rscript verify_r_compatibility.R
```

## 📁 文件位置

| 文件 | 路径 |
|------|------|
| 二进制 | `target/release/methx` |
| 快速测试脚本 | `run_quick_test.sbatch` |
| 完整测试脚本 | `run_test.sbatch` |
| 提交助手 | `submit_test.sh` |
| R 验证脚本 | `verify_r_compatibility.R` |
| 测试日志 | `logs/methx_*_<JOB_ID>.out` |
| 快速测试输出 | `testdata/mCall/rust_output_quick/` |
| 完整测试输出 | `testdata/mCall/rust_output/` |

## 📋 测试数据

- **位置**: `testdata/mCall/`
- **样本数**: 12
- **格式**: Bismark `.bismark.cov.gz`
- **R 参考**: `testdata/mCall/methrixh5/`

## 🔧 常用命令

```bash
# 取消任务
scancel <JOB_ID>

# 查看任务历史
sacct -u $USER -l

# 查看节点信息
sinfo

# 查看配额
sacctmgr show qos

# 清理输出文件
rm -rf testdata/mCall/rust_output*
```

## ⚠️ 注意事项

1. **首次运行**: 建议先运行快速测试
2. **磁盘空间**: 确保有足够空间 (~10GB)
3. **HDF5 版本**: 需要 HDF5 1.10.x
4. **Conda 环境**: 确保 rust_build 环境已创建

## 📞 获取帮助

- 查看详细文档: `SLURM_TEST_GUIDE.md`
- 查看编译状态: `BUILD_STATUS.md`
- 查看 AI 指南: `CLAUDE.md`

---

**提示**: 首次运行建议使用快速测试验证功能正常！
