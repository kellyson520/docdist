import { useState, useEffect, useRef, memo } from 'react';
import type { Archive } from '../../types';
import { formatFileSize } from '../../utils/format';
import { formatSmartTime } from '../../utils/time';
import {
  RotateCcw, Trash2, GitCompare, FileText, MoreVertical, Edit2,
  Clock, HardDrive, Hash, CheckSquare, Square, ChevronDown, ChevronUp, Tag, ExternalLink,
} from 'lucide-react';
import { EditArchiveDialog } from './EditArchiveDialog';
import { useArchiveStore } from '../../stores/archiveStore';

interface ArchiveCardProps {
  archive: Archive;
  isSelected: boolean;
  isMultiSelected?: boolean;
  onSelect: () => void;
  onRestore: () => void;
  onDelete: () => void;
  onCompare: () => void;
  onExternalCompare?: () => void;
  onToggleSelect?: () => void;
  versionIndex?: number;
  totalVersions?: number;
}

/** 文件类型图标颜色映射（模块级常量） */
const FILE_COLOR_MAP: Record<string, string> = {
  rs: 'text-orange-500 bg-orange-50 dark:bg-orange-900/20',
  ts: 'text-blue-500 bg-blue-50 dark:bg-blue-900/20',
  tsx: 'text-cyan-500 bg-cyan-50 dark:bg-cyan-900/20',
  js: 'text-yellow-500 bg-yellow-50 dark:bg-yellow-900/20',
  jsx: 'text-sky-500 bg-sky-50 dark:bg-sky-900/20',
  py: 'text-green-500 bg-green-50 dark:bg-green-900/20',
  md: 'text-gray-500 bg-gray-50 dark:bg-gray-700',
  json: 'text-amber-500 bg-amber-50 dark:bg-amber-900/20',
  html: 'text-red-500 bg-red-50 dark:bg-red-900/20',
  css: 'text-purple-500 bg-purple-50 dark:bg-purple-900/20',
  go: 'text-teal-500 bg-teal-50 dark:bg-teal-900/20',
  txt: 'text-gray-400 bg-gray-50 dark:bg-gray-700',
  yaml: 'text-pink-500 bg-pink-50 dark:bg-pink-900/20',
  yml: 'text-pink-500 bg-pink-50 dark:bg-pink-900/20',
  toml: 'text-violet-500 bg-violet-50 dark:bg-violet-900/20',
  sh: 'text-emerald-500 bg-emerald-50 dark:bg-emerald-900/20',
  sql: 'text-indigo-500 bg-indigo-50 dark:bg-indigo-900/20',
  csv: 'text-lime-500 bg-lime-50 dark:bg-lime-900/20',
  vue: 'text-emerald-600 bg-emerald-50 dark:bg-emerald-900/20',
  svelte: 'text-orange-600 bg-orange-50 dark:bg-orange-900/20',
};

/** 标签颜色映射（基于标签名哈希） */
const TAG_COLORS = [
  'bg-blue-100 text-blue-700 dark:bg-blue-900/30 dark:text-blue-400',
  'bg-green-100 text-green-700 dark:bg-green-900/30 dark:text-green-400',
  'bg-purple-100 text-purple-700 dark:bg-purple-900/30 dark:text-purple-400',
  'bg-amber-100 text-amber-700 dark:bg-amber-900/30 dark:text-amber-400',
  'bg-pink-100 text-pink-700 dark:bg-pink-900/30 dark:text-pink-400',
  'bg-cyan-100 text-cyan-700 dark:bg-cyan-900/30 dark:text-cyan-400',
  'bg-red-100 text-red-700 dark:bg-red-900/30 dark:text-red-400',
  'bg-indigo-100 text-indigo-700 dark:bg-indigo-900/30 dark:text-indigo-400',
];

function getTagColor(tag: string): string {
  let hash = 0;
  for (let i = 0; i < tag.length; i++) {
    hash = ((hash << 5) - hash) + tag.charCodeAt(i);
    hash |= 0;
  }
  return TAG_COLORS[Math.abs(hash) % TAG_COLORS.length];
}

function getFileColor(name: string): string {
  const ext = name.split('.').pop()?.toLowerCase() || '';
  return FILE_COLOR_MAP[ext] || 'text-gray-400 bg-gray-50 dark:bg-gray-700';
}

