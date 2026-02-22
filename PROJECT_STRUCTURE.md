# Methrix CLI 项目结构

## 📁 目录组织

```
methrix-cli/
├── src/                      # Rust 源代码
│   ├── main.rs              # 程序入口
│   ├── lib.rs               # 库导出
│   ├── cli/                 # CLI 命令实现
│   ├── genome/              # 参考基因组处理
│   ├── bismark/             # Bismark 文件处理
│   ├── processing/          # 核心处理逻辑
│   ├── hdf5/                # HDF5 I/O
│   └── qc/                  # 质量控制
│
├── docs/                     # 📚 项目文档
│   ├── INDEX.md             # 📖 文档索引 (从这里开始)
│   ├── QUICKSTART.md        # 🚀 快速开始
│   ├── BUILD.md             # 🔨 构建指南
│   ├── DESIGN.md            # 🏗️ 架构设计
│   ├── API.md               # 📡 API 文档
│   ├── STATUS.md            # 📊 实施状态
│   ├── ROADMAP.md           # 🗺️ 开发路线图
│   │
│   ├── HDF5 和 R 兼容性文档
│   │   ├── HDF5_STRUCTURE_AND_COORDINATES.md    # HDF5 结构详解
│   │   ├── HDF5_STRUCTURE_VISUAL.md             # HDF5 可视化
│   │   ├── BETA_COV_NAMING_UPDATE.md            # 命名更新
│   │   ├── HOW_TO_LOAD_METHRIX_OBJECT.md        # 加载指南
│   │   ├── R_METHRIX_COMPATIBILITY_GUIDE.md     # R 兼容性
│   │   ├── LOAD_HDF5_METHRIX_SUPPORT_REPORT.md  # 加载支持
│   │   ├── HDF5_R_COMPATIBILITY_REPORT.md       # 兼容性报告
│   │   └── QUICK_START_LOADING.md               # 快速加载
│   │
│   ├── 状态和测试文档
│   │   ├── BUILD_STATUS.md       # 构建状态
│   │   ├── DEBUG_REPORT.md       # 调试报告
│   │   ├── JOB_STATUS.md         # 任务状态
│   │   ├── QC_COVERAGE_REPORT.md # QC 报告
│   │   └── TESTING_QUICK_REF.md  # 测试参考
│   │
│   ├── R 脚本
│   │   └── r_scripts/           # 🔬 R 分析脚本
│   │       ├── README.md
│   │       ├── read_h5_with_new_names.R
│   │       ├── verify_h5_structure.R
│   │       ├── visualize_h5_layers.R
│   │       ├── h5_coordinate_analysis.R
│   │       ├── analyze_h5_simple.R
│   │       ├── create_se_rds_v2.R
│   │       └── verify_r_compatibility.R
│   │
│   └── 其他文档
│       ├── IMPLEMENTATION.md     # 实现细节
│       └── SLURM_TEST_GUIDE.md  # SLURM 指南
│
├── scripts/                  # 🔧 Shell 脚本
│   ├── README.md
│   ├── build.sh              # 构建脚本
│   ├── test_real_data.sh     # 真实数据测试
│   └── test_workflow.sh      # 工作流测试
│
├── slurm_scripts/            # 🖥️ SLURM 集群脚本
│   ├── README.md
│   ├── submit_test.sh        # 提交任务
│   └── monitor_job.sh        # 监控任务
│
├── archived_scripts/         # 🗄️ 归档的旧脚本
│   └── README.md            # 归档说明
│
├── tests/                   # 🧪 测试
│   └── integration/         # 集成测试
│
├── testdata/               # 📊 测试数据
│
├── CLAUDE.md               # 🤖 Claude Code 指导
├── README.md               # 📖 项目概述
└── Cargo.toml              # 📦 Rust 项目配置
```

## 🎯 快速导航

### 我想要...

#### ...开始使用 Methrix CLI
→ 阅读 [README.md](README.md)
→ 查看 [docs/QUICKSTART.md](docs/QUICKSTART.md)

#### ...构建项目
→ 运行 [scripts/build.sh](scripts/build.sh)
→ 阅读 [docs/BUILD.md](docs/BUILD.md)

#### ...运行测试
→ 运行 [scripts/test_workflow.sh](scripts/test_workflow.sh)
→ 阅读 [docs/TESTING_QUICK_REF.md](docs/TESTING_QUICK_REF.md)

#### ...在 HPC 集群上测试
→ 运行 [slurm_scripts/submit_test.sh](slurm_scripts/submit_test.sh)
→ 阅读 [docs/SLURM_TEST_GUIDE.md](docs/SLURM_TEST_GUIDE.md)

#### ...理解 HDF5 文件结构
→ 阅读 [docs/HDF5_STRUCTURE_AND_COORDINATES.md](docs/HDF5_STRUCTURE_AND_COORDINATES.md)
→ 查看 [docs/HDF5_STRUCTURE_VISUAL.md](docs/HDF5_STRUCTURE_VISUAL.md)

#### ...在 R 中加载数据
→ 阅读 [docs/QUICK_START_LOADING.md](docs/QUICK_START_LOADING.md)
→ 运行 [docs/r_scripts/read_h5_with_new_names.R](docs/r_scripts/read_h5_with_new_names.R)

