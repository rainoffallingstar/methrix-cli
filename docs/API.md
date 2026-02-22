# Methrix CLI API 文档

## 模块索引

### `methrix_cli`
主库入口，重新导出核心功能。

#### 重新导出
- `CpGData` - CpG 数据结构
- `CpGExtractor` - CpG 提取器
- `CpGSite` - 单个 CpG 位点
- `ContigInfo` - Contig 信息
- `load_cpg_data` - 加载预提取的 CpG 数据
- `download_genome` - 下载参考基因组

### `cli::process`
主处理流程模块。

#### 函数
- `run_pipeline()` - 运行完整的处理流程
- `find_bismark_files()` - 查找 Bismark 文件

#### 数据结构
- `MethrixData` - 甲基化数据容器

### `genome::cpg`
参考基因组和 CpG 提取模块。

#### 结构体
- `CpGSite` - 单个 CpG 位点
  ```rust
  pub struct CpGSite {
      pub chr: String,    // 染色体名称
      pub start: u32,     // 起始位置（0-based）
      pub end: u32,       // 结束位置
      pub strand: char,   // 链（'+', '-', '*'）
  }
  ```

- `CpGData` - CpG 数据集合
  ```rust
  pub struct CpGData {
      pub cpgs: Vec<CpGSite>,           // 所有 CpG 位点
      pub contig_lens: Vec<ContigInfo>, // Contig 长度信息
      pub release_name: String,         // 基因组版本
  }
  ```

- `ContigInfo` - Contig 信息
  ```rust
  pub struct ContigInfo {
      pub contig: String,  // Contig 名称
      pub length: u32,    // 长度
  }
  ```

#### 类
- `CpGExtractor` - CpG 提取器
  ```rust
  impl CpGExtractor {
      pub fn new(fasta_path: String) -> Self
      pub fn contigs(self, contigs: Vec<String>) -> Self
      pub fn extract(&self) -> Result<CpGData>
      pub fn save(&self, output_path: &str) -> Result<()>
  }
  ```

#### 函数
- `extract_and_save()` - 提取并保存 CpG 数据（便捷函数）
- `load_cpg_data()` - 加载预提取的 CpG 数据

### `genome::download`
参考基因组下载模块。

#### 函数
- `download_genome(genome: &str, output_dir: &str) -> Result<String>`  
  下载参考基因组
  
  **参数**:
  - `genome`: 基因组名称（hg19, hg38, mm10, mm39）
  - `output_dir`: 输出目录

  **返回**: 下载的 FASTA 文件路径

### `bismark::reader`
Bismark 文件读取模块。

#### 结构体
- `BismarkRecord` - Bismark 记录
  ```rust
  pub struct BismarkRecord {
      pub chr: String,              // 染色体
      pub start: u32,               // 起始位置（0-based）
      pub methylated_reads: u32,    // 甲基化读段数
      pub unmethylated_reads: u32,  // 未甲基化读段数
  }
  ```

  **方法**:
  - `total_reads()` -> u32 - 总读段数
  - `beta_value()` -> Option<f32> - 甲基化水平

#### 类
- `BismarkReader` - Bismark 文件读取器
  ```rust
  impl BismarkReader {
      pub fn new(file_path: String) -> Self
      pub fn read(&self) -> Result<Vec<BismarkRecord>>
  }
  ```

### `processing::filter`
数据过滤模块。

#### 函数
- `remove_uncovered(beta_matrix, cov_matrix)`  
  移除所有样本中都未覆盖的位点
  
  **参数**:
  - `beta_matrix`: 甲基化矩阵
  - `cov_matrix`: 覆盖度矩阵
  
  **返回**: 过滤后的矩阵对

- `coverage_filter(cov_matrix, cov_thr, min_samples)`  
  基于覆盖度过滤
  
  **参数**:
  - `cov_matrix`: 覆盖度矩阵
  - `cov_thr`: 最小覆盖度阈值
  - `min_samples`: 最小样本数
  
  **返回**: 布尔向量（保留的位点）

### `processing::stats`
统计计算模块。

#### 结构体
- `SampleStats` - 样本统计信息
  ```rust
  pub struct SampleStats {
      pub sample_name: String,
      pub n_covered: usize,                    // 覆盖的 CpG 数量
      pub n_total: usize,                      # 总 CpG 数量
      pub mean_coverage: f32,                  # 平均覆盖度
      pub coverage_distribution: Vec<(u16, usize)>, // (阈值, 数量)
  }
  ```

#### 函数
- `calculate_coverage_stats(cov_matrix, sample_names)`  
  计算覆盖度统计
  
  **参数**:
  - `cov_matrix`: 覆盖度矩阵
  - `sample_names`: 样本名称列表
  
  **返回**: `Vec<SampleStats>`

### `hdf5::se_compat`
HDF5 SummarizedExperiment 兼容写入模块。

#### 类
- `SummarizedExperimentWriter` - H5 文件写入器
  ```rust
  impl SummarizedExperimentWriter {
      pub fn new(output_path: String) -> Self
      pub fn write_methrix_object(&self, methrix_data: &MethrixData) -> Result<()>
  }
  ```

