import React, { useState } from 'react';
import { Plus, Minus, Edit, ChevronDown, ChevronUp } from 'lucide-react';
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
    green: 'bg-green-50 text-green-700 border-green-200',
    red: 'bg-red-50 text-red-700 border-red-200',
    yellow: 'bg-yellow-50 text-yellow-700 border-yellow-200',
    blue: 'bg-blue-50 text-blue-700 border-blue-200',
    purple: 'bg-purple-50 text-purple-700 border-purple-200',
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
  };

  const config = typeConfig[change.change_type] || typeConfig.Modification;
  const Icon = config.icon;

  return (
    <div className="border rounded-lg overflow-hidden">
      <button
        onClick={() => setExpanded(!expanded)}
        className="w-full flex items-center gap-3 p-3 hover:bg-gray-50 dark:hover:bg-gray-700"
      >
        <Icon className={`w-4 h-4 ${config.color}`} />
        <div className="flex-1 text-left">
          <div className="font-medium">{change.description}</div>
          <div className="text-sm text-gray-500">
            第 {change.location.start_line} - {change.location.end_line} 行
            {change.location.region_description && (
              <span className="ml-2">• {change.location.region_description}</span>
            )}
          </div>
        </div>
        <div className="flex items-center gap-2">
          <span className="text-sm text-gray-500">{change.line_count} 行</span>
          {expanded ? <ChevronUp className="w-4 h-4" /> : <ChevronDown className="w-4 h-4" />}
        </div>
      </button>

      {expanded && change.snippet && (
        <div className="px-3 pb-3">
          <pre className="p-2 bg-gray-100 dark:bg-gray-800 rounded text-xs font-mono overflow-x-auto">
            {change.snippet}
          </pre>
        </div>
      )}
    </div>
  );
}
