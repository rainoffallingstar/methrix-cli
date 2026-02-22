# HDF5 依赖 - 快速参考

## ✅ 是的，我们使用 Rust 原生的 HDF5 包！

### 当前配置

```toml
# Cargo.toml
[dependencies]
hdf5 = "0.8"
```

### 这是一个纯 Rust 的包

**GitHub**: https://github.com/aldanor/hdf5-rust
**文档**: https://docs.rs/hdf5/

### 架构

```
您的代码
  ↓
hdf5 crate (Rust)  ← 纯 Rust API！
  ↓
hdf5-sys (FFI)     ← Rust 到 C 的绑定
  ↓
C HDF5 库          ← 系统库依赖
```

### 关键点

✅ **Rust 原生包**: `hdf5` 是 100% Rust 代码
⚠️ **系统依赖**: 仍需要系统 HDF5 C 库（通过 FFI）
✅ **类型安全**: 完整的 Rust 类型系统
✅ **内存安全**: 自动内存管理

### 为什么还需要系统 HDF5？

Rust `hdf5` crate 使用 FFI (外部函数接口) 调用 C HDF5 库：
- C HDF5 库处理实际的文件 I/O
- Rust 提供安全的包装层
- 这是最佳实践（不需要重新实现复杂的 HDF5 格式）

### 代码示例

```rust
// 类型安全的 Rust API
use hdf5::{File, Group};
use hdf5::types::VarLenAscii;

// 打开文件（安全）
let file = File::create("output.h5")?;

// 创建数据集（类型检查）
let dataset = file.new_dataset_builder()
    .with_data(&matrix)
    .create("beta")?;

// 自动内存管理（无需手动释放）
```

### vs 纯 C HDF5

```c
// C HDF5 - 不安全
hid_t file = H5Fcreate("output.h5", ...);
hid_t dataset = H5Dcreate(...);
// 需要手动管理内存和错误
H5Dclose(dataset);
H5Fclose(file);
```

---

**结论**: 我们使用的是 Rust 原生的 `hdf5` crate，它提供了类型安全的 API，同时通过 FFI 利用成熟的 C HDF5 库。

**详细信息**: 参见 [HDF5_DEPENDENCY.md](HDF5_DEPENDENCY.md)
