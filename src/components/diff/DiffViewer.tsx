import { useArchiveStore } from '../../stores/archiveStore';
import { shallow } from 'zustand/shallow';
import { X, GitCompare, Copy, Check, ChevronDown, ChevronUp } from 'lucide-react';
import { useState, useCallback, useEffect, useRef } from 'react';

type DiffChangeRow = {
  change_type: string;
  content: string;
  old_line?: number | null;
  new_line?: number | null;
};

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
    
    const text = diffResult.hunks.flatMap(hunk => {
      const header = `@@ -${hunk.old_start},${hunk.old_lines} +${hunk.new_start},${hunk.new_lines} @@`;
      return [
        header,
        ...hunk.changes.map(change => {
        const prefix = change.change_type === 'add' ? '+' : change.change_type === 'delete' ? '-' : ' ';
          return `${prefix}${change.content}`;
        }),
      ];
    }).join('\n');

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
      <div className="flex items-center justify-between gap-3 px-4 py-3 border-b border-gray-100 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <GitCompare className="w-5 h-5 text-primary-500" />
          <h2 className="font-semibold text-base dark:text-white">版本对比</h2>
          
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

          <span className="text-xs text-gray-500 dark:text-gray-400 tabular-nums">
            {diffResult.hunks.length} 个差异块
          </span>
        </div>
      )}

      {/* Diff Content */}
      <div className="flex-1 overflow-auto font-mono text-xs bg-white dark:bg-gray-950">
        {diffResult.hunks.length === 0 && (
          <div className="flex h-full items-center justify-center text-gray-400 dark:text-gray-500">
            <div className="text-center">
              <GitCompare className="mx-auto mb-2 h-8 w-8 opacity-30" />
              <p>两个版本没有内容差异</p>
            </div>
          </div>
        )}

        {diffResult.hunks.map((hunk, hunkIdx) => {
          const isExpanded = !collapsedHunks.has(hunkIdx);
          
          return (
            <div key={hunkIdx} className="border-b border-gray-200 dark:border-gray-800">
              {/* Hunk Header */}
              <button
                onClick={() => toggleHunk(hunkIdx)}
                aria-expanded={isExpanded}
                className="sticky top-0 z-10 w-full px-3 py-1.5 bg-blue-50 dark:bg-blue-950/40 text-blue-700 dark:text-blue-300 text-xs border-y border-blue-100 dark:border-blue-900 flex items-center justify-between hover:bg-blue-100 dark:hover:bg-blue-950 transition"
              >
                <span className="font-semibold">
                  @@ -{hunk.old_start},{hunk.old_lines} +{hunk.new_start},{hunk.new_lines} @@
                </span>
                <span className="text-blue-500 dark:text-blue-400 tabular-nums">
                  {hunk.changes.length} 行
                </span>
              </button>

              {isExpanded && (
                <div>
                  {hunk.changes.map((change, index) => (
                    <DiffLine key={`${hunkIdx}-${index}`} change={change} />
                  ))}
                </div>
              )}
            </div>
          );
        })}
      </div>
    </div>
  );
}

function DiffLine({ change }: { change: DiffChangeRow }) {
  const isAdd = change.change_type === 'add';
  const isDelete = change.change_type === 'delete';
  const prefix = isAdd ? '+' : isDelete ? '-' : ' ';
  const rowClass = isAdd
    ? 'bg-green-50 text-green-900 dark:bg-green-950/35 dark:text-green-200'
    : isDelete
    ? 'bg-red-50 text-red-900 dark:bg-red-950/35 dark:text-red-200'
    : 'bg-white text-gray-800 dark:bg-gray-950 dark:text-gray-300';
  const gutterClass = isAdd
    ? 'bg-green-100/70 text-green-700 dark:bg-green-950/60 dark:text-green-300'
    : isDelete
    ? 'bg-red-100/70 text-red-700 dark:bg-red-950/60 dark:text-red-300'
    : 'bg-gray-50 text-gray-400 dark:bg-gray-900 dark:text-gray-600';

  return (
    <div className={`grid min-w-max grid-cols-[4.5rem_4.5rem_2rem_minmax(32rem,1fr)] leading-6 hover:brightness-[0.98] ${rowClass}`}>
      <span className={`border-r border-gray-200 px-3 text-right tabular-nums select-none dark:border-gray-800 ${gutterClass}`}>
        {change.old_line ?? ''}
      </span>
      <span className={`border-r border-gray-200 px-3 text-right tabular-nums select-none dark:border-gray-800 ${gutterClass}`}>
        {change.new_line ?? ''}
      </span>
      <span className="text-center font-semibold select-none">
        {prefix}
      </span>
      <code className="whitespace-pre px-2 text-[12px]">
        {change.content || ' '}
      </code>
    </div>
  );
}
