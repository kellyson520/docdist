# DocDist 更新日志

本项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)，更新日志遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/) 规范。

## [Unreleased]

## [0.1.0] - 2024-12-30

### Added
- 🗂️ **一键存档/恢复** - 增量存储记录文件历史，节省空间，快速回溯
- 📊 **版本差异对比** - 对比历史存档之间的差异，查看新增、修改、删除内容
- 🌳 **迭代关系图** - 类思维导图形式呈现存档关系，展示分支路径
- 🪟 **Mini 模式** - 悬浮窗口快速存档或切换，减少屏幕占用
- 🏷️ **备注/标签** - 为存档添加说明或颜色标签，标记重要版本
- 📅 **时间轴视图** - 按时间顺序排列存档，快速定位目标存档
- 🔍 **搜索存档** - 按文件名、备注搜索存档
- 📦 **增量存储** - 4KB 分块 + XXHash 去重，节省 60-80% 存储空间

### Technical
- React 18 + TypeScript 5 前端
- Tauri 1.5 + Rust 后端
- SQLite 数据库 + r2d2 连接池
- Vitest 单元测试
- GitHub Actions CI/CD
