# HDF5 依赖说明

## 📦 当前使用的 HDF5 实现

**本项目使用 Rust 原生的 HDF5 包** - `hdf5 = "0.8"`

这是一个 **纯 Rust 的 HDF5 绑定库**，而不是直接依赖系统的 HDF5 包。

## 🔍 依赖详情

### Cargo.toml 配置

```toml
[dependencies]
hdf5 = "0.8"
```

### 实际的依赖树

```
methrix-cli
└── hdf5 v0.8.1
    ├── hdf5-sys v0.8.1      # FFI 绑定到 C HDF5 库
    │   ├── libc            # C 库接口
    │   ├── libloading      # 动态加载库
    │   └── pkg-config      # 库配置工具
    ├── hdf5-types v0.8.1   # Rust 类型转换
    ├── hdf5-derive v0.8.1  # 过程宏，派生宏
    ├── ndarray v0.15       # 数组支持
    └── bitflags v1.3       # 位标志
```

## 🏗️ 架构说明

### Rust HDF5 包的工作原理

```
┌─────────────────────────────────────────┐
│         您的 Rust 代码                    │
│    (src/hdf5/se_compat.rs)               │
└─────────────┬───────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────┐
│      hdf5 crate (Rust)                   │
│  - 高级 Rust API                         │
│  - 类型安全的接口                         │
│  - ndarray 集成                          │
└─────────────┬───────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────┐
│     hdf5-sys (FFI 绑定)                  │
│  - unsafe Rust 函数                     │
│  - 直接调用 C HDF5 API                  │
└─────────────┬───────────────────────────┘
              │
              ↓
┌─────────────────────────────────────────┐
│    C HDF5 库 (系统库)                    │
│  libhdf5.so / libhdf5.dylib             │
└─────────────────────────────────────────┘
```

### 关键点

1. **`hdf5` crate** 是纯 Rust 实现，提供高级 API
2. **`hdf5-sys`** 提供 FFI（外部函数接口）绑定到底层 C HDF5 库
3. **仍需要系统 HDF5 库** 用于实际的 HDF5 文件读写

## 📋 系统依赖要求

虽然使用的是 Rust 的 HDF5 包，但**仍然需要系统级 HDF5 C 库**：

### Linux (Ubuntu/Debian)
```bash
sudo apt-get install libhdf5-dev
```

### macOS
```bash
brew install hdf5
```

### 使用 Conda (推荐)
```bash
conda install -c conda-forge hdf5
```

## 🔧 构建配置

### 环境变量

`hdf5-sys` 需要以下环境变量来定位系统 HDF5 库：

```bash
# 方法 1: 设置 HDF5_DIR
export HDF5_DIR=/path/to/hdf5

# 方法 2: 使用 pkg-config
export PKG_CONFIG_PATH=/path/to/hdf5/lib/pkgconfig

# 方法 3: 使用 Conda
conda activate rust_build
export HDF5_DIR=$CONDA_PREFIX
```

### Ubuntu/Debian 系统包注意事项

在 Ubuntu/Debian 上通过 `libhdf5-dev` 安装时，头文件通常位于 `/usr/include/hdf5/serial`，库位于 `/usr/lib/x86_64-linux-gnu/hdf5/serial`。

这种布局下**不要**设置 `HDF5_DIR=/usr`（`hdf5-sys` 会检查 `/usr/include` 并报头文件目录无效）。

请改用：

```bash
export HDF5_INCLUDE_DIR=/usr/include/hdf5/serial
export HDF5_LIB_DIR=/usr/lib/x86_64-linux-gnu/hdf5/serial
export PKG_CONFIG_PATH=/usr/lib/x86_64-linux-gnu/hdf5/serial/pkgconfig:${PKG_CONFIG_PATH:-}
```

### 验证 HDF5 安装

```bash
# 检查 HDF5 库
ls $HDF5_DIR/lib/libhdf5.*

# 检查头文件
ls $HDF5_DIR/include/hdf5.h

# 或使用 pkg-config
pkg-config --modversion hdf5
```

## 🆚 Rust HDF5 vs 系统 HDF5

