# DocDist 更新日志

本项目遵循 [语义化版本控制](https://semver.org/lang/zh-CN/)，更新日志遵循 [Keep a Changelog](https://keepachangelog.com/zh-CN/1.0.0/) 规范。

## [Unreleased]

## [0.1.2] - 2026-06-01

### Fixed
- 🔒 **Critical安全修复** - export_history chunk路径错误（使用两级目录结构）
- 🔒 **Critical安全修复** - get_archives_by_dir_before LIKE注入（转义通配符+ESCAPE子句）
- 🔒 **Critical安全修复** - restore_directory zip-slip路径遍历（canonicalize验证）
- 🛡️ **ErrorBoundary** - 添加全局错误边界防止白屏
- ⚡ **SettingsPanel性能** - 添加shallow selector避免频繁重渲染
- 🐛 **版本对比无法使用** - TimelineView/VersionTreeView对比后不切换视图，用户看不到结果
- 🐛 **增强对比无法使用** - FileType枚举缺少serde(tag)导致前后端序列化格式不匹配
- 🐛 **时间轴序号显示** - 用版本序号#1/#2/#3替代checksum片段#6f7809c4
- 🐛 **时间戳精度** - created_at从秒级升级到毫秒级，同一秒内存档不再排序混乱
- 📄 **卸载清理指南** - 添加uninstall-cleanup.md脚本（Windows/macOS/Linux）

### Security
- 🔒 **符号链接攻击防护** - store_file添加symlink_metadata检查，拒绝符号链接文件
- 🛡️ **chunk_size上限** - update_config添加MAX_CHUNK_SIZE (256MB) 上限校验，防止OOM
- 🛡️ **日志行数限制** - read_log_file参数上限10000行，防止内存耗尽
- ⚡ **useFocusTrap hook** - 提取共享焦点陷阱hook，消除5个对话框组件的重复代码 (-62行净减少)

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
