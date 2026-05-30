import { useArchiveStore } from '../../stores/archiveStore';
import { X, GitCompare, FileText } from 'lucide-react';

export function DiffViewer() {
  const { diffResult, clearDiff, loading } = useArchiveStore();

  if (!diffResult) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400">
        <GitCompare className="w-12 h-12 mb-3 opacity-30" />
        <p className="text-sm">选择两个存档进行对比</p>
        <p className="text-xs mt-1">在存档列表中选择一个存档，然后点击另一个的「对比」按钮</p>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full text-gray-400">
        <div className="animate-spin w-5 h-5 border-2 border-primary-400 border-t-transparent rounded-full mr-2" />
        对比中...
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between p-4 border-b border-gray-100">
        <div className="flex items-center gap-2">
          <GitCompare className="w-5 h-5 text-primary-500" />
          <h2 className="font-semibold text-lg">版本对比</h2>
          <div className="flex items-center gap-3 ml-4 text-xs">
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-green-500" />
              新增 {diffResult.stats.additions}
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-red-500" />
              删除 {diffResult.stats.deletions}
            </span>
            <span className="flex items-center gap-1">
              <span className="w-2 h-2 rounded-full bg-gray-400" />
              不变 {diffResult.stats.unchanged}
            </span>
          </div>
        </div>
        <button
          onClick={clearDiff}
          className="p-1.5 hover:bg-gray-100 rounded-lg transition"
        >
          <X className="w-4 h-4 text-gray-400" />
        </button>
      </div>

      <div className="flex-1 overflow-y-auto font-mono text-xs">
        {diffResult.hunks.map((hunk, hunkIdx) => (
          <div key={hunkIdx}>
            <div className="px-4 py-1.5 bg-gray-100 text-gray-500 text-xs border-y border-gray-200">
              @@ -{hunk.old_start},{hunk.old_lines} +{hunk.new_start},{hunk.new_lines} @@
            </div>
            {hunk.changes.map((change, changeIdx) => (
              <div
                key={changeIdx}
                className={`flex px-4 py-0.5 ${
                  change.change_type === 'add'
                    ? 'bg-green-50 text-green-800'
                    : change.change_type === 'delete'
                    ? 'bg-red-50 text-red-800'
                    : 'bg-white text-gray-700'
                }`}
              >
                <span className="w-10 text-right pr-3 text-gray-400 select-none flex-shrink-0">
                  {change.old_line || ''}
                </span>
                <span className="w-10 text-right pr-3 text-gray-400 select-none flex-shrink-0">
                  {change.new_line || ''}
                </span>
                <span className="w-5 text-center select-none flex-shrink-0">
                  {change.change_type === 'add' ? '+' : change.change_type === 'delete' ? '-' : ' '}
                </span>
                <span className="whitespace-pre overflow-x-auto">{change.content}</span>
              </div>
            ))}
          </div>
        ))}
      </div>
    </div>
  );
}
