import React from 'react';
import { Plus, Minus } from 'lucide-react';
import type { EnhancedDiffResult } from '../../types/diff';

interface DiffDetailViewProps {
  result: EnhancedDiffResult;
}

// 标准化换行符（Windows \r\n -> Unix \n）
const normalizeLineEndings = (text: string) =>
  text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

export function DiffDetailView({ result }: DiffDetailViewProps) {
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
              {hunk.changes.map((change, idx) => (
                <div
                  key={idx}
                  className={`flex px-3 py-0.5 ${
                    change.change_type === 'add'
                      ? 'bg-green-50 dark:bg-green-900/20 text-green-800 dark:text-green-300'
                      : change.change_type === 'delete'
                      ? 'bg-red-50 dark:bg-red-900/20 text-red-800 dark:text-red-300'
                      : 'bg-white dark:bg-gray-800 text-gray-700 dark:text-gray-300'
                  }`}
                >
                  <span className="w-8 text-right pr-2 text-gray-400 dark:text-gray-600 select-none">
                    {change.old_line || ''}
                  </span>
                  <span className="w-8 text-right pr-2 text-gray-400 dark:text-gray-600 select-none">
                    {change.new_line || ''}
                  </span>
                  <span className="w-5 text-center font-bold">
                    {change.change_type === 'add' ? '+' : change.change_type === 'delete' ? '-' : ' '}
                  </span>
                  <span className="whitespace-pre">{normalizeLineEndings(change.content)}</span>
                </div>
              ))}
            </div>
          </div>
        ))}
      </div>
    </div>
  );
}