**H5 文件结构**:
```
methrix_data.h5
├── assays/
│   ├── beta          # f32 矩阵，甲基化值
│   └── cov           # u16 矩阵，覆盖度
├── rowData/
│   ├── chr           # 字符串数组，染色体
│   ├── start         # u32 数组，起始位置（0-based）
│   ├── end           # u32 数组，结束位置
│   └── strand        # 字符串数组，链（'+'）
├── colData/
│   └── sample_id     # 字符串数组，样本名称
└── metadata/
    ├── genome        # 标量数据集，参考基因组名称
    └── is_h5         # 标量数据集，HDF5 格式标志
```

### `qc::report`
质量控制报告模块。

#### 函数
- `generate_coverage_report(output_path, sample_stats)`  
  生成覆盖度统计报告
  
  **参数**:
  - `output_path`: 输出 Excel 文件路径
  - `sample_stats`: 样本统计信息

- `generate_qc_report(input_dir, output_path)`  
  从 H5 文件生成 QC 报告
  
  **参数**:
  - `input_dir`: H5 文件所在目录
  - `output_path`: 输出 Excel 文件路径

## 使用示例

### 示例 1：提取 CpG 位点

```rust
use methrix_cli::genome::cpg::CpGExtractor;

// 创建提取器
let extractor = CpGExtractor::new("hg19.fa".to_string());

// 提取 CpG
let cpg_data = extractor.extract()?;

// 保存为 RON 格式
extractor.save("hg19_cpgs.ron")?;
```

### 示例 2：读取 Bismark 文件

```rust
use methrix_cli::bismark::BismarkReader;

// 创建读取器
let reader = BismarkReader::new("sample.bismark.cov.gz".to_string());

// 读取文件
let records = reader.read()?;

// 处理记录
for record in records {
    println!("{}: {} methylation reads", record.chr, record.beta_value());
}
```

### 示例 3：数据处理流程

```rust
use methrix_cli::cli::process::run_pipeline;

// 运行完整流程
run_pipeline(
    "bismark_output/".to_string(),
    "results/".to_string(),
    "hg19.fa".to_string(),
    8,  // 线程数
    1,  // 最小覆盖度
    true,  // 移除未覆盖位点
)?;
```

## 错误处理

所有函数返回 `Result<T>` 以支持错误传播：

```rust
use anyhow::{Context, Result};

fn process_data() -> Result<()> {
    let file = File::open("data.txt")
        .context("Failed to open data file")?;
    
    let data = read_data(&file)
        .context("Failed to read data")?;
    
    Ok(())
}
```

## 类型转换

### CpG 坐标

**内部表示**: 0-based（Rust 标准）  
**Bismark 格式**: 1-based  
**H5 输出**: 0-based（与 R 一致）

转换自动处理：
- 读取 Bismark 文件时：1-based → 0-based
- 输出到 H5 时：0-based（R 标准）

## 并发处理

使用 `rayon` 进行数据并行：

```rust
use rayon::prelude::*;

// 并行处理多个文件
let results: Vec<Result<ProcessedSample>> = files
    .par_iter()  // 并行迭代
    .map(|file| process_file(file))
    .collect();
```

## 序列化

### RON 格式

CpG 数据使用 RON (Rusty Object Notation) 格式序列化：

```rust
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct CpGData {
    pub cpgs: Vec<CpGSite>,
    pub contig_lens: Vec<ContigInfo>,
    pub release_name: String,
}

// 序列化
let ron_string = ron::to_string_pretty(&cpg_data, Default::Default)?;

// 反序列化
let cpg_data: CpGData = ron::from_str(&ron_string)?;
```

## 性能考虑

### 内存使用

- **u16 vs u32**: 覆盖度使用 u16 节省 50% 内存
- **f32 vs f64**: 甲基化值使用 f32 节省 50% 内存
- **内存映射**: 大文件避免完全加载到内存

### 并发

- **可配置线程数**: 通过 `--threads` 参数
- **数据并行**: 矩阵运算自动并行
- **零拷贝**: 尽可能使用引用

### I/O 优化

- **批量写入**: HDF5 批量写入
- **压缩存储**: GZIP 级别 6
- **流式处理**: 增量读取大文件

## 平台兼容性

### 支持的平台

- **Linux**: x86_64, ARM64
- **macOS**: x86_64, ARM64 (Apple Silicon)
- **Windows**: x86_64

### 系统要求

- Rust 1.75+
- HDF5 库（libhdf5-dev）
- 对于下载功能：网络连接

## 测试

### 运行测试

```bash
# 所有测试
cargo test

# 特定测试
cargo test test_cpg_extraction

# 并行测试
cargo test -- --test-threads=4

# 显示输出
cargo test -- --nocapture
```

### 基准测试

```bash
# 运行基准测试
cargo bench

# 特定基准
cargo bench -- benchmark_processing
```

## 贡献

### 代码规范

- 遵循 Rust 命名规范
- 使用 `cargo fmt` 格式化代码
- 使用 `cargo clippy` 检查代码
- 为公共 API 添加文档注释

### 提交流程

1. Fork 仓库
2. 创建功能分支
3. 实现功能并添加测试
4. 确保所有测试通过
5. 提交 Pull Request

## 许可证

MIT License - 详见 LICENSE 文件
