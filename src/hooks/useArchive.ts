import { useCallback, useMemo } from 'react';
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

  const archiveByFile = useCallback(
    (filePath: string): Archive[] => {
      return archives.filter((a) => a.file_path === filePath);
    },
    [archives],
  );

  const totalSize = useMemo(
    () => archives.reduce((sum, a) => sum + a.file_size, 0),
    [archives],
  );

  const uniqueFiles = useMemo(
    () => new Set(archives.map((a) => a.file_path)).size,
    [archives],
  );

  const selectedArchives = useMemo(
    () => archives.filter((a) => selectedIds.has(a.id)),
    [archives, selectedIds],
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

