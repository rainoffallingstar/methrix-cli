# Methrix CLI 设计文档

## 项目概述

Methrix CLI 是一个 Rust 原生的甲基化数据处理命令行工具，用于将 Bismark coverage 数据转换为版本化的 `methrix-cli.custom-hdf5` 格式，并生成 QC 与 CpG annotation 报告。

当前 HDF5 契约支持 R `rhdf5` 直接读取。它不是标准 `saveHDF5SummarizedExperiment()` 目录，不生成 `se.rds`，也不声明可由 `HDF5Array::loadHDF5SummarizedExperiment()` 或 `methrix::load_HDF5_methrix()` 直接加载。

## 设计目标

1. 主处理流程不依赖 R 或 Bioconductor。
2. 冻结并验证 `methrix-cli.custom-hdf5/1.0.0` schema。
3. 使用明确、可检查的坐标与 beta/coverage 缺失值契约。
4. 控制 HDF5 写入和样本处理的临时内存峰值。
5. 将 HDF5、QC 和 annotation 作为可回滚事务发布。
6. 对内置参考基因组执行固定来源、checksum 和 cache provenance 验证。
7. 避免 Excel worksheet 行数限制影响 WGBS annotation。

## 架构

```text
CLI (main.rs, cli/)
        |
        v
Reference genome (genome/)
  FASTA extraction / pinned download / RON cache
        |
        v
Bismark parsing and alignment (bismark/, cli/process.rs)
        |
        v
Filtering and statistics (processing/)
        |
        v
Staging and native validation
        |
        v
Transactional publication
  HDF5 + QC Excel + annotation Excel/TSV.gz
```

### 主要模块

| 模块 | 职责 |
|---|---|
| `src/genome/cpg.rs` | FASTA CpG 提取、RON 序列化、checked contig length conversion |
| `src/genome/download.rs` | UCSC 固定 release 下载、MD5、大小限制、provenance、cache 验证 |
| `src/bismark/reader.rs` | 标准六列 Bismark coverage 解析与输入契约验证 |
| `src/cli/process.rs` | 样本对齐、线程池、过滤、报告 staging 与发布 |
| `src/processing/filter.rs` | uncovered locus 过滤 |
| `src/processing/stats.rs` | coverage 单 pass 统计 |
| `src/hdf5/se_compat.rs` | custom HDF5 分块写入 |
| `src/hdf5/validate.rs` | Rust-native schema 与数值 readback validator |
| `src/annotation/mod.rs` | GTF annotation、qctb 汇总 workbook、gzip TSV 明细 |
| `src/atomic_output.rs` | 同目录 staging、backup、rollback 与 stale-output removal |

## 坐标契约

- Bismark coverage 输入使用 1-based 单碱基坐标，要求 `end == start`。
- 内部 `CpGSite` 使用 0-based start 和 end-exclusive interval。
- Bismark start 在解析时减 1，与参考 CpG start 对齐。
- HDF5 `rowData/start` 和 `rowData/end` 使用 1-based closed coordinates。
- HDF5 `width` 必须满足 `end - start + 1`，所有转换使用 checked arithmetic。
- FASTA contig length 必须能转换为 `u32`，超限时直接失败。

## HDF5 Schema

主产物为 `assays.h5`；`methrix_data.h5` 是相同字节内容的文件名 alias。

```text
/
├── beta                      # chunked f32 [sample, CpG]
├── cov                       # chunked u32 [sample, CpG]
├── rowData/
│   ├── chr
│   ├── seqnames
│   ├── start
│   ├── end
│   ├── width
│   └── strand
├── colData/
│   ├── sample_id
│   └── sample_name
└── metadata/
    ├── genome
    ├── schema_name
    ├── schema_version
    ├── loader_compatibility
    └── is_h5
```

固定 metadata：

```text
schema_name = methrix-cli.custom-hdf5
schema_version = 1.0.0
loader_compatibility = rhdf5 direct schema access only; standard HDF5Array/methrix loaders unsupported
```

### 数值契约

