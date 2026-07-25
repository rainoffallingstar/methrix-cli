# Test Data

此目录包含测试数据，**不在 Git 版本控制中**。

## 📁 目录说明

### `testdata/`
测试数据目录，包含：
- Bismark 输出文件（.cov.gz）
- 参考基因组数据（.fa）
- CpG 位点数据（.ron）
- 输出文件（.h5, .xlsx）

### `refer-code/`
参考代码目录，包含：
- R methrix 包的参考实现
- 用于兼容性验证

## 🔄 获取测试数据

### 方法 1: 使用示例数据

如果项目提供示例数据脚本：

```bash
# 运行测试数据生成脚本
python scripts/generate_test_data.py
```

### 方法 2: 使用真实数据

如果您有 Bismark 输出文件：

```bash
# 创建测试数据目录
mkdir -p testdata/samples

# 复制您的 Bismark 文件
cp /path/to/your/*.bismark.cov.gz testdata/samples/
```

### 方法 3: 下载参考基因组

```bash
# 使用 methx 下载
./target/release/methx download-genome \
    --genome hg19 \
    --output testdata/genomes/

# 或使用 wget
wget -O testdata/genomes/hg19.fa.gz \
    https://hgdownload.soe.ucsc.edu/goldenPath/hg19/bigZips/hg19.fa.gz

gunzip testdata/genomes/hg19.fa.gz
```

### 方法 4: 提取 CpG 位点

```bash
# 从基因组提取 CpG 位点
./target/release/methx extract-cpgs \
    --genome testdata/genomes/hg19.fa \
    --output testdata/genomes/hg19_cpgs.ron
```

## 📊 测试数据结构

```
testdata/
├── genomes/              # 参考基因组
│   ├── hg19.fa
│   ├── hg38.fa
│   └── *_cpgs.ron        # CpG 位点数据
│
├── samples/              # Bismark 输出文件
│   ├── sample1.bismark.cov.gz
│   ├── sample2.bismark.cov.gz
│   └── ...
│
└── output/               # 测试输出（自动生成）
    ├── assays.h5
    └── CpG_coverage.xlsx
```

## 🧪 运行测试

### 快速测试

```bash
# 使用 2 个样本测试
./target/release/methx process \
    --input testdata/samples/ \
    --output testdata/output/ \
    --genome testdata/genomes/hg19_cpgs.ron \
    --threads 4
```

### 完整测试

```bash
# 使用所有样本测试
./target/release/methx process \
    --input testdata/samples/ \
    --output testdata/output/ \
    --genome testdata/genomes/hg19_cpgs.ron \
    --threads 8 \
    --remove-uncovered
```

## 🔍 验证输出

### 使用 R 验证

```r
library(rhdf5)

# 读取 HDF5 文件
h5_file <- "testdata/output/assays.h5"
structure <- h5ls(h5_file, recursive = TRUE)
print(structure)

# 读取数据
beta <- h5read(h5_file, "/beta")
cov <- h5read(h5_file, "/cov")
```

### 使用项目提供的脚本

```bash
# 验证 HDF5 结构
Rscript docs/r_scripts/verify_h5_structure.R testdata/output/assays.h5
```

## 📝 数据要求

### Bismark 输入文件

- **格式**: Bismark coverage output (.cov.gz)
- **列数**: 6 列
- **坐标**: 1-based (自动转换为 0-based)
- **压缩**: gzip 压缩

### 参考基因组

- **格式**: FASTA (.fa)
- **压缩**: 可选 gzip (.fa.gz)
- **索引**: 自动生成

## 🗑️ 清理测试数据

```bash
# 清理输出文件
rm -rf testdata/output/

# 清理所有测试数据（谨慎！）
# rm -rf testdata/
```

## 💡 提示

1. **不要提交测试数据到 Git**
   - .gitignore 已配置忽略 testdata/
   - 大文件会影响仓库大小

2. **使用符号链接**（可选）
   ```bash
   # 如果数据在其他位置
   ln -s /path/to/real/data testdata/samples
   ```

3. **压缩数据**
   - Bismark 文件已经是 gzip 压缩
   - 基因组文件可以用 gzip 压缩节省空间

4. **权限**
   - 确保对这些目录有读写权限
   - 输出目录会自动创建

## 📚 相关文档

- [BUILD.md](../docs/BUILD.md) - 构建指南
- [QUICKSTART.md](../docs/QUICKSTART.md) - 快速开始
- [TESTING_QUICK_REF.md](../docs/TESTING_QUICK_REF.md) - 测试参考

---

**注意**: 此目录不在版本控制中。所有更改不会被提交到 Git。
