import { useEffect, useState } from 'react';
import { useArchiveStore } from './stores/archiveStore';
import { ArchiveList } from './components/archive/ArchiveList';
import { TimelineView } from './components/timeline/TimelineView';
import { DiffViewer } from './components/diff/DiffViewer';
import { IterationGraph } from './components/graph/IterationGraph';
import { MiniMode } from './components/mini/MiniMode';
import { FolderOpen, Clock, GitCompare, GitBranch, Minimize2 } from 'lucide-react';

type View = 'list' | 'timeline' | 'diff' | 'graph';

export default function App() {
  const { fetchArchives, fetchStatistics, statistics } = useArchiveStore();
  const [isMini, setIsMini] = useState(false);
  const [activeView, setActiveView] = useState<View>('list');

  useEffect(() => {
    fetchArchives();
    fetchStatistics();
  }, [fetchArchives, fetchStatistics]);

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
              className={`w-full flex items-center gap-3 px-3 py-2.5 rounded-lg text-sm transition
                ${activeView === id
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
            <div className="grid grid-cols-2 gap-2 text-center">
              <div className="p-2 bg-gray-50 rounded-lg">
                <p className="text-lg font-bold text-primary-600">{statistics.total_archives}</p>
                <p className="text-xs text-gray-400">存档</p>
              </div>
              <div className="p-2 bg-gray-50 rounded-lg">
                <p className="text-lg font-bold text-primary-600">{statistics.unique_files}</p>
                <p className="text-xs text-gray-400">文件</p>
              </div>
            </div>
          </div>
        )}

        {/* Mini mode toggle */}
        <div className="p-3 border-t border-gray-100">
          <button
            onClick={() => setIsMini(true)}
            className="w-full flex items-center justify-center gap-2 px-3 py-2 text-sm text-gray-500 hover:text-gray-700 hover:bg-gray-50 rounded-lg transition"
          >
            <Minimize2 className="w-4 h-4" />
            Mini 模式
          </button>
        </div>
      </div>

      {/* Main Content */}
      <div className="flex-1 flex flex-col min-w-0">
        {/* Content Area */}
        <div className="flex-1 overflow-hidden">
          {activeView === 'list' && <ArchiveList />}
          {activeView === 'timeline' && <TimelineView />}
          {activeView === 'diff' && <DiffViewer />}
          {activeView === 'graph' && <IterationGraph />}
        </div>
      </div>
    </div>
  );
}
