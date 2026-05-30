import { useArchiveStore } from "../stores/archiveStore";
import type { Archive } from "../types";

export function useArchive() {
  const store = useArchiveStore();

  const filteredArchives = store.archives;

  const archiveByFile = (filePath: string): Archive[] => {
    return store.archives.filter((a) => a.file_path === filePath);
  };

  const totalSize = store.archives.reduce((sum, a) => sum + a.file_size, 0);

  const uniqueFiles = new Set(store.archives.map((a) => a.file_path)).size;

  return {
    ...store,
    filteredArchives,
    archiveByFile,
    totalSize,
    uniqueFiles,
  };
}
