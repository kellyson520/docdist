import { create } from 'zustand';
import { toast } from './toastStore';
import { createLogger } from '../utils/logger';
import { invoke } from '@tauri-apps/api/tauri';
import { listen } from '@tauri-apps/api/event';
import type { Archive, DiffResult, Statistics } from '../types';
import type { EnhancedDiffResult } from '../types/diff';

const log = createLogger('store');

// ==================== Types ====================

export interface FileChangeEvent {
  path: string;
  event_type: 'detected' | 'auto_archive_triggered' | 'auto_archive_pending';
  timestamp: string;
}

export interface WatcherStatus {
  running: boolean;
  paths: string[];
}

export interface AppConfig {
  watcher: {
    enabled: boolean;
    watch_dirs: string[];
    exclude_patterns: string[];
    auto_archive_delay: number;
    min_file_size: number;
    max_file_size: number;
  };
  storage: {
    chunk_size: number;
    incremental: boolean;
    deduplication: boolean;
    storage_path: string | null;
    max_versions: number;
    auto_cleanup_days: number;
  };
  log: {
    level: string;
    file_output: boolean;
    max_file_size_mb: number;
    retention_days: number;
  };
  language: string;
  theme: string;
  auto_start: boolean;
  minimize_to_tray: boolean;
}

export interface CleanupStats {
  removed_count: number;
  removed_bytes: number;
  kept_count: number;
}

// ==================== State ====================

interface ArchiveState {
  // 存档数据
  archives: Archive[];
  selectedArchive: Archive | null;
  compareTarget: Archive | null;
  diffResult: DiffResult | null;
  enhancedDiffResult: EnhancedDiffResult | null;
  timeline: Archive[];
  statistics: Statistics | null;
  loading: boolean;
  error: string | null;
  view: 'list' | 'timeline' | 'diff' | 'graph' | 'mini';
  searchQuery: string;

  // 分页
  page: number;
  pageSize: number;
  totalCount: number;
  hasMore: boolean;

  // 批量操作
  selectedIds: Set<string>;

  // Watcher
  watcherStatus: WatcherStatus;
  fileEvents: FileChangeEvent[];

  // Config
  config: AppConfig | null;

  // ==================== Actions ====================

  // 存档 CRUD
  fetchArchives: (filePath?: string, search?: string) => Promise<void>;
  fetchArchivesPaginated: (page?: number, filePath?: string, search?: string) => Promise<void>;
  createArchive: (path: string, note?: string, tags?: string[]) => Promise<void>;
  restoreArchive: (id: string, targetPath?: string) => Promise<void>;
  deleteArchive: (id: string) => Promise<void>;
  deleteArchivesBatch: (ids: string[]) => Promise<number>;
  updateArchive: (id: string, note: string, tags: string[]) => Promise<void>;
  compareArchives: (id1: string, id2: string) => Promise<void>;
  compareArchivesEnhanced: (id1: string, id2: string) => Promise<void>;
  clearEnhancedDiff: () => void;
  fetchTimeline: (filePath: string) => Promise<void>;
  fetchStatistics: () => Promise<void>;

  // 选择
  selectArchive: (archive: Archive | null) => void;
  setCompareTarget: (archive: Archive | null) => void;
  setView: (view: ArchiveState['view']) => void;
  setSearchQuery: (query: string) => void;
  clearDiff: () => void;

  // 批量操作
  toggleSelect: (id: string) => void;
  selectAll: () => void;
  clearSelection: () => void;

  // Watcher
  startWatcher: (paths: string[]) => Promise<void>;
  stopWatcher: () => Promise<void>;
  fetchWatcherStatus: () => Promise<void>;
  addWatcherPath: (path: string) => Promise<void>;
  removeWatcherPath: (path: string) => Promise<void>;
  setWatcherExcludePatterns: (patterns: string[]) => Promise<void>;

  // Config
  fetchConfig: () => Promise<void>;
  updateConfig: (config: AppConfig) => Promise<void>;

  // 存储管理
  cleanupOrphanChunks: () => Promise<CleanupStats>;
  verifyChunks: () => Promise<string[]>;

  // 事件监听
  setupEventListeners: () => () => void;
}

// ==================== Store ====================

