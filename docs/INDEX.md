# Methrix CLI 文档索引

本目录包含 Methrix CLI 项目的完整文档。

## 📚 文档列表

### 用户文档

| 文档 | 描述 | 适合人群 |
|------|------|----------|
| [README.md](../README.md) | 项目概述、安装指南、基本使用 | 所有用户 |
| [QUICKSTART.md](QUICKSTART.md) | 快速开始指南 | 新用户 |
| [BUILD.md](BUILD.md) | 从源码构建指南 | 开发者、打包者 |
| [HDF5_DEPENDENCY.md](HDF5_DEPENDENCY.md) | HDF5 依赖说明 | 开发者、打包者 |
| [GIT_WORKFLOW.md](GIT_WORKFLOW.md) | Git 仓库和工作流说明 | 开发者、贡献者 |

### 设计文档

| 文档 | 描述 | 适合人群 |
|------|------|----------|
| [DESIGN.md](DESIGN.md) | 架构设计、技术选型、模块设计 | 架构师、开发者 |
| [API.md](API.md) | 完整 API 文档 | 开发者、贡献者 |
| [STATUS.md](STATUS.md) | 实施状态报告 | 项目管理者、用户 |
| [IMPLEMENTATION.md](IMPLEMENTATION.md) | 实现细节和技术决策 | 开发者 |

### HDF5 和 R 兼容性文档

| 文档 | 描述 | 适合人群 |
|------|------|----------|
| [HDF5_STRUCTURE_AND_COORDINATES.md](HDF5_STRUCTURE_AND_COORDINATES.md) | HDF5 文件结构和坐标系统详解 | 所有用户 |
| [HDF5_STRUCTURE_VISUAL.md](HDF5_STRUCTURE_VISUAL.md) | HDF5 结构可视化说明 | 所有用户 |
| [BETA_COV_NAMING_UPDATE.md](BETA_COV_NAMING_UPDATE.md) | Beta/Cov 命名更新说明 | 开发者、用户 |
| [HOW_TO_LOAD_METHRIX_OBJECT.md](HOW_TO_LOAD_METHRIX_OBJECT.md) | 如何加载 methrix 对象 | R 用户 |
| [R_METHRIX_COMPATIBILITY_GUIDE.md](R_METHRIX_COMPATIBILITY_GUIDE.md) | R methrix 包兼容性指南 | R 用户 |
| [LOAD_HDF5_METHRIX_SUPPORT_REPORT.md](LOAD_HDF5_METHRIX_SUPPORT_REPORT.md) | HDF5 加载支持报告 | 开发者 |
| [HDF5_R_COMPATIBILITY_REPORT.md](HDF5_R_COMPATIBILITY_REPORT.md) | HDF5 R 兼容性报告 | 开发者 |
| [QUICK_START_LOADING.md](QUICK_START_LOADING.md) | 快速开始加载数据 | R 用户 |

### 状态和测试文档

| 文档 | 描述 | 适合人群 |
|------|------|----------|
| [BUILD_STATUS.md](BUILD_STATUS.md) | 构建状态报告 | 项目管理者 |
| [DEBUG_REPORT.md](DEBUG_REPORT.md) | 调试报告 | 开发者 |
| [JOB_STATUS.md](JOB_STATUS.md) | 任务状态报告 | 项目管理者 |
| [QC_COVERAGE_REPORT.md](QC_COVERAGE_REPORT.md) | 质量控制和覆盖度报告 | 所有用户 |
| [TESTING_QUICK_REF.md](TESTING_QUICK_REF.md) | 测试快速参考 | 开发者、QA |

### 开发和部署文档

| 文档 | 描述 | 适合人群 |
|------|------|----------|
| [ROADMAP.md](ROADMAP.md) | 开发路线图 | 贡献者、规划者 |
| [SLURM_TEST_GUIDE.md](SLURM_TEST_GUIDE.md) | SLURM 集群测试指南 | HPC 用户 |

### Shell 脚本

