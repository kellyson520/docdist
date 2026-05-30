import { create } from 'zustand';
import { invoke } from '@tauri-apps/api/tauri';
import type { Archive, DiffResult, Statistics } from '../types';

interface ArchiveState {
  archives: Archive[];
  selectedArchive: Archive | null;
  compareTarget: Archive | null;
  diffResult: DiffResult | null;
  timeline: Archive[];
  statistics: Statistics | null;
  loading: boolean;
  error: string | null;
  view: 'list' | 'timeline' | 'graph' | 'mini';
  searchQuery: string;

  fetchArchives: (filePath?: string, search?: string) => Promise<void>;
  createArchive: (path: string, note?: string, tags?: string[]) => Promise<void>;
  restoreArchive: (id: string, targetPath?: string) => Promise<void>;
  deleteArchive: (id: string) => Promise<void>;
  updateArchive: (id: string, note: string, tags: string[]) => Promise<void>;
  compareArchives: (id1: string, id2: string) => Promise<void>;
  fetchTimeline: (filePath: string) => Promise<void>;
  fetchStatistics: () => Promise<void>;
  selectArchive: (archive: Archive | null) => void;
  setCompareTarget: (archive: Archive | null) => void;
  setView: (view: ArchiveState['view']) => void;
  setSearchQuery: (query: string) => void;
  clearDiff: () => void;
}

export const useArchiveStore = create<ArchiveState>((set, get) => ({
  archives: [],
  selectedArchive: null,
  compareTarget: null,
  diffResult: null,
  timeline: [],
  statistics: null,
  loading: false,
  error: null,
  view: 'list',
  searchQuery: '',

  fetchArchives: async (filePath?: string, search?: string) => {
    set({ loading: true, error: null });
    try {
      const archives = await invoke<Archive[]>('list_archives', {
        filePath: filePath || null,
        search: search || null,
      });
      set({ archives, loading: false });
    } catch (e: any) {
      set({ error: e.toString(), loading: false });
    }
  },

  createArchive: async (path: string, note?: string, tags?: string[]) => {
    set({ loading: true, error: null });
    try {
      await invoke('create_archive', {
        path,
        note: note || '',
        tags: tags || [],
        parentId: null,
      });
      await get().fetchArchives();
    } catch (e: any) {
      set({ error: e.toString(), loading: false });
    }
  },

  restoreArchive: async (id: string, targetPath?: string) => {
    try {
      await invoke('restore_archive', { id, targetPath: targetPath || null });
    } catch (e: any) {
      set({ error: e.toString() });
    }
  },

  deleteArchive: async (id: string) => {
    try {
      await invoke('delete_archive', { id });
      set((state) => ({
        archives: state.archives.filter((a) => a.id !== id),
        selectedArchive: state.selectedArchive?.id === id ? null : state.selectedArchive,
      }));
    } catch (e: any) {
      set({ error: e.toString() });
    }
  },

  updateArchive: async (id: string, note: string, tags: string[]) => {
    try {
      await invoke('update_archive', { id, note, tags });
      set((state) => ({
        archives: state.archives.map((a) =>
          a.id === id ? { ...a, note, tags } : a
        ),
      }));
    } catch (e: any) {
      set({ error: e.toString() });
    }
  },

  compareArchives: async (id1: string, id2: string) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<DiffResult>('compare_archives', { id1, id2 });
      set({ diffResult: result, loading: false });
    } catch (e: any) {
      set({ error: e.toString(), loading: false });
    }
  },

  fetchTimeline: async (filePath: string) => {
    try {
      const timeline = await invoke<Archive[]>('get_timeline', { path: filePath });
      set({ timeline });
    } catch (e: any) {
      set({ error: e.toString() });
    }
  },

  fetchStatistics: async () => {
    try {
      const statistics = await invoke<Statistics>('get_statistics');
      set({ statistics });
    } catch (e: any) {
      console.error(e);
    }
  },

  selectArchive: (archive) => set({ selectedArchive: archive }),
  setCompareTarget: (archive) => set({ compareTarget: archive }),
  setView: (view) => set({ view }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  clearDiff: () => set({ diffResult: null }),
}));
