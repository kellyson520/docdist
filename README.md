# DocDist

> 轻量简约的文件历史管理工具，帮你跟踪、回溯、整理文件历史。

## ✨ 核心功能

| 功能 | 说明 |
|------|------|
| 🗂️ **一键存档/恢复** | 增量存储记录文件历史，节省空间，快速回溯 |
| 📊 **版本差异对比** | 对比历史存档之间的差异，查看新增、修改、删除 |
| 🌳 **迭代关系图** | 类思维导图形式呈现存档关系，展示分支路径 |
| ⭐ **版本树管理** | Git风格：标记重要版本、路径搜索、文件历史、目录恢复、导出ZIP |
| 🪟 **Mini 模式** | 悬浮窗口快速存档或切换，减少屏幕占用 |
| 🏷️ **备注/标签** | 为存档添加说明或颜色标签，标记重要版本 |
| 📅 **时间轴视图** | 按时间顺序排列存档，快速定位目标存档 |
| 🔍 **搜索存档** | 按文件名、备注搜索存档 |
| 📦 **增量存储** | 4KB 分块 + XXHash 去重，节省 60-80% 存储空间 |

## 🛠️ 技术栈

- **前端**: React 18 + TypeScript 5 + Zustand + Tailwind CSS
- **后端**: Tauri 1.5 + Rust + Tokio + SQLite
- **存储**: 增量分块 (4KB) + XXHash 去重 + 引用计数

## 🚀 快速开始

### 环境要求

- Node.js >= 18
- Rust >= 1.70
- Tauri CLI: `cargo install tauri-cli`

### 开发

```bash
# 安装依赖
npm install

# 启动开发模式
npm run tauri dev
```

### 构建

```bash
npm run tauri build
```

### 测试

```bash
# 前端测试
npm run test

# Rust 测试
cd src-tauri && cargo test

# E2E 测试
npm run test:e2e
```

## 📁 项目结构

```
docdist/
├── src/                    # React 前端
│   ├── components/         # UI 组件
│   │   ├── archive/        # 存档管理
│   │   ├── timeline/       # 时间轴
│   │   ├── diff/           # 差异对比
│   │   ├── graph/          # 迭代关系图
│   │   ├── mini/           # Mini 模式
│   │   └── common/         # 通用组件
│   ├── stores/             # 状态管理
│   ├── types/              # 类型定义
│   └── utils/              # 工具函数
├── src-tauri/              # Rust 后端
│   └── src/
│       ├── commands/       # Tauri Commands
│       ├── services/       # 业务服务
│       ├── storage/        # 增量存储引擎
│       ├── diff/           # 差异对比引擎
│       ├── watcher/        # 文件监控
│       └── db/             # 数据库
├── e2e/                    # E2E 测试
├── docs/                   # 文档
└── package.json
```

## 📦 下载

前往 [Releases](https://github.com/kellyson520/docdist/releases) 页面下载最新版本。

- Windows: `.msi` 或 `.exe`
- macOS: `.dmg`
- Linux: `.AppImage` 或 `.deb`

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

1. Fork 本仓库
2. 创建功能分支: `git checkout -b feature/amazing-feature`
3. 提交更改: `git commit -m 'feat: 新增某功能'`
4. 推送分支: `git push origin feature/amazing-feature`
5. 提交 Pull Request

## 📄 License

MIT
