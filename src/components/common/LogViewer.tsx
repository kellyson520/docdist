/**
 * 日志查看器面板 — 实时日志流 + 过滤 + 导出
 */
import { useEffect, useState, useRef } from 'react';
import { Logger, type LogEntry, type LogLevel } from '../../utils/logger';
import { Terminal, Download, Trash2, Filter, ChevronDown, ChevronUp } from 'lucide-react';

const levelColors: Record<LogLevel, string> = {
  debug: 'text-gray-400',
  info: 'text-blue-500',
  warn: 'text-amber-500',
  error: 'text-red-500',
};

const levelBg: Record<LogLevel, string> = {
  debug: '',
  info: '',
  warn: 'bg-amber-50',
  error: 'bg-red-50',
};

export function LogViewer({ onClose }: { onClose: () => void }) {
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
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
      <div className="bg-white rounded-xl shadow-2xl w-[800px] h-[600px] flex flex-col overflow-hidden">
        {/* Header */}
        <div className="px-4 py-3 border-b border-gray-100 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Terminal className="w-4 h-4 text-gray-500" />
            <h3 className="text-sm font-semibold">日志查看器</h3>
            <span className="text-xs text-gray-400 bg-gray-100 px-2 py-0.5 rounded-full">
              {filtered.length}
            </span>
          </div>
          <div className="flex items-center gap-1">
            <button
              onClick={handleExport}
              className="p-1.5 hover:bg-gray-100 rounded-lg transition"
              title="导出日志"
            >
              <Download className="w-4 h-4 text-gray-400" />
            </button>
            <button
              onClick={() => { Logger.clear(); setLogs([]); }}
              className="p-1.5 hover:bg-gray-100 rounded-lg transition"
              title="清空日志"
            >
              <Trash2 className="w-4 h-4 text-gray-400" />
            </button>
            <button
              onClick={onClose}
              className="p-1.5 hover:bg-gray-100 rounded-lg transition text-xs text-gray-500 ml-2"
            >
              关闭
            </button>
          </div>
        </div>

        {/* Filters */}
        <div className="px-4 py-2 border-b border-gray-50 flex items-center gap-2">
          <Filter className="w-3.5 h-3.5 text-gray-400" />
          {(['all', 'debug', 'info', 'warn', 'error'] as const).map(level => (
            <button
              key={level}
              onClick={() => setFilter(level)}
              className={`px-2 py-0.5 text-xs rounded-full transition ${
                filter === level
                  ? 'bg-primary-100 text-primary-600 font-medium'
                  : 'text-gray-400 hover:bg-gray-100'
              }`}
            >
              {level === 'all' ? '全部' : level.toUpperCase()}
            </button>
          ))}
          <div className="flex-1" />
          <label className="flex items-center gap-1 text-xs text-gray-400 cursor-pointer">
            <input
              type="checkbox"
              checked={autoScroll}
              onChange={(e) => setAutoScroll(e.target.checked)}
              className="rounded border-gray-300 w-3 h-3"
            />
            自动滚动
          </label>
        </div>

        {/* Log entries */}
        <div className="flex-1 overflow-y-auto font-mono text-xs">
          {filtered.length === 0 ? (
            <div className="flex items-center justify-center h-full text-gray-300">
              暂无日志
            </div>
          ) : (
            filtered.map(entry => (
              <div
                key={entry.id}
                className={`px-4 py-1 border-b border-gray-50 hover:bg-gray-50 cursor-pointer ${levelBg[entry.level]}`}
                onClick={() => toggleExpand(entry.id)}
              >
                <div className="flex items-start gap-2">
                  <span className="text-gray-300 w-16 flex-shrink-0">
                    {entry.timestamp.slice(11, 19)}
                  </span>
                  <span className={`w-10 flex-shrink-0 font-semibold ${levelColors[entry.level]}`}>
                    {entry.level.toUpperCase()}
                  </span>
                  <span className="text-gray-400 w-24 flex-shrink-0 truncate">
                    [{entry.source}]
                  </span>
                  <span className="text-gray-700 flex-1">{entry.message}</span>
                  {entry.data !== undefined && (
                    expanded.has(entry.id) ? (
                      <ChevronUp className="w-3 h-3 text-gray-300 flex-shrink-0" />
                    ) : (
                      <ChevronDown className="w-3 h-3 text-gray-300 flex-shrink-0" />
                    )
                  )}
                </div>
                {expanded.has(entry.id) && entry.data !== undefined && (
                  <pre className="mt-1 ml-28 p-2 bg-gray-100 rounded text-[10px] text-gray-600 overflow-x-auto max-h-32">
                    {typeof entry.data === 'string' ? entry.data : JSON.stringify(entry.data, null, 2)}
                  </pre>
                )}
              </div>
            ))
          )}
          <div ref={bottomRef} />
        </div>
      </div>
    </div>
  );
}
