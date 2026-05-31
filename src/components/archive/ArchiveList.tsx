import { useEffect, useState } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import { ArchiveCard } from './ArchiveCard';
import { CreateArchiveDialog } from './CreateArchiveDialog';
import { SearchBar } from '../common/SearchBar';
import { ConfirmDialog } from '../common/ConfirmDialog';
import {
  Plus, FolderOpen, Trash2, CheckSquare, XSquare,
  ChevronLeft, ChevronRight,
} from 'lucide-react';
import { open } from '@tauri-apps/api/dialog';
import { toast } from '../../stores/toastStore';

export function ArchiveList() {
  const {
    archives, selectedArchive, loading, searchQuery,
    selectedIds, page, hasMore,
    fetchArchives, fetchArchivesPaginated, createArchive, restoreArchive, deleteArchive,
    deleteArchivesBatch, compareArchives, selectArchive, setSearchQuery,
    toggleSelect, selectAll, clearSelection,
  } = useArchiveStore();

  const [showCreate, setShowCreate] = useState(false);
  const [deleteTarget, setDeleteTarget] = useState<string | null>(null);
  const [createPath, setCreatePath] = useState('');

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
      setCreatePath(selected as string);
      setShowCreate(true);
    }
  };

  const handleBatchDelete = async () => {
    const ids = Array.from(selectedIds);
    if (ids.length > 0) {
      await deleteArchivesBatch(ids);
    }
  };

  const hasSelection = selectedIds.size > 0;

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <FolderOpen className="w-5 h-5 text-primary-500" />
          <h2 className="font-semibold text-lg dark:text-white">存档管理</h2>
          <span className="text-xs text-gray-400 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded-full">
            {archives.length}
          </span>
        </div>
        <button
          onClick={handleCreate}
          className="flex items-center gap-1.5 px-3 py-1.5 bg-primary-500 text-white rounded-lg text-sm hover:bg-primary-600 active:bg-primary-700 transition shadow-sm"
        >
          <Plus className="w-4 h-4" />
          新建存档
        </button>
      </div>

      {/* Batch toolbar */}
      {hasSelection && (
        <div className="px-4 py-2 bg-primary-50 dark:bg-primary-900/20 border-b border-primary-100 dark:border-primary-800 flex items-center gap-3 animate-fade-in">
          <span className="text-xs text-primary-600 dark:text-primary-400 font-medium">
            已选 {selectedIds.size} 项
          </span>
          <button
            onClick={selectAll}
            className="flex items-center gap-1 text-xs text-primary-500 hover:text-primary-600 transition"
          >
            <CheckSquare className="w-3.5 h-3.5" />
            全选
          </button>
          <button
            onClick={clearSelection}
            className="flex items-center gap-1 text-xs text-gray-500 hover:text-gray-600 transition"
          >
            <XSquare className="w-3.5 h-3.5" />
            取消
          </button>
          <div className="flex-1" />
          <button
            onClick={handleBatchDelete}
            className="flex items-center gap-1 px-2 py-1 text-xs text-red-600 bg-red-50 dark:bg-red-900/20 rounded-lg hover:bg-red-100 transition"
          >
            <Trash2 className="w-3.5 h-3.5" />
            批量删除
          </button>
        </div>
      )}

      {/* Search */}
      <div className="px-4 py-3">
        <SearchBar value={searchQuery} onChange={setSearchQuery} />
      </div>

      {/* List */}
      <div className="flex-1 overflow-y-auto px-4 pb-4 space-y-2">
        {loading && archives.length === 0 ? (
          <div className="space-y-2">
            {[1, 2, 3].map(i => (
              <div key={i} className="h-24 bg-gray-100 dark:bg-gray-800 rounded-xl animate-pulse-slow" />
            ))}
          </div>
        ) : archives.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-gray-400">
            <div className="w-16 h-16 rounded-full bg-gray-100 dark:bg-gray-800 flex items-center justify-center mb-4">
              <FolderOpen className="w-8 h-8 opacity-30" />
            </div>
            <p className="text-sm font-medium">暂无存档</p>
            <p className="text-xs mt-1 text-gray-300">点击「新建存档」开始追踪文件历史</p>
          </div>
        ) : (
          archives.map((archive) => (
            <ArchiveCard
              key={archive.id}
              archive={archive}
              isSelected={selectedArchive?.id === archive.id}
              isMultiSelected={selectedIds.has(archive.id)}
              onSelect={() => selectArchive(archive)}
              onRestore={() => restoreArchive(archive.id)}
              onDelete={() => setDeleteTarget(archive.id)}
              onCompare={() => {
                if (selectedArchive && selectedArchive.id !== archive.id) {
                  if (selectedArchive.file_path === archive.file_path) {
                    compareArchives(selectedArchive.id, archive.id);
                  } else {
                    toast.warning('只能对比同一文件的不同版本');
                  }
                }
              }}
              onToggleSelect={() => toggleSelect(archive.id)}
            />
          ))
        )}
      </div>

      {/* Pagination */}
      {(page > 1 || hasMore) && (
        <div className="px-4 py-3 border-t border-gray-100 dark:border-gray-700 flex items-center justify-between">
          <button
            disabled={page <= 1}
            onClick={() => fetchArchivesPaginated(page - 1, undefined, searchQuery)}
            className="flex items-center gap-1 px-3 py-1.5 text-xs text-gray-500 hover:bg-gray-100 rounded-lg transition disabled:opacity-30"
          >
            <ChevronLeft className="w-3.5 h-3.5" />
            上一页
          </button>
          <span className="text-xs text-gray-400">第 {page} 页</span>
          <button
            disabled={!hasMore}
            onClick={() => fetchArchivesPaginated(page + 1, undefined, searchQuery)}
            className="flex items-center gap-1 px-3 py-1.5 text-xs text-gray-500 hover:bg-gray-100 rounded-lg transition disabled:opacity-30"
          >
            下一页
            <ChevronRight className="w-3.5 h-3.5" />
          </button>
        </div>
      )}

      {/* Create Dialog */}
      {showCreate && (
        <CreateArchiveDialog
          defaultPath={createPath}
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
