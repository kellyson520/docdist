import React, { useEffect, useState, useCallback, useRef, Suspense } from 'react';
import { useArchiveStore } from './stores/archiveStore';
import { shallow } from 'zustand/shallow';
import { ArchiveList } from './components/archive/ArchiveList';
import { TimelineView } from './components/timeline/TimelineView';
import { MiniMode } from './components/mini/MiniMode';

// Lazy load heavy view components
const DiffViewer = React.lazy(() => import('./components/diff/DiffViewer').then(m => ({ default: m.DiffViewer })));
const EnhancedDiffViewer = React.lazy(() => import('./components/diff/EnhancedDiffViewer').then(m => ({ default: m.EnhancedDiffViewer })));
const IterationGraph = React.lazy(() => import('./components/graph/IterationGraph').then(m => ({ default: m.IterationGraph })));
import { ThemeToggle } from './components/common/ThemeToggle';
import { WatcherPanel } from './components/watcher/WatcherPanel';
import { SettingsPanel } from './components/settings/SettingsPanel';
import { ToastContainer } from './components/common/ToastContainer';
import { LogViewer } from './components/common/LogViewer';
import { ErrorBoundary } from './components/common/ErrorBoundary';
import { formatFileSize } from './utils/format';
import { FolderOpen, Clock, GitCompare, GitBranch, Minimize2, Settings, Terminal } from 'lucide-react';
import './styles/themes.css';
import './styles/animations.css';

import type { ArchiveState } from './stores/archiveStore';
type View = ArchiveState['view'];