- beta 必须是 NaN 或有限的 `[0, 1]` 值。
- coverage 使用 `u32`，不允许 `u16` 截断。
- schema v1 冻结缺失值关系：`cov == 0` 当且仅当 beta 是 NaN。
- assay 必须 rank 2、shape 相同、维度非零并使用 chunked storage。
- row/column metadata 的类型和长度必须与 assay shape 一致。
- sample ID 必须非空、无首尾空格、无 tab/newline、无重复，并与 sample name 相同。

`validate_custom_hdf5()` 按固定大小块读取 assay 和 row metadata，不将完整 assay 载入内存。staged HDF5 只有通过该 validator 后才能发布。

## 内存与并发

- 最终 beta/cov matrix 预分配。
- 每个 Rayon worker 处理一个样本并直接写入最终 matrix column。
- 活跃 per-sample 临时向量数量受 `--threads` 限制。
- HDF5 writer 以约 1 MiB 的 CpG block 写入 `[sample, CpG]` dataset，不创建完整转置副本。
- coverage stats 单 pass 累计 covered count、sum 和 threshold counts。
- 所有 CpG 都 covered 时，filter 直接返回原 matrix。

当前 `BismarkReader` 仍会为每个活跃 worker 将单个样本记录读入 `Vec<BismarkRecord>`；因此线程数仍直接影响解析期峰值内存。

## Annotation 契约

`methrix process` 默认发布两个 annotation 产物：

- `CpG_annotation_report.xlsx`：仅包含 `ChIPseeker_By_Sample` 汇总。
- `CpG_annotation_details.tsv.gz`：逐 CpG GTF annotation 明细。

qctb 所需的列固定优先输出：

1. `Promoter`
2. `Exon`
3. `Intron`
4. `Intergenic`

零计数类别仍保留 count/percent 列，其他类别按字典序追加。逐 CpG 明细不写入 Excel，因此不受 1,048,576 行 worksheet 上限影响。`--skip-annotation` 会在同一事务中移除两个 stale annotation 文件。

## Genome 下载与 Provenance

内置 release 为 `hg19`、`hg38`、`mm10` 和 `mm39`。每个 release 固定 UCSC HTTPS URL 和官方 compressed-source MD5。

下载流程：

```text
HTTP bounded stream
  -> compressed payload MD5 and byte count
  -> gzip decompression with output size limit
  -> FASTA MD5 and byte count
  -> stage FASTA and provenance RON
  -> transactional publication
```

provenance 记录 schema、release、source URL、source MD5/bytes 和 FASTA MD5/bytes。cache 仅在 provenance 与固定 manifest 匹配，且 FASTA size/hash 重新验证通过后复用。

## 事务发布

`AtomicOutputSet` 要求所有 target 位于同一 output directory：

1. 每个产物写入同目录临时文件并 `sync_all`。
2. HDF5 在 staging 阶段执行 native validation。
3. 发布前把旧 target 移入临时 backup directory。
4. 依次 rename staged outputs。
5. 任一 rename 失败时删除已发布新文件并恢复全部 backups。
6. stale removal 与 replacement 属于同一事务。

测试覆盖 staging failure、发布中途 failure rollback、replacement 和 stale-output removal。

## 测试与门禁

```bash
cargo fmt --all -- --check
cargo check --all-targets --all-features --locked
cargo clippy --all-targets --all-features --locked -- -D warnings
cargo test --all-targets --all-features --locked
cargo build --all-targets --all-features --locked
```

测试范围包括：

- Bismark 输入边界与 overflow。
- 单线程/多线程结果等价。
- malformed HDF5 datatype、rank、shape、metadata、strand、sample ID 和 assay 值。
- transaction staging/publish rollback 与 stale removal。
- annotation 固定列和 gzip TSV readback。
- genome manifest、download size limit 和 cache tamper detection。
- 通过真实 `methrix process` 二进制运行的最小 FASTA/Bismark integration test。
- CI 中的 R `rhdf5` direct-schema smoke test。

## 已知限制

- 不支持标准 HDF5Array 或 methrix loader 直接加载。
- Bismark reader 尚未实现逐行直接写入最终 matrix。
- `download-genome` 仅在构建时启用 `download` feature 后可用。
- 定量速度和内存收益必须通过代表性 RRBS/WGBS benchmark 验证，不应在无数据时声明固定百分比。
