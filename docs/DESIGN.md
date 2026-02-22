# Methrix CLI 设计文档

## 项目概述

Methrix CLI 是一个高性能的甲基化数据处理命令行工具，用于将 Bismark 亚硫酸氢盐测序数据转换为与 R methrix 包兼容的 HDF5 格式。该工具作为原始 R 脚本的完整替代方案，提供了显著的性能改进，并且无需 R 运行时依赖。

## 设计目标

### 核心目标
1. **完全独立于 R 环境**：不需要用户安装 R 或 Bioconductor
2. **保持 H5 格式兼容**：生成的 H5 文件能被 methrix 包的 `load_HDF5_methrix()` 加载
3. **移植核心逻辑**：将 methrix 包的关键处理逻辑用 Rust 重写
4. **功能完整性**：支持完整的 Bismark 数据处理流程和质量控制

### 性能目标
- **5-10倍加速**：相比原始 R 实现
- **30-50% 内存减少**：优化内存使用
- **亚秒级启动**：快速启动时间
- **高效并发**：支持多线程处理

## 架构设计

### 整体架构

```
┌─────────────────────────────────────────────────┐
│         CLI 层 (main.rs, cli/)                 │
│  - 参数解析 (clap)                               │
│  - 命令路由                                       │
│  - 进度显示 (indicatif)                          │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│      参考基因组层 (genome/)                      │
│  - CpG 提取 (移植 R::extract_CPGs)               │
│  - FASTA 读取                                     │
│  - 基因组下载                                     │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│      数据处理层 (bismark/, processing/)          │
│  - Bismark 文件读取 (移植 R::read_bdg)          │
│  - 并发处理 (移植 R::vect_code_batch)            │
│  - 数据过滤 (移植 R::coverage_filter)            │
│  - 统计计算 (移植 R::get_stats)                  │
└─────────────────────────────────────────────────┘
                      ↓
┌─────────────────────────────────────────────────┐
│      输出层 (hdf5/, qc/)                         │
│  - H5 文件写入 (SE 兼容)                         │
│  - QC 报告生成                                    │
└─────────────────────────────────────────────────┘
```

### 技术选型

| 层级 | 技术选择 | 理由 |
|------|----------|------|
| **编程语言** | Rust | 性能、内存安全、并发支持 |
| **CLI 框架** | clap | 类型安全、易用 |
| **数据序列化** | serde + ron | 高效序列化 |
| **HDF5 I/O** | hdf5-rust | 成熟的 HDF5 绑定 |
| **并发** | rayon | 数据并行处理 |
| **矩阵运算** | ndarray | 科学计算 |
| **FASTA 处理** | needletail | 高效序列解析 |
| **Excel 输出** | rust_xlsxwriter | Excel 文件生成 |

## 模块设计

### 1. 参考基因组模块 (`genome/`)

#### CpG 提取 (`cpg.rs`)

**功能**：从参考基因组 FASTA 文件中提取所有 CpG 位点

**数据结构**：
```rust
pub struct CpGSite {
    pub chr: String,
    pub start: u32,  // 0-based
    pub end: u32,
    pub strand: char,
}

pub struct CpGData {
    pub cpgs: Vec<CpGSite>,
    pub contig_lens: Vec<ContigInfo>,
    pub release_name: String,
}
```

**核心算法**：
- 遍历 FASTA 文件的每条染色体
- 查找所有 "CG" 模式（等同于 R 的 `Biostrings::matchPattern("CG", ...)`）
- 默认只包含标准染色体（常染色体 + 性染色体）
- 输出为 RON 格式以便快速重载

**移植自 R**：
```r
# R 代码
cgs = lapply(chrs, function(x) start(Biostrings::matchPattern("CG", ref_genome[[x]])))
```

#### 基因组下载 (`download.rs`)

**功能**：从 UCSC 下载常用参考基因组

**支持的基因组**：
- hg19
- hg38
- mm10
- mm39

### 2. Bismark 文件处理模块 (`bismark/`)

#### 文件读取器 (`reader.rs`)

**功能**：高效读取 Bismark 输出文件

**数据结构**：
```rust
pub struct BismarkRecord {
    pub chr: String,
    pub start: u32,  // 0-based 内部表示
    pub methylated_reads: u32,
    pub unmethylated_reads: u32,
}
```

**优化技术**：
- 压缩文件：使用 `flate2` 解压
- 未压缩文件：使用 `memmap2` 内存映射
- 坐标转换：自动从 1-based 转换为 0-based

**移植自 R**：
```r
# R 代码 (accessory_funcs.R::read_bdg)
bdg_dat = data.table::fread(file = bdg, sep = "\t", ...)
```

### 3. 数据处理模块 (`processing/`)

#### 处理流程 (`pipeline.rs`)

