# Reference Code

此目录包含参考代码，**不在 Git 版本控制中**。

## 📁 目录说明

`refer-code/` 包含用于参考和对比的外部代码实现，主要用于：

1. **兼容性验证**: 验证与 R methrix 包的兼容性
2. **行为对比**: 比较不同实现的输出结果
3. **学习参考**: 理解原始算法的实现细节

## 🔧 常见内容

### R methrix 包

```bash
# 克隆 R methrix 仓库
git clone https://github.com/CompEpigen/methrix.git refer-code/methrix
```

### 其他参考实现

如果有其他参考实现，可以放置在此目录：
- 原始 Python 实现
- 其他语言的端口
- 算法参考实现

## 📝 使用指南

### 设置参考代码

```bash
# R methrix 包
cd refer-code
git clone https://github.com/CompEpigen/methrix.git
```

### 运行参考实现

```r
# 在 R 中
library(methrix)

# 加载数据
m <- load_HDF5_methrix("../testdata/output/assays.h5")

# 比较结果
get_stats(m)
```

### 与 Rust 实现对比

```bash
# 1. 使用 Rust 实现
./target/release/methrix process \
    --input testdata/samples/ \
    --output testdata/rust_output/ \
    --genome testdata/genomes/hg19_cpgs.ron

# 2. 使用 R methrix
Rscript refer-code/methrix/scripts/process_methrix.R \
    --input testdata/samples/ \
    --output testdata/r_output/

# 3. 比较输出
Rscript docs/r_scripts/verify_r_compatibility.R
```

## 🚫 Git 配置

此目录已在 `.gitignore` 中配置：

```gitignore
# .gitignore
refer-code/
```

**重要**:
- ✅ 此目录中的文件**不会被**提交到 Git
- ✅ 可以安全地添加和修改参考代码
- ✅ 每个开发者可以根据需要克隆不同的参考实现

## 💡 最佳实践

### 1. 使用 Git Submodule（可选）

如果想要跟踪特定的参考实现：

```bash
# 添加 submodule
git submodule add https://github.com/CompEpigen/methrix.git refer-code/methrix

# 更新 submodule
git submodule update --remote refer-code/methrix
```

### 2. 文档化版本

在 `refer-code/VERSIONS.md` 中记录使用的版本：

```markdown
# Reference Code Versions

## R methrix
- Repository: https://github.com/CompEpigen/methrix
- Commit: abc123
- Date: 2026-02-22

## Other tools
- ...
```

### 3. 隔离环境

```bash
# 为参考代码创建独立的环境
# 避免与项目依赖冲突

# R packages
mkdir -p refer-code/r_libs
export R_LIBS_USER=$(pwd)/refer-code/r_libs
R -e "install.packages('methrix', lib='$R_LIBS_USER')"
```

## 📚 相关文档

- [R_METHRIX_COMPATIBILITY_GUIDE.md](../docs/R_METHRIX_COMPATIBILITY_GUIDE.md) - R 兼容性指南
- [TESTING_QUICK_REF.md](../docs/TESTING_QUICK_REF.md) - 测试参考
- [CONTRIBUTING.md](../CONTRIBUTING.md) - 贡献指南（如果存在）

## 🔗 相关资源

- **R methrix**: https://github.com/CompEpigen/methrix
- **Bioconductor**: https://bioconductor.org/packages/release/bioc/html/methrix/

---

**注意**: 此目录不在版本控制中。请根据需要添加参考代码。
