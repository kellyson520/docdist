import React, { useRef } from 'react';
import { Plus, Minus } from 'lucide-react';
import { useVirtualizer } from '@tanstack/react-virtual';
import type { EnhancedDiffResult } from '../../types/diff';

interface DiffDetailViewProps {
  result: EnhancedDiffResult;
}

// 标准化换行符（Windows \r\n -> Unix \n）
const normalizeLineEndings = (text: string) =>
  text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

// 虚拟化渲染 changes 列表的子组件
function VirtualizedChangesList({ changes }: { changes: Array<{ change_type: string; content: string; old_line?: number; new_line?: number }> }) {
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
              className={`absolute top-0 left-0 w-full flex px-3 py-0.5 ${
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
              <span className="w-8 text-right pr-2 text-gray-400 dark:text-gray-600 select-none">
                {change.old_line ?? ''}
              </span>
              <span className="w-8 text-right pr-2 text-gray-400 dark:text-gray-600 select-none">
                {change.new_line ?? ''}
              </span>
              <span className="w-5 text-center font-bold">
                {change.change_type === 'add' ? '+' : change.change_type === 'delete' ? '-' : ' '}
              </span>
              <span className="whitespace-pre overflow-x-auto flex-1">{normalizeLineEndings(change.content)}</span>
            </div>
          );
        })}
      </div>
    </div>
  );
}

export function DiffDetailView({ result }: DiffDetailViewProps) {
  if (!result.diff_result?.hunks?.length) {
    return (
      <div className="text-center py-8 text-gray-400 dark:text-gray-500">
        <p className="text-sm">暂无差异数据</p>
      </div>
    );
  }

  const stats = result.diff_result.stats;

  return (
    <div>
      {/* 统计概览 */}
      <div className="mb-4 p-4 bg-gray-50 dark:bg-gray-700/50 rounded-lg">
        <div className="flex items-center gap-6">
          <div className="flex items-center gap-2">
            <Plus className="w-4 h-4 text-green-500" />
            <span className="font-semibold text-green-600 dark:text-green-400">+{stats.additions}</span>
            <span className="text-gray-500 dark:text-gray-400">新增</span>
          </div>
          <div className="flex items-center gap-2">
            <Minus className="w-4 h-4 text-red-500" />
            <span className="font-semibold text-red-600 dark:text-red-400">-{stats.deletions}</span>
            <span className="text-gray-500 dark:text-gray-400">删除</span>
          </div>
          <div className="flex items-center gap-2">
            <span className="font-semibold text-gray-600 dark:text-gray-300">
              {stats.unchanged}
            </span>
            <span className="text-gray-500 dark:text-gray-400">未变</span>
          </div>
        </div>
      </div>

      {/* AI 摘要 */}
      {result.summary.ai_summary && (
        <div className="mb-4 p-4 bg-primary-50 dark:bg-primary-900/20 rounded-lg border border-primary-200 dark:border-primary-800">
          <h4 className="font-semibold text-primary-700 dark:text-primary-300 mb-2">
            📝 智能摘要
          </h4>
          <p className="text-sm text-primary-800 dark:text-primary-200">
            {result.summary.ai_summary}
          </p>
        </div>
      )}

      {/* 差异块 */}
      <div className="space-y-2">
        {result.diff_result.hunks.map((hunk, hunkIdx) => (
          <div key={hunkIdx} className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
            <div className="px-3 py-2 bg-gray-100 dark:bg-gray-700 text-left text-sm font-mono text-gray-700 dark:text-gray-300">
              @@ -{hunk.old_start},+{hunk.new_start} @@
            </div>
            <div className="font-mono text-xs">
              {/* 使用虚拟化渲染优化大量数据 */}
              <VirtualizedChangesList changes={hunk.changes} />
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