export default function App() {
  const { fetchArchives, fetchStatistics, statistics, view, setView } = useArchiveStore(
    (s) => ({
      fetchArchives: s.fetchArchives,
      fetchStatistics: s.fetchStatistics,
      statistics: s.statistics,
      view: s.view,
      setView: s.setView,
    }),
    shallow,
  );
  const [isMini, setIsMini] = useState(false);
  const [showSettings, setShowSettings] = useState(false);
  const [showLogViewer, setShowLogViewer] = useState(false);
  const [showEnhancedDiff, setShowEnhancedDiff] = useState(false);

  // Use refs for latest state in event handlers to avoid re-registering listeners
  const showSettingsRef = useRef(showSettings);
  const showLogViewerRef = useRef(showLogViewer);
  showSettingsRef.current = showSettings;
  showLogViewerRef.current = showLogViewer;

  useEffect(() => {
    fetchArchives();
    fetchStatistics();
  }, [fetchArchives, fetchStatistics]);

  // 设置事件监听 — 使用 getState() 避免依赖变化导致重复注册
  useEffect(() => {
    const cleanup = useArchiveStore.getState().setupEventListeners();
    return cleanup;
  }, []);

  // 统一使用 store 的 view 状态
  const handleViewChange = useCallback((newView: View) => {
    if (newView === 'enhanced-diff') {
      setShowEnhancedDiff(true);
      setView('diff');
    } else {
      setShowEnhancedDiff(false);
      setView(newView as 'list' | 'timeline' | 'diff' | 'graph' | 'mini');
    }
  }, [setView]);

  // 全局键盘快捷键 — 使用 ref 避免频繁重新注册
  useEffect(() => {
    const handleKeyDown = (e: KeyboardEvent) => {
      // Ctrl/Cmd + 数字切换视图
      if ((e.ctrlKey || e.metaKey) && !e.shiftKey) {
        switch (e.key) {
          case '1':
            e.preventDefault();
            handleViewChange('list');
            break;
          case '2':
            e.preventDefault();
            handleViewChange('timeline');
            break;
          case '3':
            e.preventDefault();
            handleViewChange('diff');
            break;
          case '4':
            e.preventDefault();
            handleViewChange('graph');
            break;
          case '5':
            e.preventDefault();
            handleViewChange('enhanced-diff');
            break;
          case ',':
            e.preventDefault();
            setShowSettings(s => !s);
            break;
          case 'l':
            e.preventDefault();
            setShowLogViewer(s => !s);
            break;
        }
      }
      // Escape 关闭弹窗
      if (e.key === 'Escape') {
        if (showSettingsRef.current) setShowSettings(false);
        else if (showLogViewerRef.current) setShowLogViewer(false);
      }
    };

    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [handleViewChange]);

  const navItems = [
    { id: 'list' as View, label: '存档管理', icon: FolderOpen, shortcut: 'Ctrl+1' },
    { id: 'timeline' as View, label: '时间轴', icon: Clock, shortcut: 'Ctrl+2' },
    { id: 'diff' as View, label: '版本对比', icon: GitCompare, shortcut: 'Ctrl+3' },
    { id: 'enhanced-diff' as View, label: '增强对比', icon: GitCompare, shortcut: 'Ctrl+5' },
    { id: 'graph' as View, label: '迭代图谱', icon: GitBranch, shortcut: 'Ctrl+4' },
  ];

  // Mini mode
  if (isMini) {
    return (
      <div className="fixed bottom-4 right-4 z-50">
        <MiniMode onExpand={() => setIsMini(false)} />
      </div>
    );
  }

  return (
    <>
      <div className="flex h-screen flex-col bg-gray-50 transition-colors dark:bg-gray-900 md:flex-row">
        {/* Sidebar */}
        <div className="flex w-full flex-shrink-0 flex-col border-b border-gray-200 bg-white transition-colors dark:border-gray-700 dark:bg-gray-800 md:h-full md:w-56 md:border-b-0 md:border-r">
          {/* Logo */}
          <div className="border-b border-gray-100 p-3 dark:border-gray-700 md:p-4">
            <div className="flex items-center justify-between gap-3">
              <div className="flex min-w-0 items-center gap-2">
                <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-primary-500 to-primary-600 flex items-center justify-center shadow-sm">
                  <span className="text-white text-sm font-bold">追</span>
                </div>
                <div className="min-w-0">
                  <h1 className="font-bold text-sm dark:text-white">DocDist</h1>
                  <p className="text-xs text-gray-400">文件历史管理</p>
                </div>
              </div>

              <div className="flex items-center gap-1 md:hidden">
                <ThemeToggle />
                <button
                  onClick={() => setShowLogViewer(true)}
                  aria-label="日志"
                  className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
                  title="日志"
                >
                  <Terminal className="w-4 h-4" />
                </button>
                <button
                  onClick={() => setShowSettings(true)}
                  aria-label="设置"
                  className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
                  title="设置"
                >
                  <Settings className="w-4 h-4" />
                </button>
                <button
                  onClick={() => setIsMini(true)}
                  aria-label="迷你模式"
                  className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
                  title="迷你模式"
                >
                  <Minimize2 className="w-4 h-4" />
                </button>
              </div>
            </div>
          </div>

          {/* Navigation */}
          <nav className="flex gap-1 overflow-x-auto p-2 md:block md:flex-1 md:space-y-1 md:p-3">
            {navItems.map(({ id, label, icon: Icon, shortcut }) => {
              const isActive = id === 'enhanced-diff'
                ? (view === 'enhanced-diff' || (view === 'diff' && showEnhancedDiff))
                : (view === id && (id !== 'diff' || !showEnhancedDiff));
              return (
              <button
                key={id}
                onClick={() => handleViewChange(id)}
                aria-current={isActive ? 'page' : undefined}
                className={`group flex min-w-max items-center gap-2.5 rounded-lg px-3 py-2 text-sm transition-all duration-150 md:w-full ${
                  isActive
                    ? 'bg-primary-50 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 font-medium shadow-sm'
                    : 'text-gray-600 dark:text-gray-400 hover:bg-gray-50 dark:hover:bg-gray-700 hover:text-gray-800 dark:hover:text-gray-200'
                }`}
              >
                <Icon className="w-4 h-4" />
                <span className="flex-1 text-left">{label}</span>
                <span className="text-[10px] text-gray-300 dark:text-gray-600 opacity-0 group-hover:opacity-100 transition-opacity">
                  {shortcut}
                </span>
              </button>
              );
            })}
          </nav>

          {/* Stats */}
          {statistics && (
            <div className="hidden border-t border-gray-100 p-3 dark:border-gray-700 md:block">
              <div className="text-xs text-gray-400 space-y-1">
                <p className="flex items-center gap-1.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-green-400" />
                  存档 {statistics.total_archives}
                </p>
                <p className="flex items-center gap-1.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-blue-400" />
                  文件 {statistics.unique_files}
                </p>
                <p className="flex items-center gap-1.5">
                  <span className="w-1.5 h-1.5 rounded-full bg-purple-400" />
                  {formatFileSize(statistics.total_size)}
                </p>
              </div>
            </div>
          )}

          {/* Bottom actions */}
          <div className="hidden items-center justify-between border-t border-gray-100 p-3 dark:border-gray-700 md:flex">
            <ThemeToggle />
            <div className="flex items-center gap-1">
              <button
                onClick={() => setShowLogViewer(true)}
                aria-label="日志"
                className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
                title="日志 (⌘L)"
              >
                <Terminal className="w-4 h-4" />
              </button>
              <button
                onClick={() => setShowSettings(true)}
                aria-label="设置"
                className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
                title="设置 (⌘,)"
              >
                <Settings className="w-4 h-4" />
              </button>
              <button
                onClick={() => setIsMini(true)}
                aria-label="迷你模式"
                className="p-2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
                title="迷你模式"
              >
                <Minimize2 className="w-4 h-4" />
              </button>
            </div>
          </div>
        </div>

        {/* Main content */}
        <div className="flex-1 flex flex-col overflow-hidden">
          <div className="flex-1 overflow-auto p-3 animate-fade-in sm:p-6">
            <ErrorBoundary>
              <Suspense fallback={<div className="flex items-center justify-center h-full"><div className="animate-spin w-6 h-6 border-2 border-primary-400 border-t-transparent rounded-full" /></div>}>
                {view === 'list' && <ArchiveList />}
                {view === 'timeline' && <TimelineView />}
                {view === 'diff' && (
                  showEnhancedDiff ? <EnhancedDiffViewer /> : <DiffViewer />
                )}
                {view === 'enhanced-diff' && <EnhancedDiffViewer />}
                {view === 'graph' && <IterationGraph />}
              </Suspense>
            </ErrorBoundary>
          </div>
        </div>

        {/* Right sidebar — Watcher Panel */}
        <div className="hidden lg:block w-72 bg-gray-50 dark:bg-gray-900 border-l border-gray-200 dark:border-gray-700 overflow-y-auto p-3 space-y-3 transition-colors">
          <WatcherPanel />
        </div>
      </div>

      {/* Modals */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
      {showLogViewer && <LogViewer onClose={() => setShowLogViewer(false)} />}

      {/* Toast notifications */}
      <ToastContainer />
    </>
  );
}
