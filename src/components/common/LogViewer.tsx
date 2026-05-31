/**
 * 日志查看器面板 — 前端日志实时流 + 后端日志文件读取
 */
import { useEffect, useState, useRef, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { Logger, type LogEntry, type LogLevel } from '../../utils/logger';
import { Terminal, Download, Trash2, Filter, ChevronDown, ChevronUp, RefreshCw } from 'lucide-react';

const levelColors: Record<LogLevel, string> = {
  debug: 'text-gray-400',
  info: 'text-blue-500',
  warn: 'text-amber-500',
  error: 'text-red-500',
};

const levelBg: Record<LogLevel, string> = {
  debug: '',
  info: '',
  warn: 'bg-amber-50 dark:bg-amber-900/20',
  error: 'bg-red-50 dark:bg-red-900/20',
};

type TabType = 'frontend' | 'backend';

export function LogViewer({ onClose }: { onClose: () => void }) {
  const [activeTab, setActiveTab] = useState<TabType>('frontend');

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-2xl w-[800px] h-[600px] flex flex-col overflow-hidden">
        {/* Header */}
        <div className="px-4 py-3 border-b border-gray-100 dark:border-gray-700 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Terminal className="w-4 h-4 text-gray-500 dark:text-gray-400" />
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-200">日志查看器</h3>
          </div>
          <div className="flex items-center gap-2">
            {/* Tab 切换 */}
            <div className="flex items-center bg-gray-100 dark:bg-gray-700 rounded-lg p-0.5">
              <button
                onClick={() => setActiveTab('frontend')}
                className={`px-3 py-1 text-xs rounded-md transition ${
                  activeTab === 'frontend'
                    ? 'bg-white dark:bg-gray-600 text-gray-800 dark:text-gray-100 font-medium shadow-sm'
                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                }`}
              >
                前端日志
              </button>
              <button
                onClick={() => setActiveTab('backend')}
                className={`px-3 py-1 text-xs rounded-md transition ${
                  activeTab === 'backend'
                    ? 'bg-white dark:bg-gray-600 text-gray-800 dark:text-gray-100 font-medium shadow-sm'
                    : 'text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200'
                }`}
              >
                后端日志
              </button>
            </div>
            <button
              onClick={onClose}
              className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition text-xs text-gray-500 dark:text-gray-400 ml-2"
            >
              关闭
            </button>
          </div>
        </div>

        {activeTab === 'frontend' ? (
          <FrontendLogPanel />
        ) : (
          <BackendLogPanel />
        )}
      </div>
    </div>
  );
}

/** 前端日志面板 */
function FrontendLogPanel() {
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [filter, setFilter] = useState<LogLevel | 'all'>('all');
  const [autoScroll, setAutoScroll] = useState(true);
  const [expanded, setExpanded] = useState<Set<string>>(new Set());
  const bottomRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    setLogs(Logger.getAll());
    const unsub = Logger.subscribe((entry: LogEntry) => {
      setLogs(prev => [...prev, entry].slice(-200));
    });
    return unsub;
  }, []);

  useEffect(() => {
    if (autoScroll) {
      bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
    }
  }, [logs, autoScroll]);

  const filtered = filter === 'all' ? logs : logs.filter(l => l.level === filter);

  const toggleExpand = (id: string) => {
    setExpanded(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id);
      else next.add(id);
      return next;
    });
  };

  const handleExport = () => {
    const json = Logger.export();
    const blob = new Blob([json], { type: 'application/json' });
    const url = URL.createObjectURL(blob);
    const a = document.createElement('a');
    a.href = url;
    a.download = `docdist-logs-${new Date().toISOString().slice(0, 10)}.json`;
    a.click();
    URL.revokeObjectURL(url);
  };

  return (
    <>
      {/* Filters */}
      <div className="px-4 py-2 border-b border-gray-50 dark:border-gray-700 flex items-center gap-2">
        <Filter className="w-3.5 h-3.5 text-gray-400" />
        {(['all', 'debug', 'info', 'warn', 'error'] as const).map(level => (
          <button
            key={level}
            onClick={() => setFilter(level)}
            className={`px-2 py-0.5 text-xs rounded-full transition ${
              filter === level
                ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 font-medium'
                : 'text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
            }`}
          >
            {level === 'all' ? '全部' : level.toUpperCase()}
          </button>
        ))}
        <div className="flex-1" />
        <button
          onClick={handleExport}
          className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
          title="导出日志"
        >
          <Download className="w-4 h-4 text-gray-400" />
        </button>
        <button
          onClick={() => { Logger.clear(); setLogs([]); }}
          className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
          title="清空日志"
        >
          <Trash2 className="w-4 h-4 text-gray-400" />
        </button>
        <label className="flex items-center gap-1 text-xs text-gray-400 cursor-pointer ml-2">
          <input
            type="checkbox"
            checked={autoScroll}
            onChange={(e) => setAutoScroll(e.target.checked)}
            className="rounded border-gray-300 dark:border-gray-600 w-3 h-3"
          />
          自动滚动
        </label>
      </div>

      {/* Log entries */}
      <div className="flex-1 overflow-y-auto font-mono text-xs">
        {filtered.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-300 dark:text-gray-600">
            暂无日志
          </div>
        ) : (
          filtered.map(entry => (
            <div
              key={entry.id}
              className={`px-4 py-1 border-b border-gray-50 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50 cursor-pointer ${levelBg[entry.level]}`}
              onClick={() => toggleExpand(entry.id)}
            >
              <div className="flex items-start gap-2">
                <span className="text-gray-300 dark:text-gray-600 w-16 flex-shrink-0">
                  {entry.timestamp.slice(11, 19)}
                </span>
                <span className={`w-10 flex-shrink-0 font-semibold ${levelColors[entry.level]}`}>
                  {entry.level.toUpperCase()}
                </span>
                <span className="text-gray-400 dark:text-gray-500 w-24 flex-shrink-0 truncate">
                  [{entry.source}]
                </span>
                <span className="text-gray-700 dark:text-gray-300 flex-1">{entry.message}</span>
                {entry.data !== undefined && (
                  expanded.has(entry.id) ? (
                    <ChevronUp className="w-3 h-3 text-gray-300 dark:text-gray-600 flex-shrink-0" />
                  ) : (
                    <ChevronDown className="w-3 h-3 text-gray-300 dark:text-gray-600 flex-shrink-0" />
                  )
                )}
              </div>
              {expanded.has(entry.id) && entry.data !== undefined && (
                <pre className="mt-1 ml-28 p-2 bg-gray-100 dark:bg-gray-700 rounded text-[10px] text-gray-600 dark:text-gray-300 overflow-x-auto max-h-32">
                  {typeof entry.data === 'string' ? entry.data : JSON.stringify(entry.data, null, 2)}
                </pre>
              )}
            </div>
          ))
        )}
        <div ref={bottomRef} />
      </div>
    </>
  );
}

