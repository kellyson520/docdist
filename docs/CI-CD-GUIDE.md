# DocDist - CI/CD 完整指南

## 📋 目录

1. [什么时候测试](#1-什么时候测试)
2. [什么情况测试](#2-什么情况测试)
3. [如何测试](#3-如何测试)
4. [不同情况该怎么测试](#4-不同情况该怎么测试)
5. [测试完成什么情况下发布](#5-测试完成什么情况下发布)
6. [更新日志如何编写](#6-更新日志如何编写)
7. [版本号如何迭代](#7-版本号如何迭代)
8. [构建产物如何推送](#8-构建产物如何推送)
9. [如何加快 CI 测试节约时间](#9-如何加快-ci-测试节约时间)

---

## 1. 什么时候测试

### 流水线阶段图

```
┌─────────────────────────────────────────────────────────────────────┐
│                        CI 流水线触发时机                             │
├─────────────────────────────────────────────────────────────────────┤
│                                                                     │
│  Push to main/develop/release/* ──┬── 快速检查 ── 单元测试 ── 构建   │
│                                   │                                 │
│  Pull Request to main ────────────┼── 快速检查 ── 单元测试 ── E2E    │
│                                   │                                 │
│  Tag v* ──────────────────────────┴── 快速检查 ── 单元测试 ── 发布   │
│                                                                     │
└─────────────────────────────────────────────────────────────────────┘
```

### 详细触发条件

| 事件 | 触发条件 | 执行内容 |
|------|----------|----------|
| **Push** | 推送到 main/develop/release 分支 | 快速检查 → 单元测试 → 构建 |
| **PR** | 向 main 发起 Pull Request | 快速检查 → 单元测试 → E2E 测试 |
| **Tag** | 推送 v* 标签 (如 v1.0.0) | 快速检查 → 单元测试 → 构建 → 发布 |
| **手动** | workflow_dispatch | 全部流程 |

### 不触发的情况

| 条件 | 原因 |
|------|------|
| 仅修改 .md 文件 | 文档变更不需要测试 |
| 仅修改 LICENSE | 许可证变更不需要测试 |
| 仅修改 docs/ 目录 | 文档目录变更不需要测试 |

---

## 2. 什么情况测试

### 测试类型与场景

| 测试类型 | 触发场景 | 目的 | 耗时 |
|----------|----------|------|------|
| **快速检查** | 每次 Push / PR | 拦截语法错误、格式问题 | < 2 分钟 |
| **Rust 检查** | 每次 Push / PR | Clippy 静态分析、格式检查 | < 3 分钟 |
| **前端单元测试** | 快速检查通过后 | 验证 React 组件逻辑 | < 5 分钟 |
| **Rust 单元测试** | Rust 检查通过后 | 验证 Rust 业务逻辑 | < 5 分钟 |
| **构建验证** | 测试通过 + main/release/tag | 多平台构建成功 | < 15 分钟 |
| **E2E 测试** | 仅 PR | 端到端功能验证 | < 10 分钟 |
| **安全扫描** | 快速检查通过后 | 依赖漏洞检测 | < 5 分钟 |

### 测试矩阵

```
                     快速检查    单元测试    构建    E2E    安全扫描
Push to main          ✅          ✅        ✅     ❌       ✅
Push to develop       ✅          ✅        ✅     ❌       ✅
Push to release/*     ✅          ✅        ✅     ❌       ✅
Pull Request          ✅          ✅        ❌     ✅       ✅
Tag v*                ✅          ✅        ✅     ❌       ✅
仅修改 .md            ❌          ❌        ❌     ❌       ❌
```

---

## 3. 如何测试

### 3.1 本地测试命令

```bash
# ===== 前端测试 =====
# TypeScript 类型检查
npx tsc --noEmit

# ESLint 代码检查
npx eslint src --ext ts,tsx

# 运行单元测试
npx vitest run

# 运行测试 + 覆盖率
npx vitest run --coverage

# 监听模式测试
npx vitest watch

# ===== Rust 测试 =====
# 格式检查
cd src-tauri && cargo fmt --all -- --check

# Clippy 静态分析
cd src-tauri && cargo clippy -- -D warnings

# 运行所有测试
cd src-tauri && cargo test --all

# 运行特定测试
cd src-tauri && cargo test test_name

# 测试 + 详细输出
cd src-tauri && cargo test --all --verbose

# ===== E2E 测试 =====
# 安装 Playwright
npx playwright install

# 运行 E2E 测试
npx playwright test

# 查看测试报告
npx playwright show-report

# ===== 安全扫描 =====
# Node.js 依赖审计
npm audit

# Rust 依赖审计
cargo install cargo-audit && cargo audit
```

### 3.2 CI 中的测试命令

```yaml
# 快速检查
npx tsc --noEmit                    # TypeScript 类型检查
npx eslint src --ext ts,tsx         # ESLint 检查
cargo fmt --all -- --check          # Rust 格式检查
cargo clippy -- -D warnings         # Clippy 静态分析

# 单元测试
npx vitest run --coverage           # 前端测试 + 覆盖率
cargo test --all --verbose          # Rust 测试

# E2E 测试
npx playwright test                 # Playwright 端到端测试

# 安全扫描
npm audit --audit-level=high        # Node.js 安全审计
cargo audit                         # Rust 安全审计
```

---

## 4. 不同情况该怎么测试

### 4.1 场景：修改了前端代码

```bash
# 本地验证
npx tsc --noEmit              # 类型检查
npx eslint src --ext ts,tsx   # 代码规范
npx vitest run                # 单元测试
```

**CI 流程**: 快速检查 → 前端单元测试 → 构建

### 4.2 场景：修改了 Rust 后端代码

```bash
# 本地验证
cd src-tauri
cargo fmt --all               # 格式化
cargo clippy -- -D warnings   # 静态分析
cargo test --all              # 单元测试
```

**CI 流程**: 快速检查 → Rust 检查 → Rust 单元测试 → 构建

### 4.3 场景：修改了前后端代码

```bash
# 本地验证
npx tsc --noEmit && npx vitest run
cd src-tauri && cargo test --all
```

**CI 流程**: 全部测试 → 构建

### 4.4 场景：仅修改配置文件

```bash
# 本地验证
npx tsc --noEmit              # 检查配置是否影响类型
cd src-tauri && cargo check   # 检查 Cargo.toml 变更
```

**CI 流程**: 快速检查 → 单元测试

### 4.5 场景：Pull Request

```bash
# 完整本地验证
npm run lint && npm run type-check && npm run test
cd src-tauri && cargo fmt --check && cargo clippy && cargo test
```

**CI 流程**: 快速检查 → 单元测试 → E2E 测试 → 安全扫描

### 4.6 场景：准备发布

```bash
# 完整验证
npm run lint && npm run test:coverage
cd src-tauri && cargo test --all
npx playwright test           # E2E 测试

# 构建验证
npm run tauri build           # 本地构建
```

**CI 流程**: 全部测试 → 多平台构建 → 发布

---

## 5. 测试完成什么情况下发布

### 发布条件

```
┌─────────────────────────────────────────────────────────────────┐
│                         发布决策树                               │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  推送 v* 标签?                                                  │
│  ├── 否 → 不发布                                                │
│  └── 是 → 检查以下条件:                                         │
│      ├── ✅ 快速检查通过?                                       │
│      ├── ✅ 前端单元测试通过?                                   │
│      ├── ✅ Rust 单元测试通过?                                  │
│      ├── ✅ 三平台构建成功? (Windows/macOS/Linux)               │
│      └── ✅ 无高危安全漏洞?                                     │
│          └── 全部通过 → 自动发布到 GitHub Releases              │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 发布类型

| 标签格式 | 发布类型 | 说明 |
|----------|----------|------|
| `v1.0.0` | 正式版 | Production Release |
| `v1.0.0-beta.1` | 测试版 | Pre-release |
| `v1.0.0-alpha.1` | 内测版 | Pre-release |
| `v1.0.0-rc.1` | 候选版 | Pre-release |

### 发布流程

```bash
# 1. 更新版本号
# 编辑 src-tauri/Cargo.toml 和 package.json 中的版本号

# 2. 更新 CHANGELOG.md
# 添加本次发布的变更说明

# 3. 提交版本变更
git add -A
git commit -m "chore: bump version to v1.0.0"

# 4. 创建标签
git tag -a v1.0.0 -m "Release v1.0.0"

# 5. 推送标签触发 CI
git push origin main --tags

# 6. CI 自动执行:
#    - 快速检查
#    - 单元测试
#    - 多平台构建
#    - 创建 GitHub Release
#    - 上传构建产物
```

### 手动发布 (workflow_dispatch)

如果需要手动触发发布：

```yaml
# 在 GitHub Actions 页面手动运行
# 选择分支和输入版本号
```

---

## 6. 更新日志如何编写

### 6.1 CHANGELOG.md 格式

遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/) 规范：

```markdown
# 更新日志

本项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)。

## [Unreleased]

### Added
- 新增 XXX 功能

### Changed
- 修改 XXX 行为

### Deprecated
- 废弃 XXX 功能

### Removed
- 移除 XXX 功能

### Fixed
- 修复 XXX 问题

### Security
- 修复 XXX 安全漏洞

## [1.0.0] - 2024-12-20

### Added
- 🗂️ 一键存档/恢复功能
- 📊 版本差异对比
- 🌳 迭代关系图
- 🪟 Mini 模式
- 🏷️ 备注/标签系统
- 📅 时间轴视图
- 🔍 搜索存档

### Changed
- 初始版本发布

## [0.1.0] - 2024-12-01

### Added
- 项目初始化
- 基础架构搭建
```

### 6.2 提交信息规范

使用 [Conventional Commits](https://www.conventionalcommits.org/zh-hans/) 规范：

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Type 类型:**

| Type | 说明 | 示例 |
|------|------|------|
| `feat` | 新功能 | `feat(archive): 新增批量存档功能` |
| `fix` | 修复 Bug | `fix(diff): 修复对比结果不准确` |
| `docs` | 文档 | `docs: 更新 README 安装说明` |
| `style` | 格式 | `style: 修复代码缩进` |
| `refactor` | 重构 | `refactor(storage): 优化存储引擎` |
| `perf` | 性能 | `perf(db): 优化数据库查询` |
| `test` | 测试 | `test: 新增存档功能单元测试` |
| `chore` | 构建 | `chore: 更新依赖版本` |
| `ci` | CI/CD | `ci: 优化构建缓存策略` |
| `revert` | 回滚 | `revert: 回滚 XXX 提交` |

**示例:**

```bash
# 新功能
git commit -m "feat(archive): 新增批量存档功能

支持选择多个文件同时创建存档，提升操作效率。

Closes #123"

# Bug 修复
git commit -m "fix(diff): 修复中文文件名对比乱码问题

使用 UTF-8 编码处理文件内容，解决中文显示问题。

Fixes #456"

# 版本发布
git commit -m "chore: bump version to v1.0.0"
```

### 6.3 自动生成 CHANGELOG

使用 `git-cliff` 自动生成：

```bash
# 安装
cargo install git-cliff

# 生成 CHANGELOG
git-cliff -o CHANGELOG.md

# 生成指定版本的 CHANGELOG
git-cliff --tag v1.0.0 -o CHANGELOG.md
```

配置文件 `.cliff.toml`:

```toml
[changelog]
header = "# 更新日志\n\n本项目遵循语义化版本控制。\n"
body = """
{% if version %}\
    ## [{{ version | trim_start_matches(pat="v") }}] - {{ timestamp | date(format="%Y-%m-%d") }}
{% else %}\
    ## [Unreleased]
{% endif %}\
{% for group, commits in commits | group_by(attribute="group") %}
    ### {{ group | striptags | trim | upper_first }}
    {% for commit in commits %}
        - {% if commit.scope %}**{{ commit.scope }}**: {% endif %}{{ commit.message | upper_first }}\
    {% endfor %}
{% endfor %}
"""
footer = """
{% for commit in commits %}
    {% if commit.breaking %}\
        **Breaking Change**: {{ commit.message | upper_first }}
    {% endif %}\
{% endfor %}
"""

[git]
conventional_commits = true
filter_unconventional = true
split_commits = false
commit_parsers = [
    { message = "^feat", group = "✨ Features" },
    { message = "^fix", group = "🐛 Bug Fixes" },
    { message = "^doc", group = "📖 Documentation" },
    { message = "^perf", group = "⚡ Performance" },
    { message = "^refactor", group = "♻️ Refactor" },
    { message = "^style", group = "🎨 Styling" },
    { message = "^test", group = "🧪 Tests" },
    { message = "^chore", group = "🔧 Misc" },
    { message = "^ci", group = "🚀 CI/CD" },
]
```

---

## 7. 版本号如何迭代

### 7.1 语义化版本 (SemVer)

```
MAJOR.MINOR.PATCH

MAJOR: 不兼容的 API 变更 (破坏性变更)
MINOR: 向后兼容的功能新增
PATCH: 向后兼容的问题修复
```

### 7.2 版本号决策树

```
是否有破坏性变更?
├── 是 → MAJOR +1 (MINOR/RESET to 0, PATCH RESET to 0)
└── 否 → 是否有新功能?
    ├── 是 → MINOR +1 (PATCH RESET to 0)
    └── 否 → 是否有 Bug 修复?
        ├── 是 → PATCH +1
        └── 否 → 不发版
```

### 7.3 版本号示例

| 变更类型 | 当前版本 | 新版本 | 说明 |
|----------|----------|--------|------|
| 新增功能 | 1.0.0 | 1.1.0 | 新增 Mini 模式 |
| Bug 修复 | 1.1.0 | 1.1.1 | 修复时间轴显示问题 |
| 破坏性变更 | 1.1.1 | 2.0.0 | 重构数据库 Schema |
| 预发布版本 | 2.0.0 | 2.0.0-beta.1 | 内部测试 |
| 候选版本 | 2.0.0-beta.1 | 2.0.0-rc.1 | 准备发布 |
| 正式版本 | 2.0.0-rc.1 | 2.0.0 | 正式发布 |

### 7.4 预发布版本规则

```
1.0.0-alpha.1    # 内测版，功能不完整
1.0.0-alpha.2    # 内测版迭代
1.0.0-beta.1     # 测试版，功能完整
1.0.0-beta.2     # 测试版迭代
1.0.0-rc.1       # 候选版，准备发布
1.0.0-rc.2       # 候选版迭代
1.0.0            # 正式版
```

### 7.5 版本号更新流程

```bash
# 1. 更新 Cargo.toml
# src-tauri/Cargo.toml
[package]
version = "1.1.0"

# 2. 更新 package.json
# package.json
{
  "version": "1.1.0"
}

# 3. 更新 tauri.conf.json
# src-tauri/tauri.conf.json
{
  "package": {
    "version": "1.1.0"
  }
}

# 4. 提交版本变更
git add -A
git commit -m "chore: bump version to v1.1.0"

# 5. 创建标签
git tag -a v1.1.0 -m "Release v1.1.0"

# 6. 推送
git push origin main --tags
```

---

## 8. 构建产物如何推送

### 8.1 构建产物清单

| 平台 | 格式 | 文件名 | 大小 |
|------|------|--------|------|
| Windows | MSI | `追光_Lite_1.0.0_x64.msi` | ~5-10 MB |
| Windows | EXE | `追光_Lite_1.0.0_x64-setup.exe` | ~5-10 MB |
| macOS | DMG | `追光_Lite_1.0.0_aarch64.dmg` | ~8-15 MB |
| Linux | AppImage | `追光_Lite_1.0.0_amd64.AppImage` | ~10-20 MB |
| Linux | DEB | `追光_Lite_1.0.0_amd64.deb` | ~8-15 MB |

### 8.2 推送流程

```
┌─────────────────────────────────────────────────────────────────┐
│                     构建产物推送流程                             │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  推送 v* 标签                                                   │
│      ↓                                                          │
│  CI 触发构建                                                    │
│      ↓                                                          │
│  三平台并行构建                                                 │
│  ├── Windows (MSI + EXE)                                        │
│  ├── macOS (DMG)                                                │
│  └── Linux (AppImage + DEB)                                     │
│      ↓                                                          │
│  构建产物上传到 GitHub Actions Artifacts                        │
│      ↓                                                          │
│  Release Job 触发                                               │
│      ↓                                                          │
│  下载所有构建产物                                               │
│      ↓                                                          │
│  创建 GitHub Release                                            │
│      ↓                                                          │
│  上传产物到 Release                                             │
│      ↓                                                          │
│  用户可在 Release 页面下载                                      │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

### 8.3 产物签名

Tauri 自动对构建产物进行签名：

```json
// tauri.conf.json
{
  "tauri": {
    "bundle": {
      "active": true,
      "targets": "all",
      "identifier": "com.zhuiguang.lite",
      "icon": [
        "icons/32x32.png",
        "icons/128x128.png",
        "icons/icon.icns",
        "icons/icon.ico"
      ]
    }
  }
}
```

**签名密钥配置:**

```yaml
# GitHub Secrets 中配置
TAURI_PRIVATE_KEY: <你的私钥>
TAURI_KEY_PASSWORD: <私钥密码>
```

### 8.4 自动更新

Tauri 支持自动更新，配置更新服务器：

```json
// tauri.conf.json
{
  "tauri": {
    "updater": {
      "active": true,
      "endpoints": [
        "https://releases.zhuiguang.dev/{{target}}/{{arch}}/{{current_version}}"
      ],
      "dialog": true,
      "pubkey": "你的公钥"
    }
  }
}
```

---

## 9. 如何加快 CI 测试节约时间

### 9.1 优化策略总览

| 策略 | 节省时间 | 实现难度 |
|------|----------|----------|
| Cargo 缓存 | 5-10 分钟 | ⭐ |
| npm 缓存 | 1-2 分钟 | ⭐ |
| 并行执行 | 5-10 分钟 | ⭐⭐ |
| 增量测试 | 3-5 分钟 | ⭐⭐ |
| 条件跳过 | 2-5 分钟 | ⭐ |
| 路径过滤 | 1-3 分钟 | ⭐ |

### 9.2 具体优化措施

#### 1. Cargo 缓存 (节省 5-10 分钟)

```yaml
- name: Cache Cargo
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/bin/
      ~/.cargo/registry/index/
      ~/.cargo/registry/cache/
      ~/.cargo/git/db/
      src-tauri/target/
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
    restore-keys: ${{ runner.os }}-cargo-
```

#### 2. npm 缓存 (节省 1-2 分钟)

```yaml
- name: Setup Node.js
  uses: actions/setup-node@v4
  with:
    node-version: '18'
    cache: 'npm'
```

#### 3. 并行执行 (节省 5-10 分钟)

```yaml
# Stage 1 的任务并行执行
jobs:
  quick-check:      # 并行
    ...
  rust-quick-check: # 并行
    ...
  security:         # 并行
    ...
```

#### 4. 条件跳过 (节省 2-5 分钟)

```yaml
# 仅在特定条件下执行
if: github.ref == 'refs/heads/main' || startsWith(github.ref, 'refs/tags/v')
```

#### 5. 路径过滤 (节省 1-3 分钟)

```yaml
on:
  push:
    paths-ignore:
      - '*.md'
      - 'LICENSE'
      - 'docs/**'
```

#### 6. 取消重复运行

```yaml
concurrency:
  group: ci-${{ github.ref }}
  cancel-in-progress: true
```

#### 7. 使用更快的 Runner

```yaml
# 使用更大的 Runner (需要付费)
runs-on: ubuntu-latest-8-cores
```

### 9.3 优化后的耗时对比

| 阶段 | 优化前 | 优化后 | 节省 |
|------|--------|--------|------|
| 快速检查 | 3 分钟 | 1 分钟 | 2 分钟 |
| 单元测试 | 8 分钟 | 3 分钟 | 5 分钟 |
| 构建 | 20 分钟 | 10 分钟 | 10 分钟 |
| E2E 测试 | 15 分钟 | 8 分钟 | 7 分钟 |
| **总计** | **46 分钟** | **22 分钟** | **24 分钟** |

### 9.4 最佳实践

```yaml
# 1. 使用 Node.js 18+ (更快的 npm)
node-version: '18'

# 2. 使用 Rust stable (而非 nightly)
rust-version: 'stable'

# 3. 并行运行测试
cargo test --all  # 自动并行

# 4. 跳过不必要的测试
cargo test --lib  # 仅运行库测试

# 5. 使用 --release 优化构建
cargo build --release

# 6. 使用 sccache 加速编译
- name: Setup sccache
  uses: mozilla-actions/sccache-action@v0.0.3

- name: Build
  env:
    SCCACHE_GHA_ENABLED: "true"
    RUSTC_WRAPPER: "sccache"
  run: cargo build --release
```

---

## 附录：CI 流水线图

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              CI 流水线                                       │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  Push/PR/Tag                                                                │
│      │                                                                      │
│      ▼                                                                      │
│  ┌─────────────────────────────────────────┐                                │
│  │           Stage 1: 快速检查              │  (< 2 分钟)                    │
│  │  TypeScript | ESLint | Rust fmt | Clippy │                                │
│  └─────────────────┬───────────────────────┘                                │
│                    │                                                        │
│           ┌────────┴────────┐                                               │
│           ▼                 ▼                                                │
│  ┌─────────────────┐ ┌─────────────────┐                                    │
│  │ 前端单元测试    │ │ Rust 单元测试   │  (< 5 分钟，并行)                   │
│  │ Vitest + 覆盖率 │ │ cargo test      │                                    │
│  └────────┬────────┘ └────────┬────────┘                                    │
│           │                   │                                              │
│           └─────────┬─────────┘                                              │
│                     │                                                        │
│           ┌─────────┴─────────┐                                              │
│           ▼                   ▼                                               │
│  ┌─────────────────┐ ┌─────────────────┐                                    │
│  │ 构建 (main/tag) │ │ E2E (仅 PR)     │  (< 15 分钟)                       │
│  │ 三平台并行构建  │ │ Playwright      │                                    │
│  └────────┬────────┘ └─────────────────┘                                    │
│           │                                                                  │
│           ▼                                                                  │
│  ┌─────────────────┐                                                        │
│  │ 发布 (仅 tag)   │                                                        │
│  │ GitHub Release  │                                                        │
│  └─────────────────┘                                                        │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

> 📝 **文档版本**: v1.0  
> 📅 **更新日期**: 2024-12  
> 👤 **维护者**: 追光团队
