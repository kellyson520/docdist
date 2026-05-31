import React, { useState } from 'react';
import { Plus, Minus, Edit, ChevronDown, ChevronUp, Move, Type, File } from 'lucide-react';
import type { DiffSummary, ChangeSummary } from '../../types/diff';

interface SummaryViewProps {
  summary: DiffSummary;
}

export function SummaryView({ summary }: SummaryViewProps) {
  return (
    <div className="space-y-4">
      {/* 变更分布 */}
      <div className="grid grid-cols-5 gap-4">
        <StatCard label="新增" value={summary.change_distribution.additions} color="green" />
        <StatCard label="删除" value={summary.change_distribution.deletions} color="red" />
        <StatCard label="修改" value={summary.change_distribution.modifications} color="yellow" />
        <StatCard label="移动" value={summary.change_distribution.moves} color="blue" />
        <StatCard label="重命名" value={summary.change_distribution.renames} color="purple" />
      </div>

      {/* 变更列表 */}
      <div className="space-y-2">
        <h4 className="font-semibold text-gray-700 dark:text-gray-300">变更详情</h4>
        {summary.changes.map((change) => (
          <ChangeItem key={change.id} change={change} />
        ))}
      </div>
    </div>
  );
}

function StatCard({ label, value, color }: { label: string; value: number; color: string }) {
  const colorMap: Record<string, string> = {
    green: 'bg-green-50 dark:bg-green-900/20 text-green-700 dark:text-green-400 border-green-200 dark:border-green-800',
    red: 'bg-red-50 dark:bg-red-900/20 text-red-700 dark:text-red-400 border-red-200 dark:border-red-800',
    yellow: 'bg-yellow-50 dark:bg-yellow-900/20 text-yellow-700 dark:text-yellow-400 border-yellow-200 dark:border-yellow-800',
    blue: 'bg-blue-50 dark:bg-blue-900/20 text-blue-700 dark:text-blue-400 border-blue-200 dark:border-blue-800',
    purple: 'bg-purple-50 dark:bg-purple-900/20 text-purple-700 dark:text-purple-400 border-purple-200 dark:border-purple-800',
  };

  return (
    <div className={`p-3 rounded-lg border ${colorMap[color]}`}>
      <div className="text-2xl font-bold">{value}</div>
      <div className="text-sm">{label}</div>
    </div>
  );
}

function ChangeItem({ change }: { change: ChangeSummary }) {
  const [expanded, setExpanded] = useState(false);

  const typeConfig: Record<string, { icon: React.ComponentType<{ className?: string }>; color: string }> = {
    Addition: { icon: Plus, color: 'text-green-500' },
    Deletion: { icon: Minus, color: 'text-red-500' },
    Modification: { icon: Edit, color: 'text-yellow-500' },
    Move: { icon: Move, color: 'text-blue-500' },
    Rename: { icon: Type, color: 'text-purple-500' },
    FormatChange: { icon: File, color: 'text-gray-500' },
    EncodingChange: { icon: File, color: 'text-orange-500' },
    Replacement: { icon: Edit, color: 'text-cyan-500' },
  };

  const config = typeConfig[change.change_type] || typeConfig.Modification;
  const Icon = config.icon;

  return (
    <div className="border border-gray-200 dark:border-gray-700 rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-3 p-3 hover:bg-gray-50 dark:hover:bg-gray-700/50"
      >
        <Icon className={`w-4 h-4 ${config.color}`} />
        <div className="flex-1 text-left">
          <div className="font-medium dark:text-gray-200">{change.description}</div>
          <div className="text-sm text-gray-500 dark:text-gray-400">
            第 {change.location.start_line} - {change.location.end_line} 行
            {change.location.region_description && (
              <span className="ml-2">• {change.location.region_description}</span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-sm text-gray-500 dark:text-gray-400">{change.line_count} 行</span>
          {expanded ? <ChevronUp className="w-4 h-4 text-gray-400 dark:text-gray-500" /> : <ChevronDown className="w-4 h-4 text-gray-400 dark:text-gray-500" />}
        </div>
      </button>

      {expanded && change.snippet && (
        <div className="px-3 pb-3">
          <pre className="p-2 bg-gray-100 dark:bg-gray-800 rounded text-xs font-mono overflow-x-auto text-gray-700 dark:text-gray-300">
            {change.snippet}
          </pre>
        </div>
      )}
    </div>
  );
}