#### 构建和测试脚本 ([scripts/](../scripts/))

| 脚本 | 描述 | 使用场景 |
|------|------|----------|
| [build.sh](../scripts/build.sh) | 使用 conda 环境构建项目 | 日常开发 |
| [test_real_data.sh](../scripts/test_real_data.sh) | 使用真实数据测试 | 功能测试 |
| [test_workflow.sh](../scripts/test_workflow.sh) | 测试完整处理流程 | 集成测试 |

#### SLURM 集群脚本 ([slurm_scripts/](../slurm_scripts/))

| 脚本 | 描述 | 使用场景 |
|------|------|----------|
| [submit_test.sh](../slurm_scripts/submit_test.sh) | 交互式提交 SLURM 任务 | HPC 测试 |
| [monitor_job.sh](../slurm_scripts/monitor_job.sh) | 实时监控任务状态 | 任务监控 |

### R 脚本

| 脚本 | 描述 | 使用场景 |
|------|------|----------|
| [r_scripts/read_h5_with_new_names.R](r_scripts/read_h5_with_new_names.R) | 读取使用新命名的 HDF5 文件 | 日常数据分析 |
| [r_scripts/verify_h5_structure.R](r_scripts/verify_h5_structure.R) | 验证 HDF5 文件结构 | 调试和验证 |
| [r_scripts/visualize_h5_layers.R](r_scripts/visualize_h5_layers.R) | 可视化 HDF5 层级结构 | 理解文件结构 |
| [r_scripts/h5_coordinate_analysis.R](r_scripts/h5_coordinate_analysis.R) | 分析 HDF5 坐标信息 | 坐标系统验证 |
| [r_scripts/analyze_h5_simple.R](r_scripts/analyze_h5_simple.R) | 简化的 HDF5 分析 | 快速检查数据 |
| [r_scripts/create_se_rds_v2.R](r_scripts/create_se_rds_v2.R) | 创建 SummarizedExperiment RDS 对象 | 数据导出 |
| [r_scripts/verify_r_compatibility.R](r_scripts/verify_r_compatibility.R) | 验证 R 兼容性 | 兼容性测试 |

## 🎯 快速导航

### 我想...

#### ...开始使用 Methrix CLI
→ 阅读 [QUICKSTART.md](QUICKSTART.md)

#### ...了解 HDF5 文件结构
→ 阅读 [HDF5_STRUCTURE_AND_COORDINATES.md](HDF5_STRUCTURE_AND_COORDINATES.md)
→ 查看 [HDF5_STRUCTURE_VISUAL.md](HDF5_STRUCTURE_VISUAL.md)

#### ...在 R 中加载数据
→ 阅读 [QUICK_START_LOADING.md](QUICK_START_LOADING.md)
→ 阅读 [HOW_TO_LOAD_METHRIX_OBJECT.md](HOW_TO_LOAD_METHRIX_OBJECT.md)
→ 运行 [r_scripts/read_h5_with_new_names.R](r_scripts/read_h5_with_new_names.R)

#### ...验证 HDF5 文件
→ 运行 [r_scripts/verify_h5_structure.R](r_scripts/verify_h5_structure.R)

#### ...了解 R 兼容性
→ 阅读 [R_METHRIX_COMPATIBILITY_GUIDE.md](R_METHRIX_COMPATIBILITY_GUIDE.md)
→ 阅读 [HDF5_R_COMPATIBILITY_REPORT.md](HDF5_R_COMPATIBILITY_REPORT.md)

#### ...了解如何构建
→ 阅读 [BUILD.md](BUILD.md)

#### ...理解架构设计
→ 阅读 [DESIGN.md](DESIGN.md)

#### ...查看 API 文档
→ 阅读 [API.md](API.md)

#### ...查看当前进度
→ 阅读 [STATUS.md](STATUS.md)

#### ...了解未来计划
→ 阅读 [ROADMAP.md](ROADMAP.md)

## 📖 文档结构

