import { useEffect, useState } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import {
  Eye,
  EyeOff,
  Plus,
  X,
  FolderOpen,
  Activity,
  AlertCircle,
} from 'lucide-react';
import { open } from '@tauri-apps/api/dialog';

export function WatcherPanel() {
  const {
    watcherStatus,
    fileEvents,
    startWatcher,
    stopWatcher,
    addWatcherPath,
    removeWatcherPath,
    fetchWatcherStatus,
    fetchConfig,
  } = useArchiveStore();

  const [showEvents, setShowEvents] = useState(false);

  useEffect(() => {
    fetchWatcherStatus();
    fetchConfig();
  }, [fetchWatcherStatus, fetchConfig]);

  const handleAddPath = async () => {
    const selected = await open({
      directory: true,
      multiple: false,
      title: '选择监控目录',
    });
    if (selected) {
      await addWatcherPath(selected as string);
    }
  };

  const handleToggleWatcher = async () => {
    if (watcherStatus.running) {
      await stopWatcher();
    } else {
      await startWatcher(watcherStatus.paths.length > 0 ? watcherStatus.paths : []);
    }
  };

  const recentEvents = fileEvents.slice(0, 10);

  return (
    <div className="bg-white rounded-xl border border-gray-200 overflow-hidden">
      {/* Header */}
      <div className="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
        <div className="flex items-center gap-2">
          <Activity className="w-4 h-4 text-gray-500" />
          <h3 className="text-sm font-semibold text-gray-700">文件监控</h3>
        </div>
        <button
          onClick={handleToggleWatcher}
          className={`flex items-center gap-1.5 px-3 py-1.5 rounded-lg text-xs font-medium transition ${
            watcherStatus.running
              ? 'bg-green-50 text-green-600 hover:bg-green-100'
              : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
          }`}
        >
          {watcherStatus.running ? (
            <>
              <Eye className="w-3.5 h-3.5" />
              监控中
            </>
          ) : (
            <>
              <EyeOff className="w-3.5 h-3.5" />
              已停止
            </>
          )}
        </button>
      </div>

      {/* Status indicator */}
      <div className="px-4 py-2 bg-gray-50 border-b border-gray-100">
        <div className="flex items-center gap-2">
          <div
            className={`w-2 h-2 rounded-full ${
              watcherStatus.running
                ? 'bg-green-500 animate-pulse'
                : 'bg-gray-300'
            }`}
          />
          <span className="text-xs text-gray-500">
            {watcherStatus.running
              ? `正在监控 ${watcherStatus.paths.length} 个路径`
              : '监控未启动'}
          </span>
        </div>
      </div>

      {/* Watched paths */}
      <div className="p-3 space-y-2">
        <div className="flex items-center justify-between">
          <span className="text-xs font-medium text-gray-500">监控目录</span>
          <button
            onClick={handleAddPath}
            className="p-1 hover:bg-gray-100 rounded transition"
            title="添加目录"
          >
            <Plus className="w-3.5 h-3.5 text-gray-400" />
          </button>
        </div>

        {watcherStatus.paths.length === 0 ? (
          <p className="text-xs text-gray-400 py-2 text-center">
            暂无监控目录，点击上方 + 添加
          </p>
        ) : (
          <div className="space-y-1">
            {watcherStatus.paths.map((path) => (
              <div
                key={path}
                className="flex items-center gap-2 px-2 py-1.5 bg-gray-50 rounded-lg group"
              >
                <FolderOpen className="w-3.5 h-3.5 text-gray-400 flex-shrink-0" />
                <span className="text-xs text-gray-600 truncate flex-1" title={path}>
                  {path}
                </span>
                <button
                  onClick={() => removeWatcherPath(path)}
                  className="p-0.5 opacity-0 group-hover:opacity-100 hover:bg-red-100 rounded transition"
                  title="移除"
                >
                  <X className="w-3 h-3 text-red-400" />
                </button>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Recent events */}
      {watcherStatus.running && (
        <div className="border-t border-gray-100">
          <button
            onClick={() => setShowEvents(!showEvents)}
            className="w-full px-4 py-2 flex items-center justify-between hover:bg-gray-50 transition"
          >
            <span className="text-xs font-medium text-gray-500">
              最近事件 ({fileEvents.length})
            </span>
            <span className="text-xs text-gray-400">
              {showEvents ? '收起' : '展开'}
            </span>
          </button>

          {showEvents && (
            <div className="px-3 pb-3 max-h-40 overflow-y-auto space-y-1">
              {recentEvents.length === 0 ? (
                <p className="text-xs text-gray-400 text-center py-1">暂无事件</p>
              ) : (
                recentEvents.map((evt, i) => (
                  <div key={i} className="flex items-start gap-2 px-2 py-1">
                    {evt.event_type === 'auto_archive_triggered' ? (
                      <AlertCircle className="w-3 h-3 text-blue-500 mt-0.5 flex-shrink-0" />
                    ) : (
                      <Activity className="w-3 h-3 text-gray-400 mt-0.5 flex-shrink-0" />
                    )}
                    <div className="min-w-0">
                      <p className="text-xs text-gray-600 truncate" title={evt.path}>
                        {evt.path.split('/').pop() || evt.path}
                      </p>
                      <p className="text-[10px] text-gray-400">{evt.timestamp}</p>
                    </div>
                  </div>
                ))
              )}
            </div>
          )}
        </div>
      )}
    </div>
  );
}
