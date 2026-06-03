import { useEffect, useState, useCallback, useMemo } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import { shallow } from 'zustand/shallow';
import { formatFileSize } from '../../utils/format';
import { formatSmartTime } from '../../utils/time';
import { TagBadge } from '../common/TagBadge';
import {
  RotateCcw,
  Trash2,
  FileText,
  GitCompare,
  Star,
  Download,
  GitBranch,
} from 'lucide-react';
import { ConfirmDialog } from '../common/ConfirmDialog';
import { StarDialog } from './StarDialog';
type SortOrder = 'newest' | 'oldest';

export function VersionTreeView() {
  const {
    fileHistory,
    starredArchives,
    selectedArchive,
    fetchFileHistory,
    fetchStarredArchives,
    restoreArchive,
    deleteArchive,
    compareArchives,
    setView,
    starArchive,
    unstarArchive,
    exportHistory,
  } = useArchiveStore(
    (s) => ({
      fileHistory: s.fileHistory,
      starredArchives: s.starredArchives,
      selectedArchive: s.selectedArchive,
      fetchFileHistory: s.fetchFileHistory,
      fetchStarredArchives: s.fetchStarredArchives,
      restoreArchive: s.restoreArchive,
      deleteArchive: s.deleteArchive,
      compareArchives: s.compareArchives,
      setView: s.setView,
      starArchive: s.starArchive,
      unstarArchive: s.unstarArchive,
      exportHistory: s.exportHistory,
    }),
    shallow,
  );

  const [sortOrder, setSortOrder] = useState<SortOrder>('newest');
  const [selectedForCompare, setSelectedForCompare] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<{ type: 'restore' | 'delete'; id: string } | null>(null);
  const [starDialogOpen, setStarDialogOpen] = useState(false);
  const [starTarget, setStarTarget] = useState<string | null>(null);
  const [showStarredOnly, setShowStarredOnly] = useState(false);

  const filePath = selectedArchive?.file_path;
  useEffect(() => {
    if (filePath) {
      fetchFileHistory(filePath);
    }
  }, [filePath, fetchFileHistory]);

  useEffect(() => {
    fetchStarredArchives();
  }, [fetchStarredArchives]);

  // Build a map of starred archive IDs → star info
  const starredMap = useMemo(() => {
    const map = new Map<string, { star_id: string; label: string }>();
    for (const sa of starredArchives) {
      map.set(sa.archive.id, { star_id: sa.star_id, label: sa.label });
    }
    return map;
  }, [starredArchives]);

  const handleCompare = useCallback(
    (archiveId: string) => {
      if (selectedForCompare && selectedForCompare !== archiveId) {
        compareArchives(selectedForCompare, archiveId);
        setSelectedForCompare(null);
        setView('diff');
      } else {
        setSelectedForCompare(archiveId);
      }
    },
    [selectedForCompare, compareArchives, setView],
  );

  const handleConfirm = useCallback(() => {
    if (!confirmAction) return;
    if (confirmAction.type === 'restore') {
      restoreArchive(confirmAction.id);
    } else {
      deleteArchive(confirmAction.id);
    }
    setConfirmAction(null);
  }, [confirmAction, restoreArchive, deleteArchive]);

  const handleStarConfirm = useCallback(
    (label: string) => {
      if (starTarget) {
        starArchive(starTarget, label);
      }
      setStarDialogOpen(false);
      setStarTarget(null);
    },
    [starTarget, starArchive],
  );

  const handleUnstar = useCallback(
    (archiveId: string) => {
      const info = starredMap.get(archiveId);
      if (info) {
        unstarArchive(info.star_id);
      }
    },
    [starredMap, unstarArchive],
  );

  const handleExport = useCallback(() => {
    if (selectedArchive) {
      exportHistory(selectedArchive.file_path, '');
    }
  }, [selectedArchive, exportHistory]);

  // Sort and optionally filter
  const sortedHistory = useMemo(() => {
    let list = fileHistory.slice();
    if (showStarredOnly) {
      list = list.filter((a) => starredMap.has(a.id));
    }
    list.sort((a, b) => {
      const dateA = new Date(a.created_at).getTime();
      const dateB = new Date(b.created_at).getTime();
      return sortOrder === 'newest' ? dateB - dateA : dateA - dateB;
    });
    return list;
  }, [fileHistory, sortOrder, showStarredOnly, starredMap]);

  if (!selectedArchive) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400 dark:text-gray-500">
        <GitBranch className="w-12 h-12 mb-3 opacity-30" />
        <p className="text-sm">请先选择一个文件</p>
        <p className="text-xs mt-1">在存档列表中选择文件查看版本历史</p>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <GitBranch className="w-5 h-5 text-primary-500" />
          <h2 className="font-semibold text-lg dark:text-white">版本历史</h2>
          <span className="text-xs text-gray-400 dark:text-gray-500 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded-full">
            {sortedHistory.length} 个版本
          </span>
        </div>

        <div className="flex items-center gap-2">
          {/* Starred filter */}
          <button
            onClick={() => setShowStarredOnly(!showStarredOnly)}
            className={`flex items-center gap-1 px-2 py-1.5 text-xs rounded transition ${
              showStarredOnly
                ? 'bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400'
                : 'text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700'
            }`}
          >
            <Star className="w-3.5 h-3.5" />
            {showStarredOnly ? '仅标记' : '全部'}
          </button>

          {/* Sort */}
          <button
            onClick={() => setSortOrder((prev) => (prev === 'newest' ? 'oldest' : 'newest'))}
            className="flex items-center gap-1 px-2 py-1.5 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
          >
            {sortOrder === 'newest' ? '最新优先' : '最早优先'}
          </button>

          {/* Export */}
          <button
            onClick={handleExport}
            className="flex items-center gap-1 px-2 py-1.5 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
            title="导出历史"
          >
            <Download className="w-3.5 h-3.5" />
          </button>
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

      {/* Timeline */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="relative pl-6">
          {/* Vertical line */}
          <div className="absolute left-2 top-0 bottom-0 w-0.5 bg-gray-200 dark:bg-gray-700" />

          {sortedHistory.map((archive, index) => {
            const isCompareSelected = selectedForCompare === archive.id;
            const starInfo = starredMap.get(archive.id);
            const isStarred = !!starInfo;

            return (
              <div
                key={archive.id}
                className="relative mb-6 animate-slide-in"
                style={{ animationDelay: `${index * 50}ms` }}
              >
                {/* Dot */}
                <div
                  className={`absolute -left-4 top-1.5 w-3 h-3 rounded-full border-2 transition-colors ${
                    isStarred
                      ? 'bg-yellow-400 border-yellow-300'
                      : index === 0
                      ? 'bg-primary-500 border-primary-300'
                      : 'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600'
                  } ${isCompareSelected ? 'bg-blue-500 border-blue-300' : ''}`}
                />

                <div
                  className={`p-3 rounded-lg border transition-all ${
                    isCompareSelected
                      ? 'border-blue-300 dark:border-blue-600 bg-blue-50 dark:bg-blue-900/20'
                      : isStarred
                      ? 'border-yellow-300 dark:border-yellow-700 bg-yellow-50 dark:bg-yellow-900/10'
                      : index === 0
                      ? 'border-primary-300 dark:border-primary-700 bg-primary-50 dark:bg-primary-900/20'
                      : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:border-gray-300 dark:hover:border-gray-600'
                  }`}
                >
                  {/* Header */}
                  <div className="flex items-center justify-between">
                    <div className="flex items-center gap-2">
                      <span className="text-xs text-gray-500 dark:text-gray-400">
                        {formatSmartTime(archive.created_at)}
                      </span>
                      {index === 0 && (
                        <span className="px-1.5 py-0.5 text-xs bg-primary-100 dark:bg-primary-900/40 text-primary-700 dark:text-primary-400 rounded">
                          最新
                        </span>
                      )}
                      {isStarred && (
                        <span className="flex items-center gap-1 px-1.5 py-0.5 text-xs bg-yellow-100 dark:bg-yellow-900/30 text-yellow-700 dark:text-yellow-400 rounded">
                          <Star className="w-3 h-3 fill-yellow-500" />
                          {starInfo.label}
                        </span>
                      )}
                    </div>
                    <div className="flex gap-1">
                      <button
                        onClick={() => handleCompare(archive.id)}
                        className={`p-1 rounded transition ${
                          isCompareSelected
                            ? 'bg-blue-100 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400'
                            : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-400'
                        }`}
                        title={isCompareSelected ? '取消选择' : '选择对比'}
                      >
                        <GitCompare className="w-3.5 h-3.5" />
                      </button>
                      <button
                        onClick={() =>
                          isStarred ? handleUnstar(archive.id) : (setStarTarget(archive.id), setStarDialogOpen(true))
                        }
                        className={`p-1 rounded transition ${
                          isStarred
                            ? 'text-yellow-500 hover:bg-yellow-50 dark:hover:bg-yellow-900/20'
                            : 'hover:bg-gray-100 dark:hover:bg-gray-700 text-gray-400'
                        }`}
                        title={isStarred ? '取消标记' : '标记重要版本'}
                      >
                        <Star className={`w-3.5 h-3.5 ${isStarred ? 'fill-yellow-500' : ''}`} />
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
                      <span className="ml-2 text-gray-400 dark:text-gray-500">
                        #{sortedHistory.length - index}
                      </span>
                    )}
                  </div>

                  {/* Note */}
                  {archive.note && (
                    <p className="mt-1 text-xs text-gray-600 dark:text-gray-400">{archive.note}</p>
                  )}

                  {/* Tags */}
                  {archive.tags?.length > 0 && (
                    <div className="mt-1 flex flex-wrap gap-1">
                      {archive.tags?.map((tag) => (
                        <TagBadge key={tag} tag={tag} />
                      ))}
                    </div>
                  )}
                </div>
              </div>
            );
          })}

          {sortedHistory.length === 0 && (
            <div className="text-center text-gray-400 dark:text-gray-500 py-8">
              <GitBranch className="w-8 h-8 mx-auto mb-2 opacity-30" />
              <p className="text-sm">{showStarredOnly ? '暂无标记版本' : '暂无历史记录'}</p>
              {showStarredOnly && (
                <button
                  onClick={() => setShowStarredOnly(false)}
                  className="mt-2 text-xs text-primary-500 hover:underline"
                >
                  查看全部版本
                </button>
              )}
            </div>
          )}
        </div>
      </div>

      <ConfirmDialog
        open={confirmAction !== null}
        title={confirmAction?.type === 'restore' ? '确认恢复' : '确认删除'}
        message={
          confirmAction?.type === 'restore'
            ? '确定要恢复此版本吗？'
            : '确定要删除此存档吗？删除后无法恢复。'
        }
        onConfirm={handleConfirm}
        onCancel={() => setConfirmAction(null)}
      />

      <StarDialog
        open={starDialogOpen}
        onConfirm={handleStarConfirm}
        onCancel={() => {
          setStarDialogOpen(false);
          setStarTarget(null);
        }}
      />
    </div>
  );
}