**功能**：并发处理多个 Bismark 文件

**核心类**：
```rust
pub struct MethrixProcessor {
    cpg_data: CpGData,
    cpg_index: HashMap<(String, u32), usize>,
}
```

**处理流程**：
1. 建立 CpG 索引（类似 R 的 `data.table::setkey`）
2. 并发读取每个 Bismark 文件
3. 将读取结果对齐到参考 CpG 位点
4. 合并结果为矩阵

**移植自 R**：
```r
# R 代码 (accessory_funcs.R::vect_code_batch)
bdgs <- parallel::mclapply(batch_files, read_bdg, mc.cores = thr)
```

#### 过滤操作 (`filter.rs`)

**功能**：数据过滤

**实现的过滤**：
- `remove_uncovered()`: 移除所有样本中都未覆盖的位点
- `coverage_filter()`: 基于覆盖度过滤

**移植自 R**：
```r
# R 代码 (methrix_operations.R::remove_uncovered)
row_idx<-rowSums(!is.na(assays(m)[[2]]))==0
```

#### 统计计算 (`stats.rs`)

**功能**：计算覆盖度统计

**输出统计**：
- 样本名称
- 覆盖的 CpG 数量
- 总 CpG 数量
- 平均覆盖度
- 覆盖度分布 (1X, 2X, 3X, 4X, 5X, 10X)

**移植自 R**：
```r
# R 代码 (methrix_operations.R::get_stats)
mean_cov = DelayedMatrixStats::colMeans2(get_matrix(m = m, "C"), na.rm = TRUE)
```

### 4. HDF5 输出模块 (`hdf5/`)

#### SE 兼容写入器 (`se_compat.rs`)

**功能**：创建与 R SummarizedExperiment 兼容的 H5 文件

**H5 文件结构**：
```
methrix_data.h5
├── assays/
│   ├── beta          # 甲基化矩阵 (f32)
│   └── cov           # 覆盖度矩阵 (u16)
├── rowData/
│   ├── chr           # 染色体
│   ├── start         # 起始位置 (0-based)
│   ├── end           # 结束位置
│   └── strand        # 链 (+)
├── colData/
│   └── sample_id     # 样本名称
└── metadata/
    ├── genome        # 参考基因组名称
    └── is_h5         # HDF5 格式标志
```

**关键兼容性**：
- 使用 HDF5 Group 结构
- 列优先存储（与 R 一致）
- GZIP 压缩
- SE 特定属性（se_version, delayed_array_type）

### 5. QC 报告模块 (`qc/`)

#### 报告生成器 (`report.rs`)

**功能**：生成 Excel 格式的质量控制报告

**报告内容**：
- 样本名称
- 总 CpG 数量
- 覆盖的 CpG 数量
- 分层覆盖度统计 (1X, 2X, 3X, 4X, 5X, 10X)

## 命令行接口设计

### 主命令结构

```bash
methrix <command> [OPTIONS]

Commands:
  process        处理 Bismark 输出文件
  extract-cpgs   从参考基因组提取 CpG 位点
  download-genome 下载参考基因组
  qc-report       生成 QC 报告
```

### process 命令

```bash
methrix process [OPTIONS]

Required:
  -i, --input <DIR>      包含 *.bismark.cov.gz 文件的输入目录
  -o, --output <DIR>     H5 文件和报告的输出目录
  -g, --genome <GENOME>  参考基因组（FASTA 或预提取的 .ron 文件）

Optional:
  -t, --threads <N>      并行处理线程数 [默认: CPU 核心数]
      --min-coverage <N>  考虑覆盖的最小覆盖度 [默认: 1]
      --remove-uncovered 移除未覆盖的位点 [默认: true]
  -v, --verbose          启用详细日志记录
```

### extract-cpgs 命令

```bash
methrix extract-cpgs [OPTIONS]

Required:
  -g, --genome <GENOME>  基因组 FASTA 文件或内置名称
  -o, --output <FILE>    CpG 数据的输出 RON 文件

Optional:
      --contigs <LIST>   要包含的 contigs [默认: 常染色体 + 性染色体]
  -v, --verbose          启用详细日志记录
```

### download-genome 命令

```bash
methrix download-genome [OPTIONS]

Required:
  -g, --genome <GENOME>  基因组名称: hg19, hg38, mm10, mm39
  -o, --output <DIR>     输出目录

Optional:
  -v, --verbose          启用详细日志记录
```

### qc-report 命令

```bash
methrix qc-report [OPTIONS]

Required:
  -i, --input <DIR>      包含 methrix H5 对象的输入目录
  -o, --output <FILE>    输出 Excel 文件

Optional:
  -v, --verbose          启用详细日志记录
```

## 数据流程

### 标准工作流程

