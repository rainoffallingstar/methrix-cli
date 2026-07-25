# Scripts 目录

本目录包含用于构建、测试和开发 methx 的实用脚本。

## 📋 脚本列表

### 构建脚本

#### `build.sh`
**用途**: 使用 conda 环境构建 methx

**功能**:
- 自动创建或激活 `rust_build` conda 环境
- 安装依赖 (Rust, HDF5)
- 设置 HDF5 环境变量
- 编译项目

**使用方法**:
```bash
./scripts/build.sh
```

**要求**:
- Conda 或 Miniconda
- Internet 连接 (首次运行时下载依赖)

**输出**:
- 编译后的二进制文件: `target/release/methx`

---

### 测试脚本

#### `test_real_data.sh`
**用途**: 使用真实数据测试 methx

**功能**:
- 下载参考基因组 (hg38)
- 提取 CpG 位点
- 处理 Bismark 文件
- 生成 HDF5 输出和 QC 报告

**使用方法**:
```bash
./scripts/test_real_data.sh
```

**要求**:
- 已编译的二进制文件
- 有效的测试数据路径

**输出**:
- HDF5 文件
- QC 报告 (Excel)

---

#### `test_workflow.sh`
**用途**: 测试完整的处理流程

**功能**:
- 检查参考基因组
- 运行完整的处理管道
- 验证输出

**使用方法**:
```bash
./scripts/test_workflow.sh
```

**特点**:
- 自动检查依赖
- 清晰的步骤输出
- 错误处理

---

## 🚀 快速开始

### 1. 构建项目
```bash
cd /path/to/methx
./scripts/build.sh
```

### 2. 运行测试
```bash
# 使用真实数据测试
./scripts/test_real_data.sh

# 或使用工作流测试
./scripts/test_workflow.sh
```

### 3. 验证结果
```bash
# 检查输出
ls -lh testdata/mCall/rust_output/

# 使用 R 验证
Rscript docs/r_scripts/verify_h5_structure.R testdata/mCall/rust_output/assays.h5
```

## 🔧 自定义

### 修改测试数据路径

编辑脚本中的路径变量：

```bash
# test_real_data.sh
TESTDATA_DIR="/path/to/your/data"
OUTPUT_DIR="/path/to/output"
GENOME_DIR="/path/to/genomes"
```

### 修改构建配置

编辑 `build.sh` 中的 conda 环境设置：

```bash
# 修改 conda 路径
source ~/path/to/conda/etc/profile.d/conda.sh

# 修改环境名
conda create -n my_env -y rust hdf5
```

## 📝 脚本开发规范

添加新脚本时，请遵循以下规范：

1. **Shebang**: 使用 `#!/bin/bash`
2. **错误处理**: 添加 `set -e`
3. **颜色输出**: 使用定义的颜色变量
4. **注释**: 说明脚本用途和功能
5. **参数检查**: 验证必需的参数和文件

### 模板

```bash
#!/bin/bash
# 脚本名称: script_name.sh
# 用途: 简短描述
# 作者: Your Name
# 日期: YYYY-MM-DD

set -e

# 颜色定义
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

# 帮助信息
usage() {
    echo "用法: $0 [选项]"
    echo "选项:"
    echo "  -h, --help     显示帮助"
    exit 0
}

# 参数解析
while [[ $# -gt 0 ]]; do
    case $1 in
        -h|--help)
            usage
            ;;
        *)
            echo "未知选项: $1"
            usage
            ;;
    esac
    shift
done

# 主要逻辑
echo -e "${YELLOW}执行脚本...${NC}"
# ... 脚本逻辑 ...

echo -e "${GREEN}完成!${NC}"
```

## 🐛 故障排除

### 问题: conda 未找到
```
错误: conda not found
```
**解决**: 安装 Anaconda 或 Miniconda

### 问题: HDF5 库未找到
```
错误: HDF5 headers not found
```
**解决**: 确保 conda 环境中安装了 HDF5
```bash
conda activate rust_build
conda install hdf5
```

### 问题: 权限被拒绝
```
bash: ./scripts/build.sh: Permission denied
```
**解决**: 添加执行权限
```bash
chmod +x scripts/*.sh
```

## 🔗 相关资源

- **构建文档**: [docs/BUILD.md](../docs/BUILD.md)
- **测试指南**: [docs/TESTING_QUICK_REF.md](../docs/TESTING_QUICK_REF.md)
- **SLURM 脚本**: [../slurm_scripts/](../slurm_scripts/)

---

**最后更新**: 2026-02-22