#### ...验证 HDF5 文件
→ 运行 [docs/r_scripts/verify_h5_structure.R](docs/r_scripts/verify_h5_structure.R)

#### ...理解架构设计
→ 阅读 [docs/DESIGN.md](docs/DESIGN.md)
→ 查看 [docs/API.md](docs/API.md)

#### ...查看实施状态
→ 阅读 [docs/STATUS.md](docs/STATUS.md)

#### ...查看开发计划
→ 阅读 [docs/ROADMAP.md](docs/ROADMAP.md)

## 📚 文档分类

### 用户文档 (适合所有用户)
- [README.md](README.md) - 项目概述
- [docs/QUICKSTART.md](docs/QUICKSTART.md) - 快速开始
- [docs/BUILD.md](docs/BUILD.md) - 构建指南
- [docs/HDF5_STRUCTURE_AND_COORDINATES.md](docs/HDF5_STRUCTURE_AND_COORDINATES.md) - HDF5 结构
- [docs/QUICK_START_LOADING.md](docs/QUICK_START_LOADING.md) - 加载数据

### 开发者文档 (适合开发者)
- [docs/DESIGN.md](docs/DESIGN.md) - 架构设计
- [docs/API.md](docs/API.md) - API 文档
- [docs/STATUS.md](docs/STATUS.md) - 实施状态
- [docs/ROADMAP.md](docs/ROADMAP.md) - 开发路线图
- [CLAUDE.md](CLAUDE.md) - AI 辅助开发指导

### R 用户文档 (适合 R/Bioconductor 用户)
- [docs/R_METHRIX_COMPATIBILITY_GUIDE.md](docs/R_METHRIX_COMPATIBILITY_GUIDE.md) - R 兼容性
- [docs/HOW_TO_LOAD_METHRIX_OBJECT.md](docs/HOW_TO_LOAD_METHRIX_OBJECT.md) - 加载对象
- [docs/r_scripts/README.md](docs/r_scripts/README.md) - R 脚本使用指南

### HPC 用户文档 (适合集群用户)
- [docs/SLURM_TEST_GUIDE.md](docs/SLURM_TEST_GUIDE.md) - SLURM 测试指南
- [slurm_scripts/README.md](slurm_scripts/README.md) - SLURM 脚本使用

### 管理员文档 (适合项目管理者)
- [docs/BUILD_STATUS.md](docs/BUILD_STATUS.md) - 构建状态
- [docs/QC_COVERAGE_REPORT.md](docs/QC_COVERAGE_REPORT.md) - QC 报告
- [docs/TESTING_QUICK_REF.md](docs/TESTING_QUICK_REF.md) - 测试参考

## 🔧 常用命令

### 构建
```bash
# 使用脚本构建
./scripts/build.sh

# 或手动构建
cargo build --release
```

### 测试
```bash
# 本地测试
./scripts/test_workflow.sh

# 或使用 cargo
cargo test
```

### 运行
```bash
./target/release/methrix --help
./target/release/methrix process -i <input> -o <output> -g <genome>
```

### 集群测试
```bash
# 提交 SLURM 任务
./slurm_scripts/submit_test.sh

# 监控任务
./slurm_scripts/monitor_job.sh
```

### 验证 HDF5 文件 (R)
```bash
Rscript docs/r_scripts/verify_h5_structure.R /path/to/assays.h5
```

## 📊 文件统计

| 类别 | 数量 | 位置 |
|------|------|------|
| 核心文档 | 7 | `docs/` |
| HDF5/R 文档 | 8 | `docs/` |
| 状态/测试文档 | 5 | `docs/` |
| R 脚本 | 7 | `docs/r_scripts/` |
| Shell 脚本 | 3 | `scripts/` |
| SLURM 脚本 | 2 | `slurm_scripts/` |
| 归档脚本 | 22 | `archived_scripts/` |

## 🔄 最近更新

### 2026-02-22
- ✅ 更新 HDF5 数据集命名: `assay001/assay002` → `beta/cov`
- ✅ 整理文档结构，移动到 `docs/` 目录
- ✅ 归档旧的 R 脚本到 `archived_scripts/`
- ✅ 创建 R 脚本使用指南
- ✅ 整理 Shell 脚本到 `scripts/` 和 `slurm_scripts/`
- ✅ 更新文档索引

## 💡 贡献指南

### 添加新文档
1. 将文档放在 `docs/` 目录下
2. 更新 `docs/INDEX.md`
3. 在相关文档中添加链接

### 添加新 R 脚本
1. 将脚本放在 `docs/r_scripts/` 目录下
2. 更新 `docs/r_scripts/README.md`
3. 添加使用示例和依赖说明

### 添加新 Shell 脚本
1. 构建脚本 → `scripts/`
2. SLURM 脚本 → `slurm_scripts/`
3. 更新相应目录的 README

### 归档旧脚本
1. 移动到 `archived_scripts/`
2. 更新 `archived_scripts/README.md`
3. 在新脚本中说明替代关系

## 📞 获取帮助

- 📖 查看 [docs/INDEX.md](docs/INDEX.md) 获取完整文档列表
- 🔍 搜索已有 Issues
- 📝 创建新 Issue 并包含详细信息

---

**最后更新**: 2026-02-22
**项目版本**: 2.0.0 (beta/cov 命名)