function ArchiveCardInner({
  archive,
  isSelected,
  isMultiSelected = false,
  onSelect,
  onRestore,
  onDelete,
  onCompare,
  onExternalCompare,
  onToggleSelect,
  versionIndex,
  totalVersions,
}: ArchiveCardProps) {
  const [showMenu, setShowMenu] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
  const [noteExpanded, setNoteExpanded] = useState(false);
  const updateArchive = useArchiveStore((s) => s.updateArchive);
  const menuRef = useRef<HTMLDivElement>(null);

  // 点击外部关闭下拉菜单
  useEffect(() => {
    if (!showMenu) return;
    const handleMouseDown = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setShowMenu(false);
      }
    };
    document.addEventListener('mousedown', handleMouseDown);
    return () => document.removeEventListener('mousedown', handleMouseDown);
  }, [showMenu]);

  const fileColor = getFileColor(archive.file_name);
  const hasLongNote = archive.note && archive.note.length > 80;
  const versionLabel = versionIndex != null && totalVersions != null
    ? `#${totalVersions - versionIndex}`
    : null;

  return (
    <div
      onClick={onSelect}
      onKeyDown={(e) => { if (e.key === 'Enter' || e.key === ' ') { e.preventDefault(); onSelect(); } }}
      role="button"
      tabIndex={0}
      aria-label={`存档: ${archive.file_name}`}
      aria-pressed={isSelected}
      className={`group relative bg-white dark:bg-gray-800 rounded-xl border transition-all duration-150 cursor-pointer card-hover ${
        isSelected
          ? 'border-primary-300 dark:border-primary-600 ring-2 ring-primary-100 dark:ring-primary-900/30 shadow-sm'
          : 'border-gray-100 dark:border-gray-700 hover:border-gray-200 dark:hover:border-gray-600'
      }`}
    >
      <div className="p-4">
        {/* Top row: icon + name + actions */}
        <div className="flex items-start gap-3">
          {/* Checkbox for multi-select */}
          {onToggleSelect && (
            <button
              onClick={(e) => { e.stopPropagation(); onToggleSelect(); }}
              aria-label={isMultiSelected ? '取消选择' : '选择'}
              aria-pressed={isMultiSelected}
              className="mt-0.5 flex-shrink-0"
            >
              {isMultiSelected ? (
                <CheckSquare className="w-4 h-4 text-primary-500" />
              ) : (
                <Square className="w-4 h-4 text-gray-300 dark:text-gray-600 group-hover:text-gray-400 dark:group-hover:text-gray-500 transition" />
              )}
            </button>
          )}

          {/* File icon */}
          <div className={`w-10 h-10 rounded-lg flex items-center justify-center flex-shrink-0 ${fileColor}`}>
            <FileText className="w-5 h-5" />
          </div>

          {/* Info */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-medium text-gray-800 dark:text-gray-200 truncate">
                {archive.file_name}
              </h3>
              {versionLabel && (
                <span className="flex-shrink-0 px-1.5 py-0.5 text-[10px] font-medium bg-gray-100 dark:bg-gray-700 text-gray-500 dark:text-gray-400 rounded">
                  {versionLabel}
                </span>
              )}
              {archive.parent_id && (
                <span className="flex-shrink-0 px-1.5 py-0.5 text-[10px] font-medium bg-primary-50 dark:bg-primary-900/20 text-primary-600 dark:text-primary-400 rounded">
                  迭代
                </span>
              )}
            </div>
            <p className="text-xs text-gray-400 dark:text-gray-500 mt-0.5 truncate" title={archive.file_path}>
              {archive.file_path}
            </p>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-1 sm:opacity-0 sm:group-hover:opacity-100 transition-opacity">
            <button
              onClick={(e) => { e.stopPropagation(); setShowEditDialog(true); }}
              aria-label="编辑"
              className="p-1.5 hover:bg-amber-50 dark:hover:bg-amber-900/20 rounded-lg transition"
              title="编辑备注和标签"
            >
              <Edit2 className="w-3.5 h-3.5 text-amber-500" />
            </button>
            <button
              onClick={(e) => { e.stopPropagation(); onRestore(); }}
              aria-label="恢复"
              className="p-1.5 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition"
              title="恢复此版本"
            >
              <RotateCcw className="w-3.5 h-3.5 text-green-500" />
            </button>
            <button
              onClick={(e) => { e.stopPropagation(); onCompare(); }}
              aria-label="对比"
              className="p-1.5 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg transition"
              title="选择对比"
            >
              <GitCompare className="w-3.5 h-3.5 text-blue-500" />
            </button>
            {onExternalCompare && (
              <button
                onClick={(e) => { e.stopPropagation(); onExternalCompare(); }}
                aria-label="外部对比"
                className="p-1.5 hover:bg-cyan-50 dark:hover:bg-cyan-900/20 rounded-lg transition"
                title="使用外部工具对比"
              >
                <ExternalLink className="w-3.5 h-3.5 text-cyan-500" />
              </button>
            )}
            <div className="relative" ref={menuRef}>
              <button
                onClick={(e) => { e.stopPropagation(); setShowMenu(!showMenu); }}
                aria-label="更多操作"
                aria-haspopup="menu"
                aria-expanded={showMenu}
                className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
              >
                <MoreVertical className="w-3.5 h-3.5 text-gray-400" />
              </button>
              {showMenu && (
                <div role="menu" className="absolute right-0 top-full mt-1 w-36 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg py-1 z-10 animate-scale-in">
                  <button
                    role="menuitem"
                    onClick={(e) => { e.stopPropagation(); onDelete(); setShowMenu(false); }}
                    className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-red-600 dark:text-red-400 hover:bg-red-50 dark:hover:bg-red-900/20 transition"
                  >
                    <Trash2 className="w-3.5 h-3.5" />
                    删除
                  </button>
                </div>
              )}
            </div>
          </div>
        </div>

        {/* Note */}
        {archive.note && (
          <div className="mt-2 ml-13">
            <p className={`text-xs text-gray-500 dark:text-gray-400 ${!noteExpanded && hasLongNote ? 'line-clamp-2' : ''}`}>
              {archive.note}
            </p>
            {hasLongNote && (
              <button
                onClick={(e) => { e.stopPropagation(); setNoteExpanded(!noteExpanded); }}
                className="flex items-center gap-0.5 mt-1 text-[11px] text-primary-500 hover:text-primary-600 dark:text-primary-400 transition"
              >
                {noteExpanded ? (
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
          <div className="flex flex-wrap gap-1 mt-2 ml-13">
            {archive.tags?.map((tag) => (
              <span
                key={tag}
                className={`inline-flex items-center gap-1 px-2 py-0.5 text-[11px] font-medium rounded-full ${getTagColor(tag)}`}
              >
                <Tag className="w-2.5 h-2.5" />
                {tag}
              </span>
            ))}
          </div>
        )}

        {/* Meta info */}
        <div className="flex items-center gap-3 mt-3 ml-13 text-[11px] text-gray-400 dark:text-gray-500">
          <span className="flex items-center gap-1" title={archive.created_at}>
            <Clock className="w-3 h-3" />
            {formatSmartTime(archive.created_at)}
          </span>
          <span className="flex items-center gap-1">
            <HardDrive className="w-3 h-3" />
            {formatFileSize(archive.file_size)}
          </span>
          <span className="flex items-center gap-1">
            <Hash className="w-3 h-3" />
            {archive.chunk_count} 块
          </span>
        </div>
      </div>

      {/* Selected indicator */}
      {isSelected && (
        <div className="absolute left-0 top-3 bottom-3 w-0.5 bg-primary-500 rounded-r" />
      )}
      {showEditDialog && (
        <EditArchiveDialog
          initialNote={archive.note || ''}
          initialTags={archive.tags}
          onConfirm={async (note, tags) => {
            await updateArchive(archive.id, note, tags);
            setShowEditDialog(false);
          }}
          onCancel={() => setShowEditDialog(false)}
        />
      )}
    </div>
  );
}

/**
 * 用 React.memo 包裹，避免列表中不必要的重渲染。
 * 仅比较数据 props：callback 引用由父组件在 map 中创建，每次渲染都是新引用，
 * 比较它们毫无意义且导致 memo 完全失效。
 */
export const ArchiveCard = memo(ArchiveCardInner, (prev, next) => {
  return (
    prev.archive === next.archive &&
    prev.isSelected === next.isSelected &&
    prev.isMultiSelected === next.isMultiSelected &&
    prev.versionIndex === next.versionIndex
  );
});
