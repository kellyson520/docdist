import { useEffect } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import { formatFileSize, formatDate } from '../../utils/format';
import { TagBadge } from '../common/TagBadge';
import { Clock, RotateCcw, Trash2, FileText } from 'lucide-react';

export function TimelineView() {
  const { timeline, selectedArchive, fetchTimeline, restoreArchive, deleteArchive } = useArchiveStore();

  useEffect(() => {
    if (selectedArchive) {
      fetchTimeline(selectedArchive.file_path);
    }
  }, [selectedArchive, fetchTimeline]);

  if (!selectedArchive) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400">
        <Clock className="w-12 h-12 mb-3 opacity-30" />
        <p className="text-sm">请先选择一个文件</p>
        <p className="text-xs mt-1">在存档列表中选择文件查看时间轴</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 p-4 border-b border-gray-100">
        <Clock className="w-5 h-5 text-primary-500" />
        <h2 className="font-semibold text-lg">时间轴</h2>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        <div className="mb-4 p-3 bg-gray-50 rounded-lg">
          <p className="text-sm font-medium truncate">{selectedArchive.file_name}</p>
          <p className="text-xs text-gray-500 truncate">{selectedArchive.file_path}</p>
        </div>

        <div className="relative pl-6">
          {/* Vertical line */}
          <div className="absolute left-2 top-0 bottom-0 w-0.5 bg-gray-200" />

          {timeline.map((archive, index) => (
            <div key={archive.id} className="relative mb-6 animate-slide-in" style={{ animationDelay: `${index * 50}ms` }}>
              {/* Dot */}
              <div className={`absolute -left-4 top-1.5 w-3 h-3 rounded-full border-2
                ${index === 0 ? 'bg-primary-500 border-primary-300' : 'bg-white border-gray-300'}`}
              />

              <div className={`p-3 rounded-lg border transition-all
                ${selectedArchive.id === archive.id
                  ? 'border-primary-300 bg-primary-50'
                  : 'border-gray-200 bg-white hover:border-gray-300'
                }`}
              >
                <div className="flex items-center justify-between">
                  <div className="flex items-center gap-2">
                    <FileText className="w-4 h-4 text-gray-400" />
                    <span className="text-xs text-gray-500">{formatDate(archive.created_at)}</span>
                  </div>
                  <div className="flex gap-1">
                    <button
                      onClick={() => restoreArchive(archive.id)}
                      className="p-1 hover:bg-gray-100 rounded"
                      title="恢复此版本"
                    >
                      <RotateCcw className="w-3.5 h-3.5 text-gray-400" />
                    </button>
                    <button
                      onClick={() => deleteArchive(archive.id)}
                      className="p-1 hover:bg-red-50 rounded"
                      title="删除"
                    >
                      <Trash2 className="w-3.5 h-3.5 text-gray-400" />
                    </button>
                  </div>
                </div>

                <div className="mt-1 text-xs text-gray-500">
                  {formatFileSize(archive.file_size)} · {archive.chunk_count} 块
                </div>

                {archive.note && (
                  <p className="mt-1 text-xs text-gray-600">{archive.note}</p>
                )}

                {archive.tags.length > 0 && (
                  <div className="mt-1 flex flex-wrap gap-1">
                    {archive.tags.map((tag) => <TagBadge key={tag} tag={tag} />)}
                  </div>
                )}
              </div>
            </div>
          ))}

          {timeline.length === 0 && (
            <div className="text-center text-gray-400 py-8">
              <p className="text-sm">暂无历史记录</p>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