export const useArchiveStore = create<ArchiveState>((set, get) => ({
  // Initial state
  archives: [],
  selectedArchive: null,
  compareTarget: null,
  diffResult: null,
  enhancedDiffResult: null,
  timeline: [],
  statistics: null,
  loading: false,
  error: null,
  view: 'list',
  searchQuery: '',
  page: 1,
  pageSize: 50,
  totalCount: 0,
  hasMore: false,
  selectedIds: new Set(),
  watcherStatus: { running: false, paths: [] },
  fileEvents: [],
  config: null,

  // ==================== 存档 CRUD ====================

  fetchArchives: async (filePath?: string, search?: string) => {
    set({ loading: true, error: null });
    try {
      const archives = await invoke<Archive[]>('list_archives', {
        filePath: filePath || null,
        search: search || null,
      });
      set({ archives, loading: false });
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  },

  fetchArchivesPaginated: async (page = 1, filePath?: string, search?: string) => {
    const { pageSize } = get();
    set({ loading: true, error: null });
    try {
      const [archives, total] = await invoke<[Archive[], number]>('list_archives_paginated', {
        filePath: filePath || null,
        search: search || null,
        page,
        pageSize,
      });
      set({
        archives,
        page,
        totalCount: total,
        hasMore: page * pageSize < total,
        loading: false,
      });
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
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
      log.info('存档创建成功', { path });
      toast.success('存档创建成功', path.split('/').pop());
      // Refresh list
      const { fetchArchives, searchQuery } = get();
      await fetchArchives(undefined, searchQuery || undefined);
    } catch (e: unknown) {
      const msg = e instanceof Error ? e.message : String(e);
      log.error('创建存档失败', msg);
      toast.error('创建失败', msg);
      set({ error: msg, loading: false });
    }
  },

  restoreArchive: async (id: string, targetPath?: string) => {
    set({ loading: true, error: null });
    try {
      await invoke('restore_archive', {
        id,
        targetPath: targetPath || null,
      });
      log.info('存档已恢复', { id });
      toast.success('恢复成功', '文件已恢复到指定位置');
      set({ loading: false });
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  },

  deleteArchive: async (id: string) => {
    set({ loading: true, error: null });
    try {
      await invoke('delete_archive', { id });
      log.info('存档已删除', { id });
      toast.success('已删除', '存档已成功删除');
      const { fetchArchives, searchQuery, selectedIds } = get();
      const newSelected = new Set(selectedIds);
      newSelected.delete(id);
      set({ selectedIds: newSelected });
      await fetchArchives(undefined, searchQuery || undefined);
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  },

  deleteArchivesBatch: async (ids: string[]) => {
    set({ loading: true, error: null });
    try {
      const count = await invoke<number>('delete_archives_batch', { ids });
      const { fetchArchives, searchQuery } = get();
      set({ selectedIds: new Set() });
      await fetchArchives(undefined, searchQuery || undefined);
      return count;
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
      return 0;
    }
  },

  updateArchive: async (id: string, note: string, tags: string[]) => {
    set({ loading: true, error: null });
    try {
      await invoke('update_archive', { id, note, tags });
      const { fetchArchives, searchQuery } = get();
      await fetchArchives(undefined, searchQuery || undefined);
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  },

  compareArchives: async (id1: string, id2: string) => {
    set({ loading: true, error: null, diffResult: null });
    try {
      log.info('开始对比', { id1, id2 });
      const diffResult = await invoke<DiffResult>('compare_archives', { id1, id2 });
      set({ diffResult, loading: false });
      toast.info('对比完成', `+${diffResult.stats.additions} -${diffResult.stats.deletions}`);
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  },

  compareArchivesEnhanced: async (id1: string, id2: string) => {
    set({ loading: true, error: null });
    try {
      const result = await invoke<EnhancedDiffResult>('compare_archives_enhanced', { id1, id2 });
      set({ enhancedDiffResult: result, loading: false });
    } catch (err) {
      set({ error: String(err), loading: false });
      throw err;
    }
  },

  clearEnhancedDiff: () => set({ enhancedDiffResult: null }),

  fetchTimeline: async (filePath: string) => {
    set({ loading: true, error: null });
    try {
      const timeline = await invoke<Archive[]>('get_timeline', { path: filePath });
      set({ timeline, loading: false });
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e), loading: false });
    }
  },

  fetchStatistics: async () => {
    try {
      const statistics = await invoke<Statistics>('get_statistics');
      set({ statistics });
    } catch (e: unknown) {
      console.error('Failed to fetch statistics:', e);
    }
  },

  // ==================== 选择 ====================

  selectArchive: (archive) => set({ selectedArchive: archive }),
  setCompareTarget: (archive) => set({ compareTarget: archive }),
  setView: (view) => set({ view }),
  setSearchQuery: (query) => set({ searchQuery: query }),
  clearDiff: () => set({ diffResult: null, compareTarget: null }),

  // ==================== 批量操作 ====================

  toggleSelect: (id: string) => {
    const { selectedIds } = get();
    const newSet = new Set(selectedIds);
    if (newSet.has(id)) {
      newSet.delete(id);
    } else {
      newSet.add(id);
    }
    set({ selectedIds: newSet });
  },

  selectAll: () => {
    const { archives } = get();
    set({ selectedIds: new Set(archives.map(a => a.id)) });
  },

  clearSelection: () => set({ selectedIds: new Set() }),

  // ==================== Watcher ====================

  startWatcher: async (paths: string[]) => {
    try {
      await invoke('start_watcher', { paths });
      await get().fetchWatcherStatus();
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },

  stopWatcher: async () => {
    try {
      await invoke('stop_watcher');
      await get().fetchWatcherStatus();
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },

  fetchWatcherStatus: async () => {
    try {
      const status = await invoke<WatcherStatus>('get_watcher_status');
      set({ watcherStatus: status });
    } catch (e: unknown) {
      console.error('Failed to fetch watcher status:', e);
    }
  },

  addWatcherPath: async (path: string) => {
    try {
      await invoke('add_watcher_path', { path });
      await get().fetchWatcherStatus();
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },

  removeWatcherPath: async (path: string) => {
    try {
      await invoke('remove_watcher_path', { path });
      await get().fetchWatcherStatus();
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },

  setWatcherExcludePatterns: async (patterns: string[]) => {
    try {
      await invoke('set_watcher_exclude_patterns', { patterns });
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },

  // ==================== Config ====================

  fetchConfig: async () => {
    try {
      const config = await invoke<AppConfig>('get_config');
      set({ config });
    } catch (e: unknown) {
      console.error('Failed to fetch config:', e);
    }
  },

  updateConfig: async (newConfig: AppConfig) => {
    try {
      await invoke('update_config', { newConfig });
      set({ config: newConfig });
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e) });
    }
  },

  // ==================== 存储管理 ====================

  cleanupOrphanChunks: async () => {
    try {
      const stats = await invoke<CleanupStats>('cleanup_orphan_chunks');
      return stats;
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e) });
      return { removed_count: 0, removed_bytes: 0, kept_count: 0 };
    }
  },

  verifyChunks: async () => {
    try {
      return await invoke<string[]>('verify_chunks');
    } catch (e: unknown) {
      set({ error: e instanceof Error ? e.message : String(e) });
      return [];
    }
  },

  // ==================== 事件监听 ====================

  setupEventListeners: () => {
    // 监听文件变化事件
    const unlistenFileChanged = listen<FileChangeEvent>('file-changed', (event) => {
      const { fileEvents } = get();
      const newEvents = [event.payload, ...fileEvents].slice(0, 100); // 保留最近100条
      set({ fileEvents: newEvents });
    });

    // 监听 watcher 状态变化
    const unlistenWatcherStatus = listen<WatcherStatus>('watcher-status', (event) => {
      set({ watcherStatus: event.payload });
    });

    // 监听自动存档请求
    const unlistenAutoArchive = listen<{ path: string }>('auto-archive-request', async (event) => {
      try {
        await invoke('create_archive', {
          path: event.payload.path,
          note: '自动存档',
          tags: ['auto'],
          parentId: null,
        });
        // 刷新列表
        const { fetchArchives, searchQuery } = get();
        await fetchArchives(undefined, searchQuery || undefined);
      } catch (e) {
        console.error('Auto-archive failed:', e);
      }
    });

    // 返回清理函数
    return () => {
      unlistenFileChanged.then(fn => fn());
      unlistenWatcherStatus.then(fn => fn());
      unlistenAutoArchive.then(fn => fn());
    };
  },
}));
