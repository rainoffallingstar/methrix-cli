# Methrix CLI 测试状态报告

## 📊 当前状态

**时间**: 2025-02-22 09:37
**状态**: 🔄 调试中

## ✅ 已完成的工作

1. **项目编译成功**
   - 二进制文件: `target/release/methrix` (3.6MB)
   - 使用 HDF5 1.10.6
   - Conda 环境: rust_build

2. **参考基因组准备完成**
   - hg19 基因组已下载 (905MB)
   - CpG 位点已提取: **13,382,154** 个 CpG
   - 保存位置: `testdata/genomes/hg19_cpgs.ron`

3. **测试数据准备完成**
   - 2 个 Bismark 样本文件

## ⚠️ 当前问题

### 问题描述

`process` 命令在加载 RON 文件时出现逻辑错误：

```rust
// 当前代码逻辑 (有bug)
if Path::new(&genome).exists() {
    // 总是尝试当作 FASTA 文件处理
    extractor.extract()?
} else {
    // 检查 RON 文件
}
```

**问题**: 当 RON 文件存在时，代码进入第一个分支，尝试当作 FASTA 文件读取，导致错误。

### 修复方案

已修改 `src/cli/process.rs`，优先检查 RON 文件扩展名：

```rust
// 修复后的逻辑
if genome.ends_with(".ron") {
    // 直接加载 RON 文件
    load_cpg_data(&genome)?
} else if Path::new(&genome).exists() {
    // 检查文件扩展名决定如何处理
}
```

### 需要重新编译

由于 HDF5 依赖兼容性问题，release 版本重新编译遇到错误。需要：

1. 解决 HDF5 编译问题
2. 重新编译 release 版本
3. 或使用临时解决方案

## 🔧 临时解决方案

### 方案 1: 使用预提取的 CpG 数据

CpG 数据已经提取完成，可以直接使用：

```bash
# 手动运行处理命令
./target/release/methrix process \
  --input testdata/mCall \
  --output testdata/mCall/rust_output_final \
  --genome hg19 \
  --threads 8
```

但这需要修改代码逻辑。

### 方案 2: 修改命令参数

暂时修改代码，使其能正确识别 RON 文件。

### 方案 3: 使用 FASTA 文件

直接使用 FASTA 文件（每次会重新提取 CpG）：

```bash
./target/release/methrix process \
  --input testdata/mCall \
  --output testdata/mCall/rust_output_final \
  --genome testdata/genomes/hg19.fa \
  --threads 8
```

## 📁 已准备的文件

| 文件 | 路径 | 大小 |
|------|------|------|
| hg19 基因组 | `testdata/genomes/hg19.fa` | 905MB |
| CpG 数据 (RON) | `testdata/genomes/hg19_cpgs.ron` | ~400MB |
| 测试样本 (2个) | `testdata/mCall/*.bismark.cov.gz` | ~5MB |

## 📊 数据规模

- **CpG 位点**: 13,382,154
- **Contigs**: 24
- **测试样本**: 2

## 🎯 下一步行动

1. **优先**: 修复 HDF5 编译问题，重新编译 release 版本
2. **备选**: 使用临时解决方案完成测试
3. **长期**: 优化代码逻辑，更智能地检测文件类型

## 📝 已提交的任务

最近的 SLURM 任务都因同样的逻辑错误而失败：
- 36921509: FAILED
- 36921900: FAILED
- 36921904: FAILED
- 36921906: FAILED
- 36921908: FAILED
- 36921911: FAILED (当前)

**错误信息**: `Expected '@' or '>' at the start of the file but found '('`

---

**最后更新**: 2025-02-22 09:38
**问题**: RON 文件加载逻辑错误
**状态**: 等待修复