/** 后端日志面板 */
function BackendLogPanel() {
  const [backendLogs, setBackendLogs] = useState<string[]>([]);
  const [loading, setLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const bottomRef = useRef<HTMLDivElement>(null);

  const fetchBackendLogs = useCallback(async () => {
    setLoading(true);
    setError(null);
    try {
      const lines = await invoke<string[]>('read_log_file', { lines: 200 });
      setBackendLogs(lines);
    } catch (err) {
      setError(String(err));
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    fetchBackendLogs();
  }, [fetchBackendLogs]);

  useEffect(() => {
    bottomRef.current?.scrollIntoView({ behavior: 'smooth' });
  }, [backendLogs]);

  return (
    <>
      {/* Toolbar */}
      <div className="px-4 py-2 border-b border-gray-50 dark:border-gray-700 flex items-center gap-2">
        <span className="text-xs text-gray-400 dark:text-gray-500">
          docdist.log · {backendLogs.length} 行
        </span>
        <div className="flex-1" />
        <button
          onClick={fetchBackendLogs}
          disabled={loading}
          className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition disabled:opacity-50"
          title="刷新"
        >
          <RefreshCw className={`w-4 h-4 text-gray-400 ${loading ? 'animate-spin' : ''}`} />
        </button>
      </div>

      {/* Backend log entries */}
      <div className="flex-1 overflow-y-auto font-mono text-xs">
        {error ? (
          <div className="flex items-center justify-center h-full text-red-400">
            {error}
          </div>
        ) : backendLogs.length === 0 ? (
          <div className="flex items-center justify-center h-full text-gray-300 dark:text-gray-600">
            {loading ? '加载中...' : '暂无后端日志'}
          </div>
        ) : (
          backendLogs.map((line, i) => {
            // 根据日志级别着色
            let lineClass = 'text-gray-700 dark:text-gray-300';
            if (line.includes('ERROR') || line.includes('error')) lineClass = 'text-red-500';
            else if (line.includes('WARN') || line.includes('warn')) lineClass = 'text-amber-500';
            else if (line.includes('DEBUG') || line.includes('debug')) lineClass = 'text-gray-400 dark:text-gray-500';
            else if (line.includes('INFO') || line.includes('info')) lineClass = 'text-blue-500';

            return (
              <div
                key={i}
                className="px-4 py-0.5 border-b border-gray-50 dark:border-gray-700 hover:bg-gray-50 dark:hover:bg-gray-700/50"
              >
                <span className={lineClass}>{line}</span>
              </div>
            );
          })
        )}
        <div ref={bottomRef} />
      </div>
    </>
  );
}
