import React from 'react';
import type { AffectedRegion } from '../../types/diff';

interface RegionsViewProps {
  regions: AffectedRegion[];
}

export function RegionsView({ regions }: RegionsViewProps) {
  const colorMap: Record<string, string> = {
    Addition: 'bg-green-100 text-green-700',
    Deletion: 'bg-red-100 text-red-700',
    Modification: 'bg-yellow-100 text-yellow-700',
    Move: 'bg-blue-100 text-blue-700',
    Rename: 'bg-purple-100 text-purple-700',
  };

  const labelMap: Record<string, string> = {
    Addition: '新增',
    Deletion: '删除',
    Modification: '修改',
    Move: '移动',
    Rename: '重命名',
  };

  return (
    <div className="space-y-3">
      <h4 className="font-semibold text-gray-700 dark:text-gray-300">
        受影响区域 ({regions.length})
      </h4>
      {regions.map((region, idx) => (
        <div
          key={idx}
          className="p-3 bg-gray-50 dark:bg-gray-700 rounded-lg border"
        >
          <div className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <span className="font-medium">{region.name}</span>
              <span className="px-2 py-0.5 bg-gray-200 dark:bg-gray-600 rounded text-xs">
                {region.region_type}
              </span>
            </div>
            <span className="text-sm text-gray-500 dark:text-gray-400">
              第 {region.start_line} - {region.end_line} 行
            </span>
          </div>
          <div className="mt-2 flex items-center gap-4 text-sm">
            <span className="text-gray-500 dark:text-gray-400">
              变更 {region.change_lines} 行
            </span>
            <span className={`px-2 py-0.5 rounded text-xs ${colorMap[region.change_type] || 'bg-gray-100 dark:bg-gray-700'}`}>
              {labelMap[region.change_type] || region.change_type}
            </span>
          </div>
        </div>
      ))}
    </div>
  );
}