```
docs/
├── 用户文档
│   ├── QUICKSTART.md           # 快速开始
│   ├── BUILD.md                # 构建指南
│   └── README.md               # 项目概述
│
├── 设计文档
│   ├── DESIGN.md               # 架构设计
│   ├── API.md                  # API 文档
│   ├── STATUS.md               # 实施状态
│   └── IMPLEMENTATION.md       # 实现细节
│
├── HDF5 和 R 兼容性
│   ├── HDF5_STRUCTURE_AND_COORDINATES.md      # HDF5 结构详解
│   ├── HDF5_STRUCTURE_VISUAL.md               # HDF5 可视化
│   ├── BETA_COV_NAMING_UPDATE.md              # 命名更新
│   ├── HOW_TO_LOAD_METHRIX_OBJECT.md          # 加载指南
│   ├── R_METHRIX_COMPATIBILITY_GUIDE.md       # R 兼容性
│   ├── LOAD_HDF5_METHRIX_SUPPORT_REPORT.md    # 加载支持报告
│   ├── HDF5_R_COMPATIBILITY_REPORT.md         # 兼容性报告
│   └── QUICK_START_LOADING.md                 # 快速加载
│
├── 状态和测试
│   ├── BUILD_STATUS.md         # 构建状态
│   ├── DEBUG_REPORT.md         # 调试报告
│   ├── JOB_STATUS.md           # 任务状态
│   ├── QC_COVERAGE_REPORT.md   # QC 报告
│   └── TESTING_QUICK_REF.md    # 测试参考
│
├── 开发和部署
│   ├── ROADMAP.md              # 开发路线图
│   └── SLURM_TEST_GUIDE.md     # SLURM 指南
│
└── r_scripts/                  # R 脚本
    ├── read_h5_with_new_names.R           # 读取 HDF5
    ├── verify_h5_structure.R              # 验证结构
    ├── visualize_h5_layers.R              # 可视化
    ├── h5_coordinate_analysis.R           # 坐标分析
    ├── analyze_h5_simple.R                # 简单分析
    ├── create_se_rds_v2.R                 # 创建 SE 对象
    └── verify_r_compatibility.R           # 兼容性验证
```

## 📝 文档维护

### 文档更新原则

1. **及时更新**：代码变更时同步更新文档
2. **准确性**：确保所有示例和命令都经过验证
3. **完整性**：涵盖所有公共 API 和重要功能
4. **可读性**：使用清晰的格式和适当的示例

### 文档格式

- Markdown 格式
- 代码块使用语法高亮
- 表格用于结构化信息
- 目录用于长文档导航

### 贡献文档

欢迎改进文档！请：

1. Fork 仓库
2. 编辑文档
3. 确保构建和测试通过
4. 提交 Pull Request

## 🔗 相关资源

### 项目资源

- **Rust methrix-cli**: https://github.com/CompEpigen/methrix
- **R methrix 包**: https://bioconductor.org/packages/release/bioc/html/methrix/
- **Issue 跟踪**: https://github.com/CompEpigen/methrix/issues
- **Discussions**: https://github.com/CompEpigen/methrix/discussions

### 外部资源

- **Rust 文档**: https://www.rust-lang.org/
- **Clap 文档**: https://docs.rs/clap/
- **HDF5 文档**: https://www.hdfgroup.org/
- **Bioconductor**: https://www.bioconductor.org/
- **SummarizedExperiment**: https://bioconductor.org/packages/release/bioc/html/SummarizedExperiment/

## 💡 获取帮助

### 报告问题

如果您遇到问题：

1. 查看相关文档
2. 搜索已有 Issues
3. 创建新 Issue 并包含：
   - 操作系统和版本
   - 复现步骤
   - 错误信息
   - 预期行为

### 功能请求

欢迎提出功能建议！

请在 Issue 中说明：
- 用例场景
- 期望行为
- 与现有功能的关系

---

**文档版本**: 2.0.0
**最后更新**: 2026-02-22
**主要变更**: 添加 HDF5 和 R 兼容性文档，重组文档结构
