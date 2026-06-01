import { useArchiveStore } from '../../stores/archiveStore';
import { shallow } from 'zustand/shallow';
import { X, GitCompare, Copy, Check, ChevronDown, ChevronUp } from 'lucide-react';
import { useState, useCallback, useEffect, useRef } from 'react';
import { useVirtualizer } from '@tanstack/react-virtual';

export function DiffViewer() {
  const { diffResult, clearDiff, loading } = useArchiveStore(
    (s) => ({ diffResult: s.diffResult, clearDiff: s.clearDiff, loading: s.loading }),
    shallow
  );
  const [copied, setCopied] = useState(false);
  const [collapsedHunks, setCollapsedHunks] = useState<Set<number>>(new Set());
  const [showStats, setShowStats] = useState(true);

  const copyTimerRef = useRef<ReturnType<typeof setTimeout> | null>(null);

  useEffect(() => {
    return () => {
      if (copyTimerRef.current) clearTimeout(copyTimerRef.current);
    };
  }, []);

  const handleCopy = useCallback(async () => {
    if (!diffResult) return;
    
    const text = diffResult.hunks.flatMap(hunk => 
      hunk.changes.map(change => {
        const prefix = change.change_type === 'add' ? '+' : change.change_type === 'delete' ? '-' : ' ';
        return `${prefix} ${change.content}`;
      })
    ).join('\n');

    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      copyTimerRef.current = setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('Failed to copy:', err);
    }
  }, [diffResult]);

  const toggleHunk = useCallback((index: number) => {
    setCollapsedHunks(prev => {
      const next = new Set(prev);
      if (next.has(index)) {
        next.delete(index);
      } else {
        next.add(index);
      }
      return next;
    });
  }, []);

  const expandAll = useCallback(() => {
    setCollapsedHunks(new Set());
  }, []);

  const collapseAll = useCallback(() => {
    if (!diffResult) return;
    setCollapsedHunks(new Set(diffResult.hunks.map((_, i) => i)));
  }, [diffResult]);

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400 dark:text-gray-500">
        <div className="animate-spin w-5 h-5 border-2 border-primary-400 border-t-transparent rounded-full mr-2" />
        对比中...
      </div>
    );
  }

  if (!diffResult) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400 dark:text-gray-500">
        <GitCompare className="w-12 h-12 mb-3 opacity-30" />
        <p className="text-sm">选择两个存档进行对比</p>
        <p className="text-xs mt-1">在存档列表中选择一个存档，然后点击另一个的「对比」按钮</p>
      </div>
    );
  }

  const stats = diffResult.stats;
  const totalChanges = stats.additions + stats.deletions;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <GitCompare className="w-5 h-5 text-primary-500" />
          <h2 className="font-semibold text-lg dark:text-white">版本对比</h2>
          
          {/* Stats Toggle */}
          <button
            onClick={() => setShowStats(!showStats)}
            className="ml-2 px-2 py-1 text-xs bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 text-gray-600 dark:text-gray-300 rounded transition"
          >
            {showStats ? '隐藏统计' : '显示统计'}
          </button>
        </div>

        <div className="flex items-center gap-2">
          {/* Copy Button */}
          <button
            onClick={handleCopy}
            className="flex items-center gap-1 px-2 py-1.5 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
            title="复制差异内容"
          >
            {copied ? (
              <>
                <Check className="w-3.5 h-3.5 text-green-500" />
                已复制
              </>
            ) : (
              <>
                <Copy className="w-3.5 h-3.5" />
                复制
              </>
            )}
          </button>

          {/* Expand/Collapse */}
          <button
            onClick={expandAll}
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
            title="展开所有"
          >
            <ChevronDown className="w-4 h-4 text-gray-400 dark:text-gray-500" />
          </button>
          <button
            onClick={collapseAll}
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
            title="折叠所有"
          >
            <ChevronUp className="w-4 h-4 text-gray-400 dark:text-gray-500" />
          </button>

          {/* Close */}
          <button
            onClick={clearDiff}
            aria-label="关闭对比"
            className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
          >
            <X className="w-4 h-4 text-gray-400 dark:text-gray-500" />
          </button>
        </div>
      </div>

      {/* Stats Bar */}
      {showStats && (
        <div className="flex items-center gap-4 px-4 py-2 bg-gray-50 dark:bg-gray-800/50 border-b border-gray-200 dark:border-gray-700">
          <div className="flex items-center gap-2">
            <span className="flex items-center gap-1 text-xs">
              <span className="w-3 h-3 rounded-full bg-green-500" />
              <span className="text-green-700 dark:text-green-400 font-medium">+{stats.additions}</span>
            </span>
            <span className="flex items-center gap-1 text-xs">
              <span className="w-3 h-3 rounded-full bg-red-500" />
              <span className="text-red-700 dark:text-red-400 font-medium">-{stats.deletions}</span>
            </span>
            <span className="flex items-center gap-1 text-xs">
              <span className="w-3 h-3 rounded-full bg-gray-400 dark:bg-gray-500" />
              <span className="text-gray-600 dark:text-gray-400">{stats.unchanged} 不变</span>
            </span>
          </div>

          {/* Progress Bar */}
          {totalChanges > 0 && (
            <div role="progressbar" aria-valuenow={stats.additions} aria-valuemin={0} aria-valuemax={totalChanges} aria-label={`变更进度: ${stats.additions} 新增, ${stats.deletions} 删除`} className="flex-1 h-2 bg-gray-200 dark:bg-gray-700 rounded-full overflow-hidden">
              <div
                className="h-full bg-green-500 transition-all"
                style={{ width: `${(stats.additions / totalChanges) * 100}%` }}
              />
            </div>
          )}

          <span className="text-xs text-gray-500 dark:text-gray-400">
            {diffResult.hunks.length} 个差异块
          </span>
        </div>
      )}

      {/* Diff Content */}
      <div className="flex-1 overflow-y-auto font-mono text-xs">
        {diffResult.hunks.map((hunk, hunkIdx) => {
          const isExpanded = !collapsedHunks.has(hunkIdx);
          
          return (
            <div key={hunkIdx} className="border-b border-gray-100 dark:border-gray-700">
              {/* Hunk Header */}
              <button
                onClick={() => toggleHunk(hunkIdx)}
                aria-expanded={isExpanded}
                className="w-full px-4 py-1.5 bg-gray-50 dark:bg-gray-800/50 text-gray-600 dark:text-gray-400 text-xs border-y border-gray-200 dark:border-gray-700 flex items-center justify-between hover:bg-gray-100 dark:hover:bg-gray-700 transition"
              >
                <span>
                  @@ -{hunk.old_start},{hunk.old_lines} +{hunk.new_start},{hunk.new_lines} @@
                </span>
                <span className="text-gray-400 dark:text-gray-500">
                  {hunk.changes.length} 行
                </span>
              </button>

              {/* Changes - 使用虚拟化渲染优化大量数据 */}
              {isExpanded && (
                <VirtualizedChangesList changes={hunk.changes} />
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

// 虚拟化渲染 changes 列表的子组件
function VirtualizedChangesList({ changes }: { changes: Array<{ change_type: string; content: string; old_line?: number | null; new_line?: number | null }> }) {
  const parentRef = useRef<HTMLDivElement>(null);

  const virtualizer = useVirtualizer({
    count: changes.length,
    getScrollElement: () => parentRef.current,
    estimateSize: () => 24, // 每行约24px
    overscan: 20, // 预渲染20行
  });

  return (
    <div ref={parentRef} className="max-h-[400px] overflow-y-auto">
      <div
        style={{
          height: `${virtualizer.getTotalSize()}px`,
          width: '100%',
          position: 'relative',
        }}
      >
        {virtualizer.getVirtualItems().map((virtualItem) => {
          const change = changes[virtualItem.index];
          return (
            <div
              key={virtualItem.index}
              className={`absolute top-0 left-0 w-full flex px-4 py-0.5 hover:brightness-95 transition ${
                change.change_type === 'add'
                  ? 'bg-green-50 dark:bg-green-900/20 text-green-800 dark:text-green-300'
                  : change.change_type === 'delete'
                  ? 'bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-300'
                  : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300'
              }`}
              style={{
                height: `${virtualItem.size}px`,
                transform: `translateY(${virtualItem.start}px)`,
              }}
            >
              <span className="w-10 text-right pr-3 text-gray-400 dark:text-gray-600 select-none flex-shrink-0">
                {change.old_line ?? ''}
              </span>
              <span className="w-10 text-right pr-3 text-gray-400 dark:text-gray-600 select-none flex-shrink-0">
                {change.new_line ?? ''}
              </span>
              <span className="w-5 text-center select-none flex-shrink-0 font-bold">
                {change.change_type === 'add' ? '+' : change.change_type === 'delete' ? '-' : ' '}
              </span>
              <span className="whitespace-pre overflow-x-auto flex-1">
                {change.content}
              </span>
            </div>
          );
        })}
      </div>
    </div>
  );
}
