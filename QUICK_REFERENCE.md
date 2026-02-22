# 快速参考

## 📚 文档导航

### 🚀 新用户
1. [README.md](README.md) - 项目概述
2. [docs/QUICKSTART.md](docs/QUICKSTART.md) - 快速开始
3. [docs/BUILD.md](docs/BUILD.md) - 构建指南

### 🔬 R 用户
1. [docs/HDF5_STRUCTURE_AND_COORDINATES.md](docs/HDF5_STRUCTURE_AND_COORDINATES.md) - HDF5 结构
2. [docs/QUICK_START_LOADING.md](docs/QUICK_START_LOADING.md) - 快速加载
3. [docs/r_scripts/README.md](docs/r_scripts/README.md) - R 脚本使用

### 👨‍💻 开发者
1. [docs/DESIGN.md](docs/DESIGN.md) - 架构设计
2. [docs/API.md](docs/API.md) - API 文档
3. [docs/STATUS.md](docs/STATUS.md) - 实施状态

### 📖 完整索引
→ [docs/INDEX.md](docs/INDEX.md) - 所有文档列表

## 🎯 常用任务

### 在 R 中读取 HDF5 文件
```bash
Rscript docs/r_scripts/read_h5_with_new_names.R /path/to/assays.h5
```

### 验证 HDF5 文件结构
```bash
Rscript docs/r_scripts/verify_h5_structure.R /path/to/assays.h5
```

### 构建项目
```bash
cargo build --release
```

### 运行测试
```bash
cargo test
```

## 📊 项目结构

```
methrix-cli/
├── docs/              # 📚 所有文档
│   ├── INDEX.md      # 从这里开始
│   └── r_scripts/    # R 脚本
├── archived_scripts/  # 🗄️ 归档脚本
└── src/              # 💻 源代码
```

## 🔗 重要链接

- **完整文档**: [docs/INDEX.md](docs/INDEX.md)
- **项目结构**: [PROJECT_STRUCTURE.md](PROJECT_STRUCTURE.md)
- **清理报告**: [CLEANUP_REPORT.md](CLEANUP_REPORT.md)
- **R 脚本**: [docs/r_scripts/README.md](docs/r_scripts/README.md)

---

**提示**: 将此文件加入书签以便快速访问！
