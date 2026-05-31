import { useArchiveStore } from '../stores/archiveStore';
import { shallow } from 'zustand/shallow';
import type { Archive } from '../types';

export function useArchive() {
  const {
    archives,
    selectedIds,
    fetchArchives,
    createArchive,
    restoreArchive,
    deleteArchive,
    selectArchive,
    setSearchQuery,
    searchQuery,
    loading,
    error,
    selectedArchive,
  } = useArchiveStore(
    (s) => ({
      archives: s.archives,
      selectedIds: s.selectedIds,
      fetchArchives: s.fetchArchives,
      createArchive: s.createArchive,
      restoreArchive: s.restoreArchive,
      deleteArchive: s.deleteArchive,
      selectArchive: s.selectArchive,
      setSearchQuery: s.setSearchQuery,
      searchQuery: s.searchQuery,
      loading: s.loading,
      error: s.error,
      selectedArchive: s.selectedArchive,
    }),
    shallow,
  );

  const filteredArchives = archives;

  const archiveByFile = (filePath: string): Archive[] => {
    return archives.filter((a) => a.file_path === filePath);
  };

  const totalSize = archives.reduce((sum, a) => sum + a.file_size, 0);

  const uniqueFiles = new Set(archives.map((a) => a.file_path)).size;

  const selectedArchives = archives.filter((a) =>
    selectedIds.has(a.id)
  );

  return {
    archives,
    selectedIds,
    fetchArchives,
    createArchive,
    restoreArchive,
    deleteArchive,
    selectArchive,
    setSearchQuery,
    searchQuery,
    loading,
    error,
    selectedArchive,
    filteredArchives,
    archiveByFile,
    totalSize,
    uniqueFiles,
    selectedArchives,
  };
}
