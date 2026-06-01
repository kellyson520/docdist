import { useState } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import { shallow } from 'zustand/shallow';
import { formatFileSize, formatDate } from '../../utils/format';
import { Archive, RotateCcw, Maximize2, Plus, Clock, ChevronDown } from 'lucide-react';
import { open } from '@tauri-apps/api/dialog';

export function MiniMode({ onExpand }: { onExpand: () => void }) {
  const { archives, createArchive, restoreArchive, fetchArchives } = useArchiveStore(
    (s) => ({
      archives: s.archives,
      createArchive: s.createArchive,
      restoreArchive: s.restoreArchive,
      fetchArchives: s.fetchArchives,
    }),
    shallow,
  );
  const [showRecent, setShowRecent] = useState(false);

  const recentArchives = archives.slice(0, 5);

  const handleQuickArchive = async () => {
    const selected = await open({ multiple: false, title: '快速存档' });
    if (selected) {
      await createArchive(selected as string);
    }
  };

  return (
    <div className="w-80 bg-white dark:bg-gray-800 rounded-2xl shadow-2xl border border-gray-200 dark:border-gray-700 overflow-hidden animate-fade-in">
      {/* Header */}
      <div className="flex items-center justify-between px-4 py-3 bg-gradient-to-r from-primary-500 to-primary-600 text-white">
        <div className="flex items-center gap-2">
          <Archive className="w-4 h-4" />
          <span className="text-sm font-medium">追光 Mini</span>
        </div>
        <button
          onClick={onExpand}
          aria-label="展开完整模式"
          className="p-1 hover:bg-white/20 rounded transition"
          title="展开完整模式"
        >
          <Maximize2 className="w-4 h-4" />
        </button>
      </div>

      {/* Quick Actions */}
      <div className="p-3 border-b border-gray-100 dark:border-gray-700">
        <button
          onClick={handleQuickArchive}
          className="w-full flex items-center justify-center gap-2 px-4 py-2.5 bg-primary-50 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 rounded-lg hover:bg-primary-100 dark:hover:bg-primary-900/50 transition text-sm font-medium"
        >
          <Plus className="w-4 h-4" />
          快速存档
        </button>
      </div>

      {/* Recent Archives */}
      <div className="p-3">
        <button
          onClick={() => { setShowRecent(!showRecent); if (!showRecent) fetchArchives(); }}
          aria-expanded={showRecent}
          className="flex items-center justify-between w-full text-sm text-gray-600 dark:text-gray-300 hover:text-gray-800 dark:hover:text-gray-100"
        >
          <span className="flex items-center gap-1.5">
            <Clock className="w-4 h-4" />
            最近存档
          </span>
          <ChevronDown className={`w-4 h-4 transition-transform ${showRecent ? 'rotate-180' : ''}`} />
        </button>

        {showRecent && (
          <div className="mt-2 space-y-2">
            {recentArchives.length === 0 ? (
              <p className="text-xs text-gray-400 dark:text-gray-500 text-center py-2">暂无存档</p>
            ) : (
              recentArchives.map((archive) => (
                <div
                  key={archive.id}
                  className="flex items-center justify-between p-2 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-700 group"
                >
                  <div className="min-w-0 flex-1">
                    <p className="text-xs font-medium truncate text-gray-800 dark:text-gray-200">{archive.file_name}</p>
                    <p className="text-xs text-gray-400 dark:text-gray-500">{formatDate(archive.created_at)} · {formatFileSize(archive.file_size)}</p>
                  </div>
                  <button
                    onClick={() => restoreArchive(archive.id)}
                    className="p-2.5 sm:p-1 sm:opacity-0 sm:group-hover:opacity-100 hover:bg-gray-200 dark:hover:bg-gray-600 rounded transition min-w-[44px] min-h-[44px] flex items-center justify-center"
                    aria-label={`恢复 ${archive.file_name}`}
                    title="恢复"
                  >
                    <RotateCcw className="w-3 h-3 text-gray-500 dark:text-gray-400" />
                  </button>
                </div>
              ))
            )}
          </div>
        )}
      </div>
    </div>
  );
}
