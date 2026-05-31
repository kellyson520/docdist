import { useState, useEffect, useRef, memo } from 'react';
import type { Archive } from '../../types';
import { formatFileSize } from '../../utils/format';
import { formatSmartTime } from '../../utils/time';
import { TagBadge } from '../common/TagBadge';
import {
  RotateCcw, Trash2, GitCompare, FileText, MoreVertical, Edit2,
  Clock, HardDrive, Hash, CheckSquare, Square,
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
  onToggleSelect?: () => void;
}

function ArchiveCardInner({
  archive,
  isSelected,
  isMultiSelected = false,
  onSelect,
  onRestore,
  onDelete,
  onCompare,
  onToggleSelect,
}: ArchiveCardProps) {
  const [showMenu, setShowMenu] = useState(false);
  const [showEditDialog, setShowEditDialog] = useState(false);
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

  // 文件类型图标颜色
  const getFileColor = (name: string) => {
    const ext = name.split('.').pop()?.toLowerCase() || '';
    const colors: Record<string, string> = {
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
    };
    return colors[ext] || 'text-gray-400 bg-gray-50 dark:bg-gray-700';
  };

  const fileColor = getFileColor(archive.file_name);

  return (
    <div
      onClick={onSelect}
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
            <h3 className="text-sm font-medium text-gray-800 dark:text-gray-200 truncate">
              {archive.file_name}
            </h3>
            <p className="text-xs text-gray-400 dark:text-gray-500 mt-0.5 truncate" title={archive.file_path}>
              {archive.file_path}
            </p>
          </div>

          {/* Actions */}
          <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onClick={(e) => { e.stopPropagation(); setShowEditDialog(true); }}
              className="p-1.5 hover:bg-amber-50 dark:hover:bg-amber-900/20 rounded-lg transition"
              title="编辑"
            >
              <Edit2 className="w-3.5 h-3.5 text-amber-500" />
            </button>
            <button
              onClick={(e) => { e.stopPropagation(); onRestore(); }}
              className="p-1.5 hover:bg-green-50 dark:hover:bg-green-900/20 rounded-lg transition"
              title="恢复"
            >
              <RotateCcw className="w-3.5 h-3.5 text-green-500" />
            </button>
            <button
              onClick={(e) => { e.stopPropagation(); onCompare(); }}
              className="p-1.5 hover:bg-blue-50 dark:hover:bg-blue-900/20 rounded-lg transition"
              title="对比"
            >
              <GitCompare className="w-3.5 h-3.5 text-blue-500" />
            </button>
            <div className="relative" ref={menuRef}>
              <button
                onClick={(e) => { e.stopPropagation(); setShowMenu(!showMenu); }}
                className="p-1.5 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
              >
                <MoreVertical className="w-3.5 h-3.5 text-gray-400" />
              </button>
              {showMenu && (
                <div className="absolute right-0 top-full mt-1 w-36 bg-white dark:bg-gray-800 border border-gray-200 dark:border-gray-700 rounded-lg shadow-lg py-1 z-10 animate-scale-in">
                  <button
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
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-2 ml-13 line-clamp-2">
            {archive.note}
          </p>
        )}

        {/* Tags */}
        {archive.tags.length > 0 && (
          <div className="flex flex-wrap gap-1 mt-2 ml-13">
            {archive.tags.map((tag) => (
              <TagBadge key={tag} tag={tag} />
            ))}
          </div>
        )}

        {/* Meta info */}
        <div className="flex items-center gap-4 mt-3 ml-13 text-[11px] text-gray-400 dark:text-gray-500">
          <span className="flex items-center gap-1">
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
          {archive.parent_id && (
            <span className="flex items-center gap-1 text-primary-400 dark:text-primary-500">
              <GitCompare className="w-3 h-3" />
              迭代版本
            </span>
          )}
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

/** 用 React.memo 包裹，避免列表中不必要的重渲染 */
export const ArchiveCard = memo(ArchiveCardInner, (prev, next) => {
  return (
    prev.archive === next.archive &&
    prev.isSelected === next.isSelected &&
    prev.isMultiSelected === next.isMultiSelected &&
    prev.onSelect === next.onSelect &&
    prev.onRestore === next.onRestore &&
    prev.onDelete === next.onDelete &&
    prev.onCompare === next.onCompare &&
    prev.onToggleSelect === next.onToggleSelect
  );
});