```
1. Bismark 输出文件
   sample1.bismark.cov.gz
   sample2.bismark.cov.gz
   sample3.bismark.cov.gz
   ↓
2. 参考基因组处理
   hg19.fa → CpG 提取 → hg19_cpgs.ron
   ↓
3. 并发处理
   ┌─ 读取 Bismark 文件
   ├─ 对齐到参考 CpG
   ├─ 创建矩阵
   └─ 移除未覆盖位点
   ↓
4. 输出生成
   ├── methrix_data.h5
   └── CpG_coverage.xlsx
```

### 优化工作流程

```
1. 预提取 CpG (一次性)
   methrix extract-cpgs --genome hg19.fa --output hg19_cpgs.ron

2. 多次使用预提取数据
   methrix process --input batch1/ --genome hg19_cpgs.ron ...
   methrix process --input batch2/ --genome hg19_cpgs.ron ...
   methrix process --input batch3/ --genome hg19_cpgs.ron ...
```

## 性能优化策略

### 1. 内存优化

**内存映射**：
- 大文件使用 `memmap2` 避免完全加载到内存
- 零拷贝读取

**数据结构**：
- 使用 `u16` 而非 `u32` 存储覆盖度（节省 50% 内存）
- 使用 `f32` 而非 `f64` 存储甲基化值（节省 50% 内存）

**HDF5 优化**：
- 分块写入
- GZIP 压缩（级别 6）

### 2. 并发优化

**任务并行**：
- 使用 `rayon` 的 `par_iter` 并行处理多个文件
- 可配置线程池大小

**数据并行**：
- 矩阵运算使用 `ndarray` 的并行功能

### 3. I/O 优化

**流式处理**：
- 增量读取大文件
- 避免不必要的缓冲

**批量写入**：
- HDF5 批量写入
- 原子文件操作

## 错误处理

### 错误类型

```rust
pub enum MethrixError {
    IoError(std::io::Error),
    ParseError(String),
    ValidationError(String),
    Hdf5Error(String),
    GenomeError(String),
}
```

### 错误传播

使用 `anyhow` 进行上下文丰富的错误处理：

```rust
let file = File::open(&path)
    .context("Failed to open input file")?;
```

## 测试策略

### 单元测试

- CpG 提取逻辑
- Bismark 文件解析
- 坐标转换
- 覆盖度统计

### 集成测试

- 完整处理流程
- H5 文件生成
- R 兼容性验证

### 基准测试

- CpG 提取性能
- Bismark 文件读取速度
- 整体处理时间

## 兼容性保证

### H5 格式兼容

确保生成的 H5 文件能被 R 的 `load_HDF5SummarizedExperiment()` 加载：

1. **Group 结构**：assays/, rowData/, colData/, metadata/
2. **数据类型**：beta (f32), cov (u16)
3. **属性**：se_version, delayed_array_type
4. **存储顺序**：列优先（与 R 一致）

### 结果一致性

与 R 实现的结果在合理误差范围内一致：
- 允许浮点精度差异
- 允许不同的 NA 处理顺序
- 保证统计意义一致

## 部署方案

### 二进制分发

1. **GitHub Releases**：
   - Linux (x86_64)
   - macOS (x86_64, ARM64)
   - Windows (x86_64)

2. **包管理器**：
   - Debian/Ubuntu: .deb 包
   - RedHat/CentOS: .rpm 包
   - Homebrew (macOS)
   - Scoop (Windows)

### Docker 镜像

```dockerfile
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libhdf5-serial-dev ca-certificates
COPY methrix /usr/local/bin/
ENTRYPOINT ["methrix"]
```

## 未来扩展

### Phase 2 功能

- 区域过滤
- SNP 掩盖
- 链特异性处理
- 批量处理
- 进度条

### Phase 3 功能

- 差异甲基化分析
- DMR 检测
- PCA 分析
- 聚类分析

### 长期目标

- 与 methrix R 包完全功能对等
- REST API
- Web 界面
- 云存储集成

## 开发工作流

### 代码风格

- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码质量
- 遵循 Rust 命名规范

### 提交流程

1. Fork 仓库
2. 创建功能分支
3. 编写代码和测试
4. 提交 Pull Request
5. 代码审查和合并

### 发布流程

1. 更新版本号
2. 更新 CHANGELOG
3. 运行测试套件
4. 创建 Git 标签
5. 构建发布二进制文件
6. 创建 GitHub Release

## 文档维护

### 用户文档

- README.md：概述和基本使用
- docs/QUICKSTART.md：快速开始指南
- docs/BUILD.md：构建说明

### 开发者文档

- docs/DESIGN.md：本文档
- docs/ROADMAP.md：开发路线图
- 代码内文档：rustdoc 注释

### 测试文档

- tests/integration/test_r_compatibility.R：R 兼容性测试
- scripts/generate_test_data.py：测试数据生成器
