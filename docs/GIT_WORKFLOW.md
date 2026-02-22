# Git 仓库设置说明

## 📋 仓库信息

**初始化日期**: 2026-02-22
**初始提交**: a4e3e71
**分支**: master
**文件数**: 96 个已跟踪文件

## 📁 仓库结构

```
methrix-cli/
├── .gitignore              # Git 忽略规则
├── .git/                   # Git 仓库数据
├── src/                    # Rust 源代码 (已跟踪)
├── docs/                   # 文档 (已跟踪)
├── scripts/                # Shell 脚本 (已跟踪)
├── slurm_scripts/          # SLURM 脚本 (已跟踪)
├── archived_scripts/       # 归档脚本 (已跟踪)
├── tests/                  # 测试 (已跟踪)
├── testdata/               # 测试数据 (部分已跟踪)
├── target/                 # ❌ 已忽略 (构建产物)
└── logs/                   # ❌ 已忽略 (日志文件)
```

## 🚫 .gitignore 规则

### 主要忽略项

1. **Rust 构建产物**
   ```
   /target/
   *.rs.bk
   Cargo.lock
   ```

2. **测试数据和大文件**
   ```
   testdata/**/*.h5
   testdata/**/*.rds
   testdata/**/*.xlsx
   testdata/**/*.ron
   testdata/**/*.fa
   testdata/**/*.fa.gz
   ```

3. **IDE 和编辑器**
   ```
   .idea/
   .vscode/
   *.swp
   *~
   ```

4. **日志和输出**
   ```
   logs/
   *.log
   *.out
   *.err
   ```

5. **SLURM**
   ```
   slurm-*.out
   slurm-*.err
   .last_job_id
   ```

6. **操作系统**
   ```
   .DS_Store
   Thumbs.db
   ```

## 📊 当前状态

### 已跟踪文件
- **总数**: 96 个文件
- **源代码**: 14 个 Rust 文件
- **文档**: 27 个 Markdown 文件
- **脚本**: 37 个脚本文件（Shell + R）
- **测试**: 3 个测试文件
- **其他**: 配置文件、README 等

### 未跟踪文件（被忽略）
- `/target/` - Rust 构建产物
- `/logs/` - 日志文件
- 大文件 - HDF5 文件、基因组数据等

## 🔧 常用 Git 命令

### 查看状态
```bash
git status
git status --short
```

### 查看日志
```bash
git log
git log --oneline
git log --graph --all
```

### 创建分支
```bash
git checkout -b feature/your-feature
```

### 提交更改
```bash
git add .
git commit -m "Your commit message"
```

### 推送到远程
```bash
git remote add origin <repository-url>
git push -u origin master
```

## 📝 提交规范

### 提交消息格式

```
<type>: <subject>

<body>

<footer>
```

### 类型（type）

- **feat**: 新功能
- **fix**: 修复 bug
- **docs**: 文档更新
- **style**: 代码格式（不影响功能）
- **refactor**: 重构
- **test**: 测试相关
- **chore**: 构建/工具相关

### 示例

```
feat: add HDF5 dataset name validation

- Add validation for beta/cov dataset names
- Improve error messages for invalid names
- Add unit tests for validation logic

Closes #123
```

## 🌱 分支策略

### 主要分支

- **master**: 主分支，稳定版本
- **develop**: 开发分支（如果需要）

### 功能分支

- **feature/***: 新功能
- **fix/***: Bug 修复
- **docs/***: 文档更新
- **refactor/***: 重构

### 示例

```bash
# 创建功能分支
git checkout -b feature/add-genome-download

# 完成后合并回 master
git checkout master
git merge feature/add-genome-download
```

## 🏷️ 标签管理

### 创建标签

```bash
# 创建注释标签
git tag -a v1.0.0 -m "Release version 1.0.0"

# 推送标签
git push origin v1.0.0
```

### 查看标签

```bash
# 列出所有标签
git tag

# 显示标签详情
git show v1.0.0
```

## 🔍 查看差异

```bash
# 查看未暂存的更改
git diff

# 查看已暂存的更改
git diff --staged

# 查看特定文件的更改
git diff src/main.rs
```

## 📤 推送到远程

### 设置远程仓库

```bash
# GitHub
git remote add origin https://github.com/username/methrix-cli.git

# 或 SSH
git remote add origin git@github.com:username/methrix-cli.git
```

### 首次推送

```bash
git push -u origin master
```

### 推送所有分支

```bash
git push --all origin
```

### 推送标签

```bash
git push --tags
```

## 🔄 拉取更新

```bash
# 拉取并合并
git pull origin master

# 或分别执行
git fetch origin
git merge origin/master
```

## 🗑️ 清理

### 清理未跟踪的文件

```bash
# 预览要删除的文件
git clean -n

# 删除文件
git clean -f

# 删除文件和目录
git clean -fd
```

### 清理已跟踪但被删除的文件

```bash
git gc --prune=now
```

## 📊 统计信息

```bash
# 代码行数统计
git ls-files | xargs wc -l

# 提交统计
git shortlog -sn

# 按作者统计
git shortlog -sn --all
```

## 🔐 安全注意事项

1. **不要提交敏感信息**
   - 密码
   - API 密钥
   - 个人数据
   - 测试数据中的真实样本信息

2. **检查大文件**
   ```bash
   # 查找大于 10MB 的文件
   find . -type f -size +10M
   ```

3. **使用 .gitignore**
   - 确保构建产物被忽略
   - 确保测试数据被忽略
   - 确保日志文件被忽略

## 📚 相关资源

- **Git 文档**: https://git-scm.com/doc
- **GitHub 文档**: https://docs.github.com/
- **.gitignore 模板**: https://github.com/github/gitignore

---

**最后更新**: 2026-02-22
**维护者**: methrix-cli contributors
