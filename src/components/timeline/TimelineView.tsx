import { useEffect, useState, useCallback, useMemo } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import { shallow } from 'zustand/shallow';
import { formatFileSize } from '../../utils/format';
import { formatSmartTime } from '../../utils/time';
import {
  Clock, RotateCcw, Trash2, FileText, GitCompare, Filter,
  Search, X, ChevronDown, ChevronUp, Tag, GitBranch,
} from 'lucide-react';
import { ConfirmDialog } from '../common/ConfirmDialog';

type SortOrder = 'newest' | 'oldest';
type FilterTag = string | null;

/** 标签颜色映射 */
const TAG_COLORS = [
  'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
  'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
  'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400',
  'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400',
  'bg-pink-100 text-pink-700 dark:bg-pink-900/30 dark:text-pink-400',
  'bg-cyan-100 text-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-400',
];

function getTagColor(tag: string): string {
  let hash = 0;
  for (let i = 0; i < tag.length; i++) {
    hash = ((hash << 5) - hash) + tag.charCodeAt(i);
    hash |= 0;
  }
  return TAG_COLORS[Math.abs(hash) % TAG_COLORS.length];
}

/** 计算两个时间戳之间的时间间隔描述 */
function getTimeGap(prev: string, curr: string): string {
  const diff = new Date(prev).getTime() - new Date(curr).getTime();
  const mins = Math.floor(diff / 60000);
  if (mins < 1) return '不到1分钟';
  if (mins < 60) return `${mins}分钟`;
  const hours = Math.floor(mins / 60);
  if (hours < 24) return `${hours}小时`;
  const days = Math.floor(hours / 24);
  if (days < 30) return `${days}天`;
  const months = Math.floor(days / 30);
  return `${months}个月`;
}

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
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedForCompare, setSelectedForCompare] = useState<string | null>(null);
  const [confirmAction, setConfirmAction] = useState<{ type: 'restore' | 'delete'; id: string } | null>(null);
  const [expandedNotes, setExpandedNotes] = useState<Set<string>>(new Set());

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

  const toggleNote = useCallback((id: string) => {
    setExpandedNotes(prev => {
      const next = new Set(prev);
      if (next.has(id)) next.delete(id); else next.add(id);
      return next;
    });
  }, []);

  // Get all unique tags from timeline
  const allTags = useMemo(
    () => Array.from(new Set(timeline.flatMap(a => a.tags ?? []))),
    [timeline]
  );

  // Filter and sort timeline
  const filteredTimeline = useMemo(() => {
    return timeline
      .slice()
      .filter(a => {
        if (filterTag && !(a.tags ?? []).includes(filterTag)) return false;
        if (searchQuery) {
          const q = searchQuery.toLowerCase();
          return (
            a.file_name.toLowerCase().includes(q) ||
            a.note?.toLowerCase().includes(q) ||
            a.tags?.some(t => t.toLowerCase().includes(q))
          );
        }
        return true;
      })
      .sort((a, b) => {
        const dateA = new Date(a.created_at).getTime();
        const dateB = new Date(b.created_at).getTime();
        return sortOrder === 'newest' ? dateB - dateA : dateA - dateB;
      });
  }, [timeline, filterTag, sortOrder, searchQuery]);

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
          <div className="min-w-0 flex-1">
            <p className="text-sm font-medium truncate dark:text-gray-200">{selectedArchive.file_name}</p>
            <p className="text-xs text-gray-500 dark:text-gray-400 truncate">{selectedArchive.file_path}</p>
          </div>
          <div className="flex items-center gap-1 text-xs text-gray-400 dark:text-gray-500">
            <GitBranch className="w-3.5 h-3.5" />
            <span>{timeline.length} 个版本</span>
          </div>
        </div>
      </div>

      {/* Search Bar */}
      <div className="px-4 py-2 border-b border-gray-200 dark:border-gray-700">
        <div className="relative">
          <Search className="absolute left-2.5 top-1/2 -translate-y-1/2 w-3.5 h-3.5 text-gray-400" />
          <input
            type="text"
            placeholder="搜索版本备注或标签..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="w-full pl-8 pr-8 py-1.5 text-xs bg-gray-50 dark:bg-gray-700 border border-gray-200 dark:border-gray-600 rounded-lg focus:outline-none focus:ring-1 focus:ring-primary-400 dark:text-gray-200 placeholder-gray-400"
          />
          {searchQuery && (
            <button
              onClick={() => setSearchQuery('')}
              className="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600 dark:hover:text-gray-300"
            >
              <X className="w-3.5 h-3.5" />
            </button>
          )}
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

      {/* Compare Mode Indicator */}
      {selectedForCompare && (
        <div className="px-4 py-2 bg-blue-50 dark:bg-blue-900/20 border-b border-blue-200 dark:border-blue-800 flex items-center justify-between">
          <div className="flex items-center gap-2 text-xs text-blue-700 dark:text-blue-400">
            <GitCompare className="w-3.5 h-3.5" />
            <span>已选择一个版本，请点击另一个版本进行对比</span>
          </div>
          <button
            onClick={() => setSelectedForCompare(null)}
            className="text-xs text-blue-500 hover:text-blue-700 dark:hover:text-blue-300"
          >
            取消
          </button>
        </div>
      )}

      {/* Timeline */}
      <div className="flex-1 overflow-y-auto p-4">
        <div className="relative pl-8">
          {/* Vertical line */}
          <div className="absolute left-3 top-0 bottom-0 w-0.5 bg-gradient-to-b from-primary-300 via-gray-200 to-gray-100 dark:from-primary-700 dark:via-gray-700 dark:to-gray-800" />

          {filteredTimeline.map((archive, index) => {
            const isCompareSelected = selectedForCompare === archive.id;
            const isFirst = index === 0 && sortOrder === 'newest';
            const isLast = index === filteredTimeline.length - 1 && sortOrder === 'oldest';
            const isLatest = isFirst || isLast;
            const versionNum = sortOrder === 'newest'
              ? timeline.length - timeline.findIndex(a => a.id === archive.id)
              : timeline.findIndex(a => a.id === archive.id) + 1;
            const prevArchive = index > 0 ? filteredTimeline[index - 1] : null;
            const timeGap = prevArchive ? getTimeGap(prevArchive.created_at, archive.created_at) : null;
            const isNoteExpanded = expandedNotes.has(archive.id);

            return (
              <div key={archive.id}>
                {/* Time gap indicator */}
                {timeGap && index > 0 && (
                  <div className="relative flex items-center gap-2 mb-3 ml-1">
                    <div className="absolute left-[-20px] top-1/2 w-3 h-px bg-gray-200 dark:bg-gray-700" />
                    <span className="text-[10px] text-gray-400 dark:text-gray-600 italic">
                      间隔 {timeGap}
                    </span>
                  </div>
                )}

                <div className="relative mb-4 animate-slide-in" style={{ animationDelay: `${index * 50}ms` }}>
                  {/* Dot */}
                  <div className={`absolute -left-5 top-3 w-3.5 h-3.5 rounded-full border-2 transition-all shadow-sm
                    ${isLatest ? 'bg-primary-500 border-primary-300 dark:border-primary-400 scale-110' : 'bg-white dark:bg-gray-800 border-gray-300 dark:border-gray-600'}
                    ${isCompareSelected ? 'bg-blue-500 border-blue-300 dark:border-blue-400 scale-110' : ''}
                  `}>
                    {isLatest && (
                      <div className="absolute inset-0 rounded-full bg-primary-400 dark:bg-primary-500 animate-ping opacity-20" />
                    )}
                  </div>

                  <div className={`p-3 rounded-lg border transition-all
                    ${isCompareSelected
                      ? 'border-blue-300 dark:border-blue-600 bg-blue-50 dark:bg-blue-900/20 shadow-sm'
                      : isLatest
                      ? 'border-primary-200 dark:border-primary-700 bg-primary-50/50 dark:bg-primary-900/10'
                      : 'border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 hover:border-gray-300 dark:hover:border-gray-600 hover:shadow-sm'
                    }`}
                  >
                    {/* Header */}
                    <div className="flex items-center justify-between">
                      <div className="flex items-center gap-2">
                        <span className="px-1.5 py-0.5 text-[10px] font-medium bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded">
                          #{versionNum}
                        </span>
                        <span className="text-xs text-gray-500 dark:text-gray-400">
                          {formatSmartTime(archive.created_at)}
                        </span>
                        {isLatest && (
                          <span className="px-1.5 py-0.5 text-[10px] font-medium bg-primary-100 dark:bg-primary-900/40 text-primary-700 dark:text-primary-400 rounded">
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
                          className="p-1 hover:bg-green-50 dark:hover:bg-green-900/20 rounded text-gray-400 hover:text-green-500 transition"
                          title="恢复此版本"
                        >
                          <RotateCcw className="w-3.5 h-3.5" />
                        </button>
                        <button
                          onClick={() => setConfirmAction({ type: 'delete', id: archive.id })}
                          className="p-1 hover:bg-red-50 dark:hover:bg-red-900/20 rounded text-gray-400 hover:text-red-500 transition"
                          title="删除"
                        >
                          <Trash2 className="w-3.5 h-3.5" />
                        </button>
                      </div>
                    </div>

                    {/* Meta */}
                    <div className="mt-1.5 flex items-center gap-3 text-[11px] text-gray-500 dark:text-gray-400">
                      <span>{formatFileSize(archive.file_size)}</span>
                      <span>·</span>
                      <span>{archive.chunk_count} 块</span>
                    </div>

                    {/* Note */}
                    {archive.note && (
                      <div className="mt-1.5">
                        <p className={`text-xs text-gray-600 dark:text-gray-400 ${!isNoteExpanded ? 'line-clamp-2' : ''}`}>
                          {archive.note}
                        </p>
                        {archive.note.length > 100 && (
                          <button
                            onClick={() => toggleNote(archive.id)}
                            className="flex items-center gap-0.5 mt-1 text-[10px] text-primary-500 hover:text-primary-600 transition"
                          >
                            {isNoteExpanded ? (
                              <><ChevronUp className="w-3 h-3" /> 收起</>
                            ) : (
                              <><ChevronDown className="w-3 h-3" /> 展开</>
                            )}
                          </button>
                        )}
                      </div>
                    )}

                    {/* Tags */}
                    {archive.tags?.length > 0 && (
                      <div className="mt-1.5 flex flex-wrap gap-1">
                        {archive.tags?.map((tag) => (
                          <span
                            key={tag}
                            className={`inline-flex items-center gap-1 px-1.5 py-0.5 text-[10px] font-medium rounded-full ${getTagColor(tag)}`}
                          >
                            <Tag className="w-2 h-2" />
                            {tag}
                          </span>
                        ))}
                      </div>
                    )}
                  </div>
                </div>
              </div>
            );
          })}

          {filteredTimeline.length === 0 && (
            <div className="text-center text-gray-400 dark:text-gray-500 py-8">
              <Clock className="w-8 h-8 mx-auto mb-2 opacity-30" />
              <p className="text-sm">
                {searchQuery ? '没有匹配的版本' : '暂无历史记录'}
              </p>
              {(filterTag || searchQuery) && (
                <button
                  onClick={() => { setFilterTag(null); setSearchQuery(''); }}
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
