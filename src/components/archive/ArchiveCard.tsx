import type { Archive } from '../../types';
import { formatFileSize, formatDate } from '../../utils/format';
import { TagBadge } from '../common/TagBadge';
import { RotateCcw, Trash2, GitCompare, FileText, MoreVertical } from 'lucide-react';
import { useState } from 'react';

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
          <div className="w-10 h-10 rounded-lg bg-primary-100 flex items-center justify-center flex-shrink-0">
            <FileText className="w-5 h-5 text-primary-600" />
          </div>
          <div className="min-w-0">
            <h3 className="font-medium text-sm truncate">{archive.file_name}</h3>
            <p className="text-xs text-gray-500 truncate">{archive.file_path}</p>
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
                onClick={(e) => { e.stopPropagation(); onRestore(); setShowMenu(false); }}
                className="w-full px-3 py-2 text-sm text-left hover:bg-gray-50 flex items-center gap-2"
              >
                <RotateCcw className="w-4 h-4" /> 恢复
              </button>
              <button
                onClick={(e) => { e.stopPropagation(); onCompare(); setShowMenu(false); }}
                className="w-full px-3 py-2 text-sm text-left hover:bg-gray-50 flex items-center gap-2"
              >
                <GitCompare className="w-4 h-4" /> 对比
              </button>
              <hr className="my-1" />
              <button
                onClick={(e) => { e.stopPropagation(); onDelete(); setShowMenu(false); }}
                className="w-full px-3 py-2 text-sm text-left hover:bg-red-50 text-red-600 flex items-center gap-2"
              >
                <Trash2 className="w-4 h-4" /> 删除
              </button>
            </div>
          )}
        </div>
      </div>

      <div className="mt-3 flex items-center gap-3 text-xs text-gray-500">
        <span>{formatFileSize(archive.file_size)}</span>
        <span>·</span>
        <span>{formatDate(archive.created_at)}</span>
        <span>·</span>
        <span>{archive.chunk_count} 块</span>
      </div>

      {archive.note && (
        <p className="mt-2 text-xs text-gray-600 bg-gray-50 rounded px-2 py-1">{archive.note}</p>
      )}

      {archive.tags.length > 0 && (
        <div className="mt-2 flex flex-wrap gap-1">
          {archive.tags.map((tag) => (
            <TagBadge key={tag} tag={tag} />
          ))}
        </div>
      )}
    </div>
  );
}