| 特性 | Rust HDF5 包 | 直接使用系统 HDF5 |
|------|-------------|-----------------|
| **类型安全** | ✅ 完全类型安全 | ❌ 不安全（C 接口） |
| **API 风格** | 🔧 Rust 风格 | 🔨 C 风格 |
| **内存管理** | ✅ 自动管理 | ❌ 手动管理 |
| **错误处理** | ✅ Result<T, E> | ❌ 返回码 |
| **依赖** | ⚠️ 需要系统 HDF5 | ✅ 仅系统 HDF5 |
| **性能** | 🚀 接近原生 C | 🚀 原生 C |
| **集成** | ✅ ndarray, serde | ❌ 需要手动转换 |

## 💡 为什么使用 Rust HDF5 包？

### 优势

1. **类型安全**
   ```rust
   // Rust HDF5 - 类型安全
   let dataset = file.dataset("beta")?;
   let data: Array2<f32> = dataset.read()?;

   // vs C HDF5 - 不安全
   let data = unsafe { H5Dread(...) };  // 需要手动管理类型
   ```

2. **内存安全**
   - 自动管理内存
   - 无需手动释放资源
   - Rust 所有权系统保证

3. **与生态系统集成**
   ```rust
   // 直接与 ndarray 集成
   use hdf5::types::VarLenAscii;
   use ndarray::Array2;

   let matrix = Array2::<f32>::zeros((n_rows, n_cols));
   dataset.write(&matrix)?;
   ```

4. **错误处理**
   ```rust
   // 清晰的错误处理
   let file = File::open("data.h5")
       .context("Failed to open HDF5 file")?;
   ```

### 权衡

**优势**:
- ✅ 更安全的 API
- ✅ Rust 生态集成
- ✅ 更容易维护
- ✅ 编译时检查

**劣势**:
- ⚠️ 仍需要系统 HDF5 库
- ⚠️ 构建配置稍复杂
- ⚠️ FFI 开销（可忽略）

## 📚 相关资源

### Rust HDF5 项目

- **GitHub**: https://github.com/aldanor/hdf5-rust
- **文档**: https://docs.rs/hdf5/
- **版本**: 0.8.x

### 系统 HDF5

- **官方网站**: https://www.hdfgroup.org/
- **文档**: https://portal.hdfgroup.org/display/HDF5/

### 代码示例

项目中使用的示例：
- `src/hdf5/se_compat.rs` - HDF5 写入实现
- `docs/HDF5_STRUCTURE_AND_COORDINATES.md` - HDF5 文件格式

## 🔧 常见问题

### Q: 为什么还需要系统 HDF5 库？

**A**: Rust HDF5 包是一个 FFI 绑定，它提供了安全的 Rust API 来调用底层的 C HDF5 库。实际的 HDF5 文件读写仍然由 C 库完成。

### Q: 可以完全用 Rust 实现 HDF5 吗？

**A**: 理论上可以，但 HDF5 是一个非常复杂的格式（包含多种数据类型、压缩算法、并行 I/O 等），完全重新实现工作量巨大。当前的 FFI 方法是最佳平衡。

### Q: 性能会比纯 C 慢吗？

**A**: FFI 开销非常小（纳秒级），对于大型数据集操作（如本项目），性能差异可以忽略不计。

### Q: 如何选择 HDF5 版本？

**A**: 推荐使用：
- **开发环境**: 最新稳定版（1.14.x）
- **生产环境**: 与 R methrix 兼容的版本（1.12.x）
- **Conda**: `hdf5=1.12.2`（与本项目兼容）

## 📊 版本兼容性

| 组件 | 推荐版本 | 说明 |
|------|---------|------|
| Rust hdf5 crate | 0.8.x | 项目使用 0.8.1 |
| 系统 HDF5 | 1.12.x - 1.14.x | 兼容版本 |
| R rhdf5 | 任意 | 向后兼容 |
| R methrix | 任意 | 依赖 HDF5 文件格式 |

---

**最后更新**: 2026-02-22
**相关文档**:
- [BUILD.md](BUILD.md) - 构建指南
- [scripts/build.sh](../scripts/build.sh) - 构建脚本
