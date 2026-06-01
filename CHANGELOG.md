# DocDist 更新日志

本项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)，更新日志遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/) 规范。

## [Unreleased]

## [0.1.1] - 2026-06-01

### Added
- ⭐ **版本树功能** - Git风格版本管理：标记重要版本、按路径搜索、文件历史时间线、目录级恢复、历史导出ZIP
- 🧪 **测试覆盖扩展** - 新增25个测试：Store actions(16)、DB函数(4)、watcher路径排除(5)
- 📊 **虚拟化渲染** - DiffViewer/DiffDetailView使用@tanstack/react-virtual，大diff不再卡顿
- 🔧 **新增Tauri命令** - star_archive、unstar_archive、get_starred_archives、search_archives_by_path、get_file_history、restore_directory、export_history

### Fixed
- 🐛 **竞态条件** - starArchive/unstarArchive未await导致状态不一致
- 🐛 **错误处理** - fetchStarredArchives/fetchFileHistory吞掉错误无用户反馈
- 🐛 **SQL优化** - star_archive使用ON CONFLICT替代INSERT OR REPLACE，避免UUID泄漏
- 🐛 **表单验证** - StarDialog添加100字符限制、空值禁用、字符计数器
- 🐛 **配置同步** - setWatcherExcludePatterns成功后刷新配置状态
- 🐛 **导出安全** - exportHistory空目录验证防止create_dir_all失败
- 🐛 **内存清理** - StarDialog setTimeout清理防止卸载后focus
- 🐛 **useEffect优化** - VersionTreeView依赖file_path而非整个selectedArchive对象
- 🐛 **toast通知** - deleteArchive失败时显示错误提示

### Changed
- ⚡ **归档树N+1→1次查询** - get_all_archives_grouped_by_parent一次性加载，内存BFS构建树
- ⚡ **DiffViewer虚拟化** - 大diff从全量DOM改为只渲染可见行(400px+overscan 20)
- 📦 **新增依赖** - @tanstack/react-virtual (前端虚拟化)

### Technical
- Rust后端：新增db::get_all_archives_grouped_by_parent函数
- 前端Store：6个新增action全部有测试覆盖
- Watcher模块：is_path_excluded函数5个边界测试
- CI：8/8 jobs全绿

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
