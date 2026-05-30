import type { Archive } from '../../types';
import { formatFileSize } from '../../utils/format';
import { formatSmartTime } from '../../utils/time';
import { TagBadge } from '../common/TagBadge';
import { RotateCcw, Trash2, GitCompare, FileText, MoreVertical, Clock, HardDrive, Hash } from 'lucide-react';
import { useState, useCallback } from 'react';

interface ArchiveCardProps {
  archive: Archive;
  onRestore: () => void;
  onDelete: () => void;
  onCompare: () => void;
  onSelect: () => void;
  isSelected: boolean;
}

export function ArchiveCard({ archive, onRestore, onDelete, onCompare, onSelect, isSelected }: ArchiveCardProps) {
  const [showMenu, setShowMenu] = useState(false);

  const handleMenuClick = useCallback((e: React.MouseEvent, action: () => void) => {
    e.stopPropagation();
    action();
    setShowMenu(false);
  }, []);

  const getFileIcon = () => {
    const ext = archive.file_name.split('.').pop()?.toLowerCase();
    const iconColors: Record<string, string> = {
      'txt': 'text-gray-500',
      'md': 'text-blue-500',
      'json': 'text-yellow-500',
      'js': 'text-yellow-400',
      'ts': 'text-blue-400',
      'py': 'text-green-500',
      'rs': 'text-orange-500',
      'go': 'text-cyan-500',
      'java': 'text-red-500',
      'cpp': 'text-purple-500',
      'c': 'text-gray-600',
    };
    return iconColors[ext || ''] || 'text-primary-500';
  };

  return (
    <div
      onClick={onSelect}
      className={`group relative p-4 rounded-xl border transition-all cursor-pointer animate-fade-in
        ${isSelected
          ? 'border-primary-400 bg-primary-50 shadow-sm'
          : 'border-gray-200 bg-white hover:border-gray-300 hover:shadow-sm'
        }`}
    >
      <div className="flex items-start justify-between">
        <div className="flex items-center gap-3 min-w-0">
          <div className={`w-10 h-10 rounded-lg bg-gray-100 flex items-center justify-center flex-shrink-0`}>
            <FileText className={`w-5 h-5 ${getFileIcon()}`} />
          </div>
          <div className="min-w-0">
            <h3 className="font-medium text-sm truncate">{archive.file_name}</h3>
            <p className="text-xs text-gray-500 truncate flex items-center gap-1">
              <HardDrive className="w-3 h-3" />
              {archive.file_path}
            </p>
          </div>
        </div>

        <div className="relative">
          <button
            onClick={(e) => { e.stopPropagation(); setShowMenu(!showMenu); }}
            className="p-1 opacity-0 group-hover:opacity-100 hover:bg-gray-100 rounded transition"
          >
            <MoreVertical className="w-4 h-4 text-gray-400" />
          </button>

          {showMenu && (
            <div className="absolute right-0 top-8 w-36 bg-white rounded-lg shadow-lg border py-1 z-10 animate-fade-in">
              <button
                onClick={(e) => handleMenuClick(e, onRestore)}
                className="w-full px-3 py-2 text-sm text-left hover:bg-gray-50 flex items-center gap-2"
              >
                <RotateCcw className="w-4 h-4" /> 恢复
              </button>
              <button
                onClick={(e) => handleMenuClick(e, onCompare)}
                className="w-full px-3 py-2 text-sm text-left hover:bg-gray-50 flex items-center gap-2"
              >
                <GitCompare className="w-4 h-4" /> 对比
              </button>
              <hr className="my-1" />
              <button
                onClick={(e) => handleMenuClick(e, onDelete)}
                className="w-full px-3 py-2 text-sm text-left hover:bg-red-50 text-red-600 flex items-center gap-2"
              >
                <Trash2 className="w-4 h-4" /> 删除
              </button>
            </div>
          )}
        </div>
      </div>

      {/* 元信息 */}
      <div className="mt-3 flex items-center gap-4 text-xs text-gray-500">
        <span className="flex items-center gap-1" title="文件大小">
          <HardDrive className="w-3 h-3" />
          {formatFileSize(archive.file_size)}
        </span>
        <span className="flex items-center gap-1" title="创建时间">
          <Clock className="w-3 h-3" />
          {formatSmartTime(archive.created_at)}
        </span>
        <span className="flex items-center gap-1" title="数据块数量">
          <Hash className="w-3 h-3" />
          {archive.chunk_count} 块
        </span>
      </div>

      {/* 备注 */}
      {archive.note && (
        <p className="mt-2 text-xs text-gray-600 bg-gray-50 rounded px-2 py-1.5 line-clamp-2">
          📝 {archive.note}
        </p>
      )}

      {/* 标签 */}
      {archive.tags.length > 0 && (
        <div className="mt-2 flex flex-wrap gap-1">
          {archive.tags.map((tag) => (
            <TagBadge key={tag} tag={tag} />
          ))}
        </div>
      )}

      {/* 选中指示器 */}
      {isSelected && (
        <div className="absolute left-0 top-1/2 -translate-y-1/2 w-1 h-8 bg-primary-500 rounded-r-full" />
      )}
    </div>
  );
}
