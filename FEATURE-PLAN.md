# DocDist 功能完善计划

> **目标**: 全方位完善 DocDist 文件历史管理工具，对齐颗粒度，优化代码质量

**当前状态**: CI 全部通过 ✅，核心功能完成，测试覆盖良好

**最近更新**: 2026-06-01 - 版本树功能、性能优化、测试扩展

---

## 📋 Phase 1: 代码质量与 ESLint 修复 (优先级: HIGH)

### Task 1.1: 修复 ESLint 警告
**问题**: 有 27 个 ESLint 警告需要修复

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/components/archive/ArchiveList.tsx`

**修复内容:**
1. 移除未使用的 imports (`Settings`, `Maximize2`, `setView`)
2. 修复 React Hook useEffect 缺失依赖
3. 修复 `any` 类型使用

### Task 1.2: TypeScript 严格模式
**目标**: 启用 strict mode，修复所有类型问题

**Files:**
- Modify: `tsconfig.json`
- Modify: 所有 `.ts/.tsx` 文件

---

## 📋 Phase 2: 前端功能完善 (优先级: HIGH)

### Task 2.1: 完善 ArchiveCard 组件
**当前状态**: 基础卡片，缺少详细信息展示

**增强内容:**
- 文件大小格式化显示
- 标签彩色显示
- 创建时间相对显示 (刚刚/5分钟前/1小时前)
- 操作按钮优化 (恢复/对比/删除)
- 选中状态高亮

**Files:**
- Modify: `src/components/archive/ArchiveCard.tsx`
- Create: `src/utils/time.ts` (时间格式化工具)

### Task 2.2: 完善 DiffViewer 组件 ✅ **已完成 (2026-06-01)**
**当前状态**: 已实现虚拟化渲染，支持大diff

**已完成内容:**
- 语法高亮 (通过diff引擎)
- 行号显示
- 统计信息展示 (新增/删除/修改行数)
- **虚拟化渲染** - 使用@tanstack/react-virtual，大diff不再卡顿
- 折叠/展开功能
- 复制差异内容

**Files:**
- Modify: `src/components/diff/DiffViewer.tsx`
- Modify: `src/components/diff/DiffDetailView.tsx`
- Add dependency: `@tanstack/react-virtual`

### Task 2.3: 完善 TimelineView 组件
**当前状态**: 基础时间轴

**增强内容:**
- 可视化时间轴设计
- 版本节点交互
- 快速对比功能
- 筛选和排序

**Files:**
- Modify: `src/components/timeline/TimelineView.tsx`

### Task 2.4: 完善 IterationGraph 组件
**当前状态**: 基础图谱

**增强内容:**
- 使用 D3.js 或 vis.js 绘制真正的迭代图谱
- 节点交互 (点击查看详情)
- 分支可视化
- 缩放和平移

**Files:**
- Modify: `src/components/graph/IterationGraph.tsx`
- Add dependency: `d3` or `vis-network`

### Task 2.5: 完善 MiniMode 组件
**当前状态**: 基础迷你模式

**增强内容:**
- 快速存档按钮
- 最近存档列表
- 系统托盘集成
- 全局快捷键支持

**Files:**
- Modify: `src/components/mini/MiniMode.tsx`

---

## 📋 Phase 3: 后端功能完善 (优先级: HIGH)

### Task 3.1: 完善文件监控 (Watcher) ✅ **已完成 (2026-06-01)**
**当前状态**: 已实现实时监控、排除规则、TTL防抖

**已完成内容:**
- 实时文件变化监控
- 自动存档建议
- 监控目录管理
- 排除规则配置 (支持glob和目录名匹配)
- triggered_paths TTL防抖 (失败后可重试)
- is_path_excluded测试覆盖

**Files:**
- Modify: `src-tauri/src/watcher/mod.rs`

### Task 3.2: 完善存储优化 (Storage)
**当前状态**: 基础分块存储

**增强内容:**
- 增量存储优化
- 重复数据删除
- 存储空间统计
- 清理孤立 chunks

**Files:**
- Modify: `src-tauri/src/storage/mod.rs`

### Task 3.3: 完善差异算法 (Diff)
**当前状态**: 基础 diff

**增强内容:**
- 支持二进制文件 diff
- 图片 diff 可视化
- 性能优化 (大文件处理)
- 上下文行数配置

**Files:**
- Modify: `src-tauri/src/diff/mod.rs`

### Task 3.4: 完善数据库操作
**当前状态**: 基础 CRUD

**增强内容:**
- 分页查询
- 高级搜索 (正则/模糊)
- 批量操作
- 数据导出/导入

**Files:**
- Modify: `src-tauri/src/db/mod.rs`
- Modify: `src-tauri/src/commands/mod.rs`

---

## 📋 Phase 4: UI/UX 优化 (优先级: MEDIUM)

### Task 4.1: 主题系统
**增强内容:**
- 深色模式支持
- 主题切换动画
- 自定义主题色
- 系统主题跟随

**Files:**
- Create: `src/styles/themes.css`
- Modify: `tailwind.config.js`
- Modify: `src/App.tsx`

### Task 4.2: 国际化 (i18n)
**增强内容:**
- 中英文切换
- 语言包管理
- 日期/数字本地化

**Files:**
- Create: `src/i18n/` 目录
- Modify: 所有组件

### Task 4.3: 快捷键系统
**增强内容:**
- 全局快捷键
- 快捷键提示
- 自定义快捷键

**Files:**
- Create: `src/hooks/useKeyboard.ts`
- Modify: `src/App.tsx`

### Task 4.4: 通知系统
**增强内容:**
- 操作成功/失败提示
- 存档完成通知
- 错误处理优化

**Files:**
- Create: `src/components/common/Toast.tsx`
- Modify: `src/stores/archiveStore.ts`

---

## 📋 Phase 5: 测试完善 (优先级: MEDIUM)

### Task 5.1: 前端单元测试 ✅ **已完成 (2026-06-01)**
**当前状态**: 145个测试用例，覆盖核心功能

**已完成内容:**
- 组件测试覆盖率 > 80%
- Store测试 (archiveStore 16个测试)
- 工具函数测试 (format/time)
- Mock Tauri API (tauri-mocks.ts)
- 测试文件清单:
  - `src/stores/__tests__/archiveStore.test.ts` (16 tests)
  - `src/stores/__tests__/toastStore.test.ts` (11 tests)
  - `src/hooks/__tests__/useArchive.test.ts` (26 tests)
  - `src/hooks/__tests__/useTheme.test.ts` (22 tests)
  - `src/components/archive/__tests__/archive-components.test.tsx` (13 tests)
  - `src/components/common/__tests__/common-components.test.tsx` (17 tests)
  - `src/utils/__tests__/format.test.ts` (17 tests)
  - `src/utils/__tests__/time.test.ts` (21 tests)
  - `src/types/__tests__/types.test.ts` (2 tests)

**Files:**
- Create: `src/stores/__tests__/archiveStore.test.ts`
- Create: `src/test/tauri-mocks.ts`

### Task 5.2: Rust 单元测试 ✅ **已完成 (2026-06-01)**
**当前状态**: 200+测试用例，覆盖核心模块

**已完成内容:**
- 数据库操作测试 (40+ tests)
- 存储模块测试 (16 tests)
- Diff算法测试 (8 tests)
- 集成测试
- Watcher路径排除测试 (5 tests)
- 新增函数测试:
  - `db::get_archives_by_dir_before` (2 tests)
  - `db::get_all_archives_grouped_by_parent` (1 test)
  - `watcher::is_path_excluded` (5 tests)

**Files:**
- Modify: `src-tauri/src/db/mod.rs` (添加测试)
- Modify: `src-tauri/src/watcher/mod.rs` (添加测试)

### Task 5.3: E2E 测试
**增强内容:**
- Playwright 测试框架
- 关键流程测试
- 跨平台测试

**Files:**
- Create: `e2e/` 目录
- Modify: `.github/workflows/ci.yml`

---

## 📋 Phase 6: 性能优化 (优先级: LOW)

### Task 6.1: 前端性能 ✅ **已完成 (2026-06-01)**
**已完成内容:**
- 虚拟滚动 (大列表) - @tanstack/react-virtual
- DiffViewer/DiffDetailView虚拟化渲染
- 大diff从全量DOM改为只渲染可见行(400px+overscan 20)

### Task 6.2: 后端性能 ✅ **已完成 (2026-06-01)**
**已完成内容:**
- 归档树N+1查询优化 - get_all_archives_grouped_by_parent一次性加载
- 数据库索引优化 (已有)
- 连接池调优 (r2d2)
- 异步操作优化 (Tokio)

---

## 📋 Phase 7: 文档完善 (优先级: LOW)

### Task 7.1: 用户文档
- 使用指南
- API 文档
- 故障排除

### Task 7.2: 开发文档
- 架构设计
- 贡献指南
- 代码规范

---

## 🚀 执行策略

### 优先级排序
1. **Phase 1**: 代码质量 (立即开始)
2. **Phase 2**: 前端功能 (并行进行)
3. **Phase 3**: 后端功能 (并行进行)
4. **Phase 4**: UI/UX (后续迭代)
5. **Phase 5**: 测试 (持续进行)
6. **Phase 6**: 性能 (按需优化)
7. **Phase 7**: 文档 (最后完善)

### 执行方式
- 使用 `subagent-driven-development` 技能
- 每个 Task 独立子任务
- 完成后自动提交
- 持续集成验证

---

## 📊 成功指标

- [x] ESLint 警告: 0 (CI检查通过)
- [x] TypeScript 错误: 0 (CI检查通过)
- [x] 测试覆盖率: > 80% (前端145测试, Rust 200+测试)
- [ ] 构建时间: < 5 分钟
- [ ] Bundle 大小: < 5MB
- [ ] 用户满意度: > 4.5/5

---

**开始执行**: 主人确认后，洛熙将按计划逐步实施！💝
