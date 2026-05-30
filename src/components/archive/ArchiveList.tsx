import { useEffect, useState } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import { ArchiveCard } from './ArchiveCard';
import { CreateArchiveDialog } from './CreateArchiveDialog';
import { SearchBar } from '../common/SearchBar';
import { ConfirmDialog } from '../common/ConfirmDialog';
import { Plus, FolderOpen } from 'lucide-react';
import { open } from '@tauri-apps/api/dialog';

export function ArchiveList() {
  const {
    archives, selectedArchive, loading, searchQuery,
    fetchArchives, createArchive, restoreArchive, deleteArchive,
    compareArchives, selectArchive, setSearchQuery,
  } = useArchiveStore();

  const [showCreate, setShowCreate] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);

  useEffect(() => {
    fetchArchives();
  }, [fetchArchives]);

  useEffect(() => {
    const timer = setTimeout(() => {
      fetchArchives(undefined, searchQuery);
    }, 300);
    return () => clearTimeout(timer);
  }, [searchQuery, fetchArchives]);

  const handleCreate = async () => {
    const selected = await open({
      multiple: false,
      title: '选择要存档的文件',
    });
    if (selected) {
      setShowCreate(true);
      // Store the path for the dialog
      (window as unknown as Record<string, string>).__selectedFilePath = selected as string;
    }
  };

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-100">
        <div className="flex items-center gap-2">
          <FolderOpen className="w-5 h-5 text-primary-500" />
          <h2 className="font-semibold text-lg">存档管理</h2>
          <span className="text-xs text-gray-400 bg-gray-100 px-2 py-0.5 rounded-full">
            {archives.length}
          </span>
        </div>
        <button
          onClick={handleCreate}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-500 text-white rounded-lg text-sm hover:bg-primary-600 transition"
        >
          <Plus className="w-4 h-4" />
          新建存档
        </button>
      </div>

      {/* Search */}
      <div className="px-4 py-3">
        <SearchBar value={searchQuery} onChange={setSearchQuery} />
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto px-4 pb-4 space-y-3">
        {loading && archives.length === 0 ? (
          <div className="flex items-center justify-center h-32 text-gray-400">
            <div className="animate-spin w-5 h-5 border-2 border-primary-400 border-t-transparent rounded-full mr-2" />
            加载中...
          </div>
        ) : archives.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-gray-400">
            <FolderOpen className="w-12 h-12 mb-3 opacity-30" />
            <p className="text-sm">暂无存档</p>
            <p className="text-xs mt-1">点击「新建存档」开始追踪文件历史</p>
          </div>
        ) : (
          archives.map((archive) => (
            <ArchiveCard
              key={archive.id}
              archive={archive}
              isSelected={selectedArchive?.id === archive.id}
              onSelect={() => selectArchive(archive)}
              onRestore={() => restoreArchive(archive.id)}
              onDelete={() => setDeleteTarget(archive.id)}
              onCompare={() => {
                if (selectedArchive && selectedArchive.id !== archive.id) {
                  compareArchives(selectedArchive.id, archive.id);
                }
              }}
            />
          ))
        )}
      </div>

      {/* Create Dialog */}
      {showCreate && (
        <CreateArchiveDialog
          defaultPath={(window as unknown as Record<string, string>).__selectedFilePath || ''}
          onConfirm={(path, note, tags) => {
            createArchive(path, note, tags);
            setShowCreate(false);
          }}
          onCancel={() => setShowCreate(false)}
        />
      )}

      {/* Delete Confirmation */}
      <ConfirmDialog
        open={deleteTarget !== null}
        title="删除存档"
        message="确定要删除这个存档吗？此操作不可撤销。"
        onConfirm={() => {
          if (deleteTarget) deleteArchive(deleteTarget);
          setDeleteTarget(null);
        }}
        onCancel={() => setDeleteTarget(null)}
      />
    </div>
  );
}
