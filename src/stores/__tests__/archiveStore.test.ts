import { describe, it, expect, beforeEach } from 'vitest';
import { mockInvoke, mockListen } from '../../test/tauri-mocks';
import { useArchiveStore } from '../archiveStore';
import type { Archive } from '../../types';

function makeArchive(id: string, note = ''): Archive {
  return {
    id,
    file_path: `/tmp/${id}.txt`,
    file_name: `${id}.txt`,
    file_size: 100,
    checksum: `checksum-${id}`,
    chunk_count: 1,
    note,
    tags: [],
    parent_id: null,
    created_at: '2026-01-01 00:00:00.000',
  };
}

describe('archiveStore 版本管理 actions', () => {
  beforeEach(() => {
    useArchiveStore.setState({
      archives: [],
      selectedArchive: null,
      selectedIds: new Set(),
      compareTarget: null,
      diffResult: null,
      enhancedDiffResult: null,
      loading: false,
      error: null,
      view: 'list',
      searchQuery: '',
      starredArchives: [],
      fileHistory: [],
      config: null,
      statistics: null,
      timeline: [],
      watcherStatus: undefined,
      fileEvents: [],
    });
    mockInvoke.mockReset();
    mockListen.mockReset();
  });

  describe('starArchive', () => {
    it('should call invoke and refresh starred archives', async () => {
      mockInvoke.mockResolvedValueOnce(undefined); // star_archive
      mockInvoke.mockResolvedValueOnce([]); // get_starred_archives

      await useArchiveStore.getState().starArchive('test-id', 'test label');

      expect(mockInvoke).toHaveBeenCalledWith('star_archive', { archiveId: 'test-id', label: 'test label' });
      expect(mockInvoke).toHaveBeenCalledWith('get_starred_archives');
    });

    it('should set loading to false after completion', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      mockInvoke.mockResolvedValueOnce([]);

      await useArchiveStore.getState().starArchive('test-id', 'test label');

      expect(useArchiveStore.getState().loading).toBe(false);
    });

    it('should handle error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await useArchiveStore.getState().starArchive('test-id', 'test label');

      expect(useArchiveStore.getState().error).toBe('Database error');
      expect(useArchiveStore.getState().loading).toBe(false);
    });
  });

  describe('unstarArchive', () => {
    it('should call invoke and refresh starred archives', async () => {
      mockInvoke.mockResolvedValueOnce(undefined);
      mockInvoke.mockResolvedValueOnce([]);

      await useArchiveStore.getState().unstarArchive('test-id');

      expect(mockInvoke).toHaveBeenCalledWith('unstar_archive', { archiveId: 'test-id' });
      expect(mockInvoke).toHaveBeenCalledWith('get_starred_archives');
    });

    it('should handle error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await useArchiveStore.getState().unstarArchive('test-id');

      expect(useArchiveStore.getState().error).toBe('Database error');
      expect(useArchiveStore.getState().loading).toBe(false);
    });
  });

  describe('fetchStarredArchives', () => {
    it('should set starred archives', async () => {
      const mockData = [
        { archive: { id: '1', file_path: '/test.txt', file_name: 'test.txt', file_size: 100, checksum: 'abc', chunk_count: 1, note: '', tags: '[]', parent_id: null, created_at: '2025-01-01' }, star_id: 's1', label: 'test' }
      ];
      mockInvoke.mockResolvedValueOnce(mockData);

      await useArchiveStore.getState().fetchStarredArchives();

      expect(useArchiveStore.getState().starredArchives).toEqual(mockData);
    });

    it('should handle error and clear starred archives', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await useArchiveStore.getState().fetchStarredArchives();

      expect(useArchiveStore.getState().starredArchives).toEqual([]);
      expect(useArchiveStore.getState().error).toBe('Database error');
    });
  });

  describe('fetchFileHistory', () => {
    it('should set file history', async () => {
      const mockData = [
        { id: '1', file_path: '/test.txt', file_name: 'test.txt', file_size: 100, checksum: 'abc', chunk_count: 1, note: '', tags: '[]', parent_id: null, created_at: '2025-01-01' },
        { id: '2', file_path: '/test.txt', file_name: 'test.txt', file_size: 200, checksum: 'def', chunk_count: 2, note: '', tags: '[]', parent_id: null, created_at: '2025-01-02' }
      ];
      mockInvoke.mockResolvedValueOnce(mockData);

      await useArchiveStore.getState().fetchFileHistory('/test.txt');

      expect(useArchiveStore.getState().fileHistory).toEqual(mockData);
    });

    it('should handle error and clear file history', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      await useArchiveStore.getState().fetchFileHistory('/test.txt');

      expect(useArchiveStore.getState().fileHistory).toEqual([]);
      expect(useArchiveStore.getState().error).toBe('Database error');
    });
  });

  describe('exportHistory', () => {
    it('should return null for empty outputDir', async () => {
      const result = await useArchiveStore.getState().exportHistory('/test.txt', '');
      expect(result).toBeNull();
      expect(mockInvoke).not.toHaveBeenCalled();
    });

    it('should return result on success', async () => {
      const mockResult = { output_path: '/output/history.zip', archive_count: 5 };
      mockInvoke.mockResolvedValueOnce(mockResult);

      const result = await useArchiveStore.getState().exportHistory('/test.txt', '/output');

      expect(result).toEqual(mockResult);
      expect(mockInvoke).toHaveBeenCalledWith('export_history', { filePath: '/test.txt', outputPath: '/output' });
    });

    it('should return null on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Export failed'));

      const result = await useArchiveStore.getState().exportHistory('/test.txt', '/output');

      expect(result).toBeNull();
    });
  });

  describe('searchByPath', () => {
    it('should return results on success', async () => {
      const mockData = [
        { id: '1', file_path: '/test.txt', file_name: 'test.txt', file_size: 100, checksum: 'abc', chunk_count: 1, note: '', tags: '[]', parent_id: null, created_at: '2025-01-01' }
      ];
      mockInvoke.mockResolvedValueOnce(mockData);

      const result = await useArchiveStore.getState().searchByPath('/test');

      expect(result).toEqual(mockData);
      expect(mockInvoke).toHaveBeenCalledWith('search_archives_by_path', { pattern: '/test' });
    });

    it('should return empty array on error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Database error'));

      const result = await useArchiveStore.getState().searchByPath('/test');

      expect(result).toEqual([]);
    });
  });

  describe('setWatcherExcludePatterns', () => {
    it('should call invoke and refresh config', async () => {
      mockInvoke.mockResolvedValueOnce(undefined); // set_watcher_exclude_patterns
      mockInvoke.mockResolvedValueOnce({}); // fetch_config

      await useArchiveStore.getState().setWatcherExcludePatterns(['*.tmp', '.git']);

      expect(mockInvoke).toHaveBeenCalledWith('set_watcher_exclude_patterns', { patterns: ['*.tmp', '.git'] });
    });

    it('should handle error', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Update failed'));

      await useArchiveStore.getState().setWatcherExcludePatterns(['*.tmp']);

      expect(useArchiveStore.getState().error).toBe('Update failed');
    });
  });

  describe('archive refresh', () => {
    it('ignores stale archive list responses', async () => {
      let resolveOld!: (value: Archive[]) => void;
      let resolveNew!: (value: Archive[]) => void;
      const oldRequest = new Promise<Archive[]>(resolve => { resolveOld = resolve; });
      const newRequest = new Promise<Archive[]>(resolve => { resolveNew = resolve; });
      const oldArchive = makeArchive('old');
      const newArchive = makeArchive('new');

      mockInvoke.mockReturnValueOnce(oldRequest);
      mockInvoke.mockReturnValueOnce(newRequest);

      const oldFetch = useArchiveStore.getState().fetchArchives(undefined, 'old');
      const newFetch = useArchiveStore.getState().fetchArchives(undefined, 'new');

      resolveNew([newArchive]);
      await newFetch;
      expect(useArchiveStore.getState().archives).toEqual([newArchive]);

      resolveOld([oldArchive]);
      await oldFetch;
      expect(useArchiveStore.getState().archives).toEqual([newArchive]);
    });

    it('updates selected archive reference after refresh', async () => {
      const staleArchive = makeArchive('selected', '旧备注');
      const freshArchive = makeArchive('selected', '新备注');
      useArchiveStore.setState({ selectedArchive: staleArchive });
      mockInvoke.mockResolvedValueOnce([freshArchive]);

      await useArchiveStore.getState().fetchArchives();

      expect(useArchiveStore.getState().selectedArchive).toEqual(freshArchive);
    });
  });
});
