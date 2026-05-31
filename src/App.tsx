import { useEffect, useState } from 'react';
import { useArchiveStore } from './stores/archiveStore';
import { ArchiveList } from './components/archive/ArchiveList';
import { TimelineView } from './components/timeline/TimelineView';
import { DiffViewer } from './components/diff/DiffViewer';
import { IterationGraph } from './components/graph/IterationGraph';
import { MiniMode } from './components/mini/MiniMode';
import { ThemeToggle } from './components/common/ThemeToggle';
import { WatcherPanel } from './components/watcher/WatcherPanel';
import { SettingsPanel } from './components/settings/SettingsPanel';
import { FolderOpen, Clock, GitCompare, GitBranch, Minimize2, Settings } from 'lucide-react';
import './styles/themes.css';

type View = 'list' | 'timeline' | 'diff' | 'graph';

export default function App() {
  const { fetchArchives, fetchStatistics, statistics, setupEventListeners } = useArchiveStore();
  const [isMini, setIsMini] = useState(false);
  const [activeView, setActiveView] = useState<View>('list');
  const [showSettings, setShowSettings] = useState(false);

  useEffect(() => {
    fetchArchives();
    fetchStatistics();
  }, [fetchArchives, fetchStatistics]);

  // 设置事件监听
  useEffect(() => {
    const cleanup = setupEventListeners();
    return cleanup;
  }, [setupEventListeners]);

  const navItems = [
    { id: 'list' as View, label: '存档管理', icon: FolderOpen },
    { id: 'timeline' as View, label: '时间轴', icon: Clock },
    { id: 'diff' as View, label: '版本对比', icon: GitCompare },
    { id: 'graph' as View, label: '迭代图谱', icon: GitBranch },
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
    <div className="flex h-screen bg-gray-50">
      {/* Sidebar */}
      <div className="w-56 bg-white border-r border-gray-200 flex flex-col">
        {/* Logo */}
        <div className="p-4 border-b border-gray-100">
          <div className="flex items-center gap-2">
            <div className="w-8 h-8 rounded-lg bg-gradient-to-br from-primary-500 to-primary-600 flex items-center justify-center">
              <span className="text-white text-sm font-bold">追</span>
            </div>
            <div>
              <h1 className="font-bold text-sm">DocDist</h1>
              <p className="text-xs text-gray-400">文件历史管理</p>
            </div>
          </div>
        </div>

        {/* Navigation */}
        <nav className="flex-1 p-3 space-y-1">
          {navItems.map(({ id, label, icon: Icon }) => (
            <button
              key={id}
              onClick={() => setActiveView(id)}
              className={`w-full flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm transition ${
                activeView === id
                  ? 'bg-primary-50 text-primary-600 font-medium'
                  : 'text-gray-600 hover:bg-gray-50 hover:text-gray-800'
              }`}
            >
              <Icon className="w-4 h-4" />
              {label}
            </button>
          ))}
        </nav>

        {/* Stats */}
        {statistics && (
          <div className="p-3 border-t border-gray-100">
            <div className="text-xs text-gray-400 space-y-1">
              <p>📦 存档数：{statistics.total_archives}</p>
              <p>📁 文件数：{statistics.unique_files}</p>
              <p>💾 总大小：{formatBytes(statistics.total_size)}</p>
            </div>
          </div>
        )}

        {/* Bottom actions */}
        <div className="p-3 border-t border-gray-100 flex items-center justify-between">
          <ThemeToggle />
          <div className="flex items-center gap-1">
            <button
              onClick={() => setShowSettings(true)}
              className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition"
              title="设置"
            >
              <Settings className="w-4 h-4" />
            </button>
            <button
              onClick={() => setIsMini(true)}
              className="p-2 text-gray-400 hover:text-gray-600 hover:bg-gray-100 rounded-lg transition"
              title="迷你模式"
            >
              <Minimize2 className="w-4 h-4" />
            </button>
          </div>
        </div>
      </div>

      {/* Main content */}
      <div className="flex-1 flex flex-col overflow-hidden">
        {/* Content area */}
        <div className="flex-1 overflow-auto p-6">
          {activeView === 'list' && <ArchiveList />}
          {activeView === 'timeline' && <TimelineView />}
          {activeView === 'diff' && <DiffViewer />}
          {activeView === 'graph' && <IterationGraph />}
        </div>
      </div>

      {/* Right sidebar — Watcher Panel */}
      <div className="w-72 bg-gray-50 border-l border-gray-200 overflow-y-auto p-3 space-y-3">
        <WatcherPanel />
      </div>

      {/* Settings modal */}
      {showSettings && <SettingsPanel onClose={() => setShowSettings(false)} />}
    </div>
  );
}

function formatBytes(bytes: number): string {
  if (bytes === 0) return '0 B';
  const k = 1024;
  const sizes = ['B', 'KB', 'MB', 'GB'];
  const i = Math.floor(Math.log(bytes) / Math.log(k));
  return parseFloat((bytes / Math.pow(k, i)).toFixed(1)) + ' ' + sizes[i];
}
