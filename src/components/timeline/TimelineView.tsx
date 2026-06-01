import { useEffect, useState, useCallback } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import { shallow } from 'zustand/shallow';
import { formatFileSize } from '../../utils/format';
import { formatSmartTime } from '../../utils/time';
import { TagBadge } from '../common/TagBadge';
import { Clock, RotateCcw, Trash2, FileText, GitCompare, Filter } from 'lucide-react';
import { ConfirmDialog } from '../common/ConfirmDialog';

type SortOrder = 'newest' | 'oldest';
type FilterTag = string | null;

export function TimelineView() {
  const { timeline, selectedArchive, fetchTimeline, restoreArchive, deleteArchive, compareArchives, setView } = useArchiveStore(
    (s) => ({
      timeline: s.timeline,
      selectedArchive: s.selectedArchive,
      fetchTimeline: s.fetchTimeline,
      restoreArchive: s.restoreArchive,
      deleteArchive: s.deleteArchive,
      compareArchives: s.compareArchives,
      setView: s.setView,
    }),
    shallow,
  );
  const [sortOrder, setSortOrder] = useState<SortOrder>('newest');
  const [filterTag, setFilterTag] = useState<FilterTag>(null);
  const [showFilters, setShowFilters] = useState(false);
  const [selectedForCompare, setSelectedForCompare] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<{ type: 'restore' | 'delete'; id: string } | null>(null);

  useEffect(() => {
    if (selectedArchive) {
      fetchTimeline(selectedArchive.file_path);
    }
  }, [selectedArchive, fetchTimeline]);

  const handleCompare = useCallback((archiveId: string) => {
    if (selectedForCompare && selectedForCompare !== archiveId) {
      compareArchives(selectedForCompare, archiveId);
      setSelectedForCompare(null);
      setView('diff');
    } else {
      setSelectedForCompare(archiveId);
    }
  }, [selectedForCompare, compareArchives, setView]);

  const handleConfirm = useCallback(() => {
    if (!confirmAction) return;
    if (confirmAction.type === 'restore') {
      restoreArchive(confirmAction.id);
    } else {
      deleteArchive(confirmAction.id);
    }
    setConfirmAction(null);
  }, [confirmAction, restoreArchive, deleteArchive]);

  // Get all unique tags from timeline
  const allTags = Array.from(new Set(timeline.flatMap(a => a.tags ?? [])));

  // Filter and sort timeline
  const filteredTimeline = timeline
    .slice()
    .filter(a => !filterTag || (a.tags ?? []).includes(filterTag))
    .sort((a, b) => {
      const dateA = new Date(a.created_at).getTime();
      const dateB = new Date(b.created_at).getTime();
      return sortOrder === 'newest' ? dateB - dateA : dateA - dateB;
    });

  if (!selectedArchive) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400 dark:text-gray-500">
        <Clock className="w-12 h-12 mb-3 opacity-30" />
        <p className="text-sm">请先选择一个文件</p>
        <p className="text-xs mt-1">在存档列表中选择文件查看时间轴</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <Clock className="w-5 h-5 text-primary-500" />
          <h2 className="font-semibold text-lg dark:text-white">时间轴</h2>
          <span className="text-xs text-gray-400 dark:text-gray-500 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded-full">
            {filteredTimeline.length} 个版本
          </span>
        </div>

        <div className="flex items-center gap-2">
          {/* Sort Button */}
          <button
            onClick={() => setSortOrder(prev => prev === 'newest' ? 'oldest' : 'newest')}
            className="flex items-center gap-1 px-2 py-1.5 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
          >
            {sortOrder === 'newest' ? '最新优先' : '最早优先'}
          </button>

          {/* Filter Button */}
          {allTags.length > 0 && (
            <button
              onClick={() => setShowFilters(!showFilters)}
              className={`flex items-center gap-1 px-2 py-1.5 text-xs rounded transition ${
                filterTag ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-400' : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
              }`}
            >
              <Filter className="w-3.5 h-3.5" />
              {filterTag || '筛选'}
            </button>
          )}
        </div>
      </div>

      {/* File Info */}
      <div className="px-4 py-3 bg-gray-50 dark:bg-gray-800/50 border-b border-gray-200 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <FileText className="w-4 h-4 text-gray-400 dark:text-gray-500" />
          <div className="min-w-0">
            <p className="text-sm font-medium truncate dark:text-gray-200">{selectedArchive.file_name}</p>
            <p className="text-xs text-gray-500 dark:text-gray-400 truncate">{selectedArchive.file_path}</p>
          </div>
        </div>
      </div>

      {/* Filter Tags */}
      {showFilters && allTags.length > 0 && (
        <div className="px-4 py-2 border-b border-gray-200 dark:border-gray-700 flex flex-wrap gap-1">
          <button
            onClick={() => setFilterTag(null)}
            className={`px-2 py-1 text-xs rounded-full transition ${
              !filterTag ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-400' : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
            }`}
          >
            全部
          </button>
          {allTags.map(tag => (
            <button
              key={tag}
              onClick={() => setFilterTag(filterTag === tag ? null : tag)}
              className={`px-2 py-1 text-xs rounded-full transition ${
                filterTag === tag ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-700 dark:text-primary-400' : 'bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-gray-200 dark:hover:bg-gray-600'
              }`}
            >
              {tag}
            </button>
          ))}
        </div>
      )}

      {/* Timeline */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="relative pl-6">
          {/* Vertical line */}
          <div className="absolute left-2 top-0 bottom-0 w-0.5 bg-gray-200 dark:bg-gray-700" />

          {filteredTimeline.map((archive, index) => {
            const isCompareSelected = selectedForCompare === archive.id;
            
            return (
              <div key={archive.id} className="relative mb-6 animate-slide-in" style={{ animationDelay: `${index * 50}ms` }}>
                {/* Dot */}
                <div className={`absolute -left-4 top-1.5 w-3 h-3 rounded-full border-2 transition-colors
                  ${index === 0 ? 'bg-primary-500 border-primary-300' : 'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600'}
                  ${isCompareSelected ? 'bg-blue-500 border-blue-300' : ''}
                `} />

                <div className={`p-3 rounded-lg border transition-all
                  ${isCompareSelected
                    ? 'border-blue-300 dark:border-blue-600 bg-blue-50 dark:bg-blue-900/20'
                    : index === 0
                    ? 'border-primary-300 dark:border-primary-700 bg-primary-50 dark:bg-primary-900/20'
                    : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:border-gray-300 dark:hover:border-gray-600'
                  }`}
                >
                  {/* Header */}
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-gray-500 dark:text-gray-400">{formatSmartTime(archive.created_at)}</span>
                      {index === 0 && (
                        <span className="px-1.5 py-0.5 text-xs bg-primary-100 dark:bg-primary-900/40 text-primary-700 dark:text-primary-400 rounded">
                          最新
                        </span>
                      )}
                    </div>
                    <div className="flex gap-1">
                      <button
                        onClick={() => handleCompare(archive.id)}
                        className={`p-1 rounded transition ${
                          isCompareSelected ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400' : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-400'
                        }`}
                        title={isCompareSelected ? '取消选择' : '选择对比'}
                      >
                        <GitCompare className="w-3.5 h-3.5" />
                      </button>
                      <button
                        onClick={() => setConfirmAction({ type: 'restore', id: archive.id })}
                        className="p-1 hover:bg-gray-100 dark:hover:bg-gray-700 rounded text-gray-400"
                        title="恢复此版本"
                      >
                        <RotateCcw className="w-3.5 h-3.5" />
                      </button>
                      <button
                        onClick={() => setConfirmAction({ type: 'delete', id: archive.id })}
                        className="p-1 hover:bg-red-50 dark:hover:bg-red-900/20 rounded text-gray-400"
                        title="删除"
                      >
                        <Trash2 className="w-3.5 h-3.5" />
                      </button>
                    </div>
                  </div>

                  {/* Meta */}
                  <div className="mt-1 text-xs text-gray-500 dark:text-gray-400">
                    {formatFileSize(archive.file_size)} · {archive.chunk_count} 块
                    {archive.checksum && (
                      <span className="ml-2 text-gray-400 dark:text-gray-500">#{filteredTimeline.length - index}</span>
                    )}
                  </div>

                  {/* Note */}
                  {archive.note && (
                    <p className="mt-1 text-xs text-gray-600 dark:text-gray-400">{archive.note}</p>
                  )}

                  {/* Tags */}
                  {archive.tags?.length > 0 && (
                    <div className="mt-1 flex flex-wrap gap-1">
                      {archive.tags?.map((tag) => <TagBadge key={tag} tag={tag} />)}
                    </div>
                  )}
                </div>
              </div>
            );
          })}

          {filteredTimeline.length === 0 && (
            <div className="text-center text-gray-400 dark:text-gray-500 py-8">
              <p className="text-sm">暂无历史记录</p>
              {filterTag && (
                <button
                  onClick={() => setFilterTag(null)}
                  className="mt-2 text-xs text-primary-500 hover:underline"
                >
                  清除筛选
                </button>
              )}
            </div>
          )}
        </div>
      </div>
      <ConfirmDialog
        open={confirmAction !== null}
        title={confirmAction?.type === 'restore' ? '确认恢复' : '确认删除'}
        message={confirmAction?.type === 'restore' ? '确定要恢复此版本吗？' : '确定要删除此存档吗？删除后无法恢复。'}
        onConfirm={handleConfirm}
        onCancel={() => setConfirmAction(null)}
      />
    </div>
  );
}
