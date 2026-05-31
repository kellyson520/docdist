/**
 * archiveStore 单元测试
 * 覆盖所有同步 actions 和异步 actions（invoke mock）
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mockInvoke } from '../../test/tauri-mocks'
import { useArchiveStore } from '../archiveStore'
import type { Archive, DiffResult } from '../../types'

// Mock toastStore
vi.mock('../toastStore', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}))

// Mock logger
vi.mock('../../utils/logger', () => ({
  createLogger: () => ({
    info: vi.fn(),
    warn: vi.fn(),
    error: vi.fn(),
    debug: vi.fn(),
  }),
}))

// ==================== Test Fixtures ====================

const mockArchive1: Archive = {
  id: 'archive-1',
  file_path: '/home/user/docs/report.pdf',
  file_name: 'report.pdf',
  file_size: 1024,
  checksum: 'abc123',
  chunk_count: 2,
  note: 'first version',
  tags: ['important'],
  parent_id: null,
  created_at: '2025-01-01T00:00:00Z',
}

const mockArchive2: Archive = {
  id: 'archive-2',
  file_path: '/home/user/docs/report.pdf',
  file_name: 'report.pdf',
  file_size: 2048,
  checksum: 'def456',
  chunk_count: 3,
  note: 'second version',
  tags: ['important', 'review'],
  parent_id: 'archive-1',
  created_at: '2025-01-02T00:00:00Z',
}

const mockArchive3: Archive = {
  id: 'archive-3',
  file_path: '/home/user/docs/notes.txt',
  file_name: 'notes.txt',
  file_size: 512,
  checksum: 'ghi789',
  chunk_count: 1,
  note: '',
  tags: [],
  parent_id: null,
  created_at: '2025-01-03T00:00:00Z',
}

const mockDiffResult: DiffResult = {
  hunks: [
    {
      old_start: 1,
      old_lines: 3,
      new_start: 1,
      new_lines: 4,
      changes: [
        { change_type: 'equal', content: 'line1', old_line: 1, new_line: 1 },
        { change_type: 'delete', content: 'line2', old_line: 2, new_line: null },
        { change_type: 'add', content: 'new_line2', old_line: null, new_line: 2 },
        { change_type: 'add', content: 'new_line3', old_line: null, new_line: 3 },
        { change_type: 'equal', content: 'line3', old_line: 3, new_line: 4 },
      ],
    },
  ],
  stats: {
    additions: 2,
    deletions: 1,
    unchanged: 2,
  },
}

// ==================== Tests ====================

describe('useArchiveStore', () => {
  beforeEach(() => {
    // Reset store to initial-like state before each test
    useArchiveStore.setState({
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
      page: 1,
      pageSize: 50,
      totalCount: 0,
      hasMore: false,
      selectedIds: new Set(),
      watcherStatus: { running: false, paths: [] },
      fileEvents: [],
      config: null,
    })
    mockInvoke.mockReset()
  })

  // ==================== 1. 初始状态默认值 ====================
  describe('initial state', () => {
    it('should have correct default values', () => {
      // Create a fresh store to check actual initial values
      const state = useArchiveStore.getState()
      expect(state.archives).toEqual([])
      expect(state.loading).toBe(false)
      expect(state.error).toBeNull()
      expect(state.selectedIds).toEqual(new Set())
      expect(state.view).toBe('list')
      expect(state.searchQuery).toBe('')
      expect(state.selectedArchive).toBeNull()
      expect(state.page).toBe(1)
      expect(state.hasMore).toBe(false)
      expect(state.diffResult).toBeNull()
      expect(state.compareTarget).toBeNull()
    })
  })

  // ==================== 2. selectArchive ====================
  describe('selectArchive', () => {
    it('should set selectedArchive', () => {
      useArchiveStore.getState().selectArchive(mockArchive1)
      expect(useArchiveStore.getState().selectedArchive).toEqual(mockArchive1)
    })

    it('should clear selectedArchive when null is passed', () => {
      useArchiveStore.setState({ selectedArchive: mockArchive1 })
      useArchiveStore.getState().selectArchive(null)
      expect(useArchiveStore.getState().selectedArchive).toBeNull()
    })
  })

  // ==================== 3. setView ====================
  describe('setView', () => {
    it('should switch to timeline view', () => {
      useArchiveStore.getState().setView('timeline')
      expect(useArchiveStore.getState().view).toBe('timeline')
    })

    it('should switch to diff view', () => {
      useArchiveStore.getState().setView('diff')
      expect(useArchiveStore.getState().view).toBe('diff')
    })

    it('should switch to graph view', () => {
      useArchiveStore.getState().setView('graph')
      expect(useArchiveStore.getState().view).toBe('graph')
    })

    it('should switch to mini view', () => {
      useArchiveStore.getState().setView('mini')
      expect(useArchiveStore.getState().view).toBe('mini')
    })

    it('should switch back to list view', () => {
      useArchiveStore.setState({ view: 'timeline' })
      useArchiveStore.getState().setView('list')
      expect(useArchiveStore.getState().view).toBe('list')
    })
  })

  // ==================== 4. setSearchQuery ====================
  describe('setSearchQuery', () => {
    it('should update search query', () => {
      useArchiveStore.getState().setSearchQuery('report')
      expect(useArchiveStore.getState().searchQuery).toBe('report')
    })

    it('should update to empty string', () => {
      useArchiveStore.setState({ searchQuery: 'old query' })
      useArchiveStore.getState().setSearchQuery('')
      expect(useArchiveStore.getState().searchQuery).toBe('')
    })

    it('should handle unicode search query', () => {
      useArchiveStore.getState().setSearchQuery('搜索中文')
      expect(useArchiveStore.getState().searchQuery).toBe('搜索中文')
    })
  })

  // ==================== 5. clearDiff ====================
  describe('clearDiff', () => {
    it('should clear diffResult and compareTarget', () => {
      useArchiveStore.setState({
        diffResult: mockDiffResult,
        compareTarget: mockArchive2,
      })
      useArchiveStore.getState().clearDiff()
      expect(useArchiveStore.getState().diffResult).toBeNull()
      expect(useArchiveStore.getState().compareTarget).toBeNull()
    })

    it('should work when diffResult and compareTarget are already null', () => {
      useArchiveStore.getState().clearDiff()
      expect(useArchiveStore.getState().diffResult).toBeNull()
      expect(useArchiveStore.getState().compareTarget).toBeNull()
    })
  })

  // ==================== 6. toggleSelect ====================
  describe('toggleSelect', () => {
    it('should add id to selectedIds when not present', () => {
      useArchiveStore.getState().toggleSelect('archive-1')
      expect(useArchiveStore.getState().selectedIds.has('archive-1')).toBe(true)
    })

    it('should remove id from selectedIds when already present', () => {
      useArchiveStore.setState({ selectedIds: new Set(['archive-1', 'archive-2']) })
      useArchiveStore.getState().toggleSelect('archive-1')
      expect(useArchiveStore.getState().selectedIds.has('archive-1')).toBe(false)
      expect(useArchiveStore.getState().selectedIds.has('archive-2')).toBe(true)
    })

    it('should toggle id off and then on again', () => {
      useArchiveStore.getState().toggleSelect('archive-1')
      expect(useArchiveStore.getState().selectedIds.has('archive-1')).toBe(true)
      useArchiveStore.getState().toggleSelect('archive-1')
      expect(useArchiveStore.getState().selectedIds.has('archive-1')).toBe(false)
    })

    it('should handle multiple independent selections', () => {
      useArchiveStore.getState().toggleSelect('archive-1')
      useArchiveStore.getState().toggleSelect('archive-2')
      expect(useArchiveStore.getState().selectedIds).toEqual(new Set(['archive-1', 'archive-2']))
    })
  })

  // ==================== 7. selectAll ====================
  describe('selectAll', () => {
    it('should select all archives by id', () => {
      useArchiveStore.setState({ archives: [mockArchive1, mockArchive2, mockArchive3] })
      useArchiveStore.getState().selectAll()
      expect(useArchiveStore.getState().selectedIds).toEqual(
        new Set(['archive-1', 'archive-2', 'archive-3'])
      )
    })

    it('should work with empty archives list', () => {
      useArchiveStore.setState({ archives: [] })
      useArchiveStore.getState().selectAll()
      expect(useArchiveStore.getState().selectedIds).toEqual(new Set())
    })

    it('should override previous selections', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2],
        selectedIds: new Set(['old-id']),
      })
      useArchiveStore.getState().selectAll()
      expect(useArchiveStore.getState().selectedIds).toEqual(new Set(['archive-1', 'archive-2']))
    })
  })

  // ==================== 8. clearSelection ====================
  describe('clearSelection', () => {
    it('should clear all selected ids', () => {
      useArchiveStore.setState({ selectedIds: new Set(['a', 'b', 'c']) })
      useArchiveStore.getState().clearSelection()
      expect(useArchiveStore.getState().selectedIds).toEqual(new Set())
    })

    it('should work when already empty', () => {
      useArchiveStore.getState().clearSelection()
      expect(useArchiveStore.getState().selectedIds).toEqual(new Set())
    })
  })

  // ==================== 9. fetchArchives 成功 ====================
  describe('fetchArchives', () => {
    it('should load archives on success', async () => {
      const archives = [mockArchive1, mockArchive2]
      mockInvoke.mockResolvedValueOnce(archives)

      await useArchiveStore.getState().fetchArchives()

      expect(mockInvoke).toHaveBeenCalledWith('list_archives', {
        filePath: null,
        search: null,
      })
      expect(useArchiveStore.getState().archives).toEqual(archives)
      expect(useArchiveStore.getState().loading).toBe(false)
      expect(useArchiveStore.getState().error).toBeNull()
    })

    it('should pass filePath and search args to invoke', async () => {
      mockInvoke.mockResolvedValueOnce([])

      await useArchiveStore.getState().fetchArchives('/some/path', 'query')

      expect(mockInvoke).toHaveBeenCalledWith('list_archives', {
        filePath: '/some/path',
        search: 'query',
      })
    })

    it('should set loading=true while fetching', async () => {
      let resolvePromise: (value: any) => void
      const pendingPromise = new Promise((resolve) => {
        resolvePromise = resolve
      })
      mockInvoke.mockReturnValueOnce(pendingPromise)

      const fetchPromise = useArchiveStore.getState().fetchArchives()
      expect(useArchiveStore.getState().loading).toBe(true)
      expect(useArchiveStore.getState().error).toBeNull()

      resolvePromise!([])
      await fetchPromise

      expect(useArchiveStore.getState().loading).toBe(false)
    })

    // ==================== 10. fetchArchives 失败 ====================
    it('should set error on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Network error'))

      await useArchiveStore.getState().fetchArchives()

      expect(useArchiveStore.getState().error).toBe('Network error')
      expect(useArchiveStore.getState().loading).toBe(false)
      expect(useArchiveStore.getState().archives).toEqual([])
    })

    it('should handle non-Error thrown values', async () => {
      mockInvoke.mockRejectedValueOnce('string error')

      await useArchiveStore.getState().fetchArchives()

      expect(useArchiveStore.getState().error).toBe('string error')
      expect(useArchiveStore.getState().loading).toBe(false)
    })
  })

  // ==================== 11. createArchive 成功 ====================
  describe('createArchive', () => {
    it('should create archive and refresh list on success', async () => {
      // First invoke: create_archive, second invoke: list_archives (via fetchArchives), third: get_statistics
      mockInvoke
        .mockResolvedValueOnce(undefined) // create_archive
        .mockResolvedValueOnce([mockArchive1]) // list_archives
        .mockResolvedValueOnce({ total_archives: 1, unique_files: 1, total_size: 100 }) // get_statistics

      await useArchiveStore.getState().createArchive('/path/to/file.pdf', 'my note', ['tag1'])

      expect(mockInvoke).toHaveBeenCalledWith('create_archive', {
        path: '/path/to/file.pdf',
        note: 'my note',
        tags: ['tag1'],
        parentId: null,
      })
      // Should have refreshed the list and statistics
      expect(mockInvoke).toHaveBeenCalledTimes(3)
      expect(mockInvoke).toHaveBeenNthCalledWith(2, 'list_archives', {
        filePath: null,
        search: null,
      })
      expect(useArchiveStore.getState().archives).toEqual([mockArchive1])
      expect(useArchiveStore.getState().loading).toBe(false)
    })

    it('should default note to empty string and tags to empty array', async () => {
      mockInvoke
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      await useArchiveStore.getState().createArchive('/path/to/file.pdf')

      expect(mockInvoke).toHaveBeenCalledWith('create_archive', {
        path: '/path/to/file.pdf',
        note: '',
        tags: [],
        parentId: null,
      })
    })

    // ==================== 12. createArchive 失败 ====================
    it('should set error on creation failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Disk full'))

      await useArchiveStore.getState().createArchive('/path/to/file.pdf')

      expect(useArchiveStore.getState().error).toBe('Disk full')
      expect(useArchiveStore.getState().loading).toBe(false)
    })
  })

  // ==================== 13. deleteArchive 成功 ====================
  describe('deleteArchive', () => {
    it('should delete archive and remove from selectedIds', async () => {
      useArchiveStore.setState({
        selectedIds: new Set(['archive-1', 'archive-2']),
      })

      mockInvoke
        .mockResolvedValueOnce(undefined) // delete_archive
        .mockResolvedValueOnce([mockArchive2]) // list_archives (refresh)

      await useArchiveStore.getState().deleteArchive('archive-1')

      expect(mockInvoke).toHaveBeenCalledWith('delete_archive', { id: 'archive-1' })
      expect(useArchiveStore.getState().selectedIds.has('archive-1')).toBe(false)
      expect(useArchiveStore.getState().selectedIds.has('archive-2')).toBe(true)
      expect(useArchiveStore.getState().loading).toBe(false)
    })

    it('should handle delete when id is not in selectedIds', async () => {
      useArchiveStore.setState({
        selectedIds: new Set(['archive-2']),
      })

      mockInvoke
        .mockResolvedValueOnce(undefined)
        .mockResolvedValueOnce([])

      await useArchiveStore.getState().deleteArchive('archive-999')

      expect(useArchiveStore.getState().selectedIds.has('archive-2')).toBe(true)
      expect(useArchiveStore.getState().selectedIds.has('archive-999')).toBe(false)
    })

    it('should set error on delete failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Permission denied'))

      await useArchiveStore.getState().deleteArchive('archive-1')

      expect(useArchiveStore.getState().error).toBe('Permission denied')
      expect(useArchiveStore.getState().loading).toBe(false)
    })
  })

  // ==================== 14. fetchArchivesPaginated ====================
  describe('fetchArchivesPaginated', () => {
    it('should set hasMore=true when there are more pages', async () => {
      // pageSize=50 (default), total=120, page=1 → 1*50 < 120 → hasMore=true
      mockInvoke.mockResolvedValueOnce([[mockArchive1, mockArchive2], 120])

      await useArchiveStore.getState().fetchArchivesPaginated(1)

      expect(mockInvoke).toHaveBeenCalledWith('list_archives_paginated', {
        filePath: null,
        search: null,
        page: 1,
        pageSize: 50,
      })
      expect(useArchiveStore.getState().archives).toEqual([mockArchive1, mockArchive2])
      expect(useArchiveStore.getState().page).toBe(1)
      expect(useArchiveStore.getState().totalCount).toBe(120)
      expect(useArchiveStore.getState().hasMore).toBe(true)
      expect(useArchiveStore.getState().loading).toBe(false)
    })

    it('should set hasMore=false when on the last page', async () => {
      // pageSize=50, total=50, page=1 → 1*50 = 50 → hasMore=false
      mockInvoke.mockResolvedValueOnce([[mockArchive1], 50])

      await useArchiveStore.getState().fetchArchivesPaginated(1)

      expect(useArchiveStore.getState().hasMore).toBe(false)
    })

    it('should set hasMore=false when page*pageSize > total', async () => {
      // pageSize=50, total=60, page=2 → 2*50=100 > 60 → hasMore=false
      mockInvoke.mockResolvedValueOnce([[mockArchive2], 60])

      await useArchiveStore.getState().fetchArchivesPaginated(2)

      expect(useArchiveStore.getState().hasMore).toBe(false)
      expect(useArchiveStore.getState().page).toBe(2)
    })

    it('should set hasMore=false when empty result', async () => {
      mockInvoke.mockResolvedValueOnce([[], 0])

      await useArchiveStore.getState().fetchArchivesPaginated(1)

      expect(useArchiveStore.getState().hasMore).toBe(false)
      expect(useArchiveStore.getState().totalCount).toBe(0)
      expect(useArchiveStore.getState().archives).toEqual([])
    })

    it('should pass filePath and search args', async () => {
      mockInvoke.mockResolvedValueOnce([[], 0])

      await useArchiveStore.getState().fetchArchivesPaginated(2, '/docs', 'test')

      expect(mockInvoke).toHaveBeenCalledWith('list_archives_paginated', {
        filePath: '/docs',
        search: 'test',
        page: 2,
        pageSize: 50,
      })
    })

    it('should set error on pagination failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('DB error'))

      await useArchiveStore.getState().fetchArchivesPaginated(1)

      expect(useArchiveStore.getState().error).toBe('DB error')
      expect(useArchiveStore.getState().loading).toBe(false)
    })
  })

  // ==================== 15. compareArchives ====================
  describe('compareArchives', () => {
    it('should set diffResult on success', async () => {
      mockInvoke.mockResolvedValueOnce(mockDiffResult)

      await useArchiveStore.getState().compareArchives('archive-1', 'archive-2')

      expect(mockInvoke).toHaveBeenCalledWith('compare_archives', {
        id1: 'archive-1',
        id2: 'archive-2',
      })
      expect(useArchiveStore.getState().diffResult).toEqual(mockDiffResult)
      expect(useArchiveStore.getState().loading).toBe(false)
      expect(useArchiveStore.getState().error).toBeNull()
    })

    it('should clear previous diffResult before comparing', async () => {
      useArchiveStore.setState({ diffResult: mockDiffResult })

      const newDiff: DiffResult = {
        hunks: [],
        stats: { additions: 0, deletions: 0, unchanged: 5 },
      }
      mockInvoke.mockResolvedValueOnce(newDiff)

      await useArchiveStore.getState().compareArchives('archive-1', 'archive-2')

      expect(useArchiveStore.getState().diffResult).toEqual(newDiff)
    })

    it('should set error on comparison failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Archive not found'))

      await useArchiveStore.getState().compareArchives('archive-1', 'non-existent')

      expect(useArchiveStore.getState().error).toBe('Archive not found')
      expect(useArchiveStore.getState().loading).toBe(false)
      expect(useArchiveStore.getState().diffResult).toBeNull()
    })
  })

  // ==================== 额外覆盖: setCompareTarget ====================
  describe('setCompareTarget', () => {
    it('should set compareTarget', () => {
      useArchiveStore.getState().setCompareTarget(mockArchive2)
      expect(useArchiveStore.getState().compareTarget).toEqual(mockArchive2)
    })

    it('should clear compareTarget with null', () => {
      useArchiveStore.setState({ compareTarget: mockArchive2 })
      useArchiveStore.getState().setCompareTarget(null)
      expect(useArchiveStore.getState().compareTarget).toBeNull()
    })
  })

  // ==================== 16. Enhanced Diff ====================
  describe('Enhanced Diff', () => {
    beforeEach(() => {
      useArchiveStore.setState({
        enhancedDiffResult: null,
        loading: false,
        error: null,
      })
    })

    it('should have null enhancedDiffResult initially', () => {
      const { enhancedDiffResult } = useArchiveStore.getState()
      expect(enhancedDiffResult).toBeNull()
    })

    it('should call compare_archives_enhanced command', async () => {
      const mockResult = {
        diff_result: { hunks: [], stats: { additions: 0, deletions: 0, unchanged: 0 } },
        summary: {
          stats: { additions: 0, deletions: 0, unchanged: 0 },
          changes: [],
          change_distribution: { additions: 0, deletions: 0, modifications: 0, moves: 0, renames: 0 },
          affected_regions: [],
          ai_summary: 'No changes',
        },
        file_type: { type: 'Text', encoding: 'UTF-8', line_ending: '\n' },
        preview: null,
      }

      mockInvoke.mockResolvedValueOnce(mockResult)

      await useArchiveStore.getState().compareArchivesEnhanced('id1', 'id2')

      expect(mockInvoke).toHaveBeenCalledWith('compare_archives_enhanced', { id1: 'id1', id2: 'id2' })
    })

    it('should update enhancedDiffResult on success', async () => {
      const mockResult = {
        diff_result: { hunks: [], stats: { additions: 5, deletions: 2, unchanged: 10 } },
        summary: {
          stats: { additions: 5, deletions: 2, unchanged: 0 },
          changes: [
            {
              id: 0,
              change_type: 'Addition',
              location: { file_path: '', start_line: 1, end_line: 1, start_col: null, end_col: null, region_description: null },
              description: 'Added line',
              snippet: 'new code',
              line_count: 1,
            },
          ],
          change_distribution: { additions: 5, deletions: 2, modifications: 0, moves: 0, renames: 0 },
          affected_regions: [],
          ai_summary: '5 additions, 2 deletions',
        },
        file_type: { type: 'Text', encoding: 'UTF-8', line_ending: '\n' },
        preview: null,
      }

      mockInvoke.mockResolvedValueOnce(mockResult)

      await useArchiveStore.getState().compareArchivesEnhanced('id1', 'id2')

      const { enhancedDiffResult, loading } = useArchiveStore.getState()
      expect(enhancedDiffResult).toEqual(mockResult)
      expect(loading).toBe(false)
    })

    it('should set error on failure', async () => {
      mockInvoke.mockRejectedValueOnce(new Error('Compare failed'))

      // 不再抛出异常，错误设置到store中
      await useArchiveStore.getState().compareArchivesEnhanced('id1', 'id2')

      const { error, loading } = useArchiveStore.getState()
      expect(error).toBe('Error: Compare failed')
      expect(loading).toBe(false)
    })

    it('should clear enhancedDiffResult', () => {
      useArchiveStore.setState({
        enhancedDiffResult: {
          diff_result: { hunks: [], stats: { additions: 0, deletions: 0, unchanged: 0 } },
          summary: {
            stats: { additions: 0, deletions: 0, unchanged: 0 },
            changes: [],
            change_distribution: { additions: 0, deletions: 0, modifications: 0, moves: 0, renames: 0 },
            affected_regions: [],
            ai_summary: null,
          },
          file_type: { type: 'Text', encoding: 'UTF-8', line_ending: '\n' },
          preview: null,
        },
      })

      useArchiveStore.getState().clearEnhancedDiff()

      expect(useArchiveStore.getState().enhancedDiffResult).toBeNull()
    })

    it('should set loading true during compare', async () => {
      let resolvePromise: (value: any) => void
      const promise = new Promise((resolve) => {
        resolvePromise = resolve
      })

      mockInvoke.mockReturnValueOnce(promise as any)

      const comparePromise = useArchiveStore.getState().compareArchivesEnhanced('id1', 'id2')

      expect(useArchiveStore.getState().loading).toBe(true)

      resolvePromise!({
        diff_result: { hunks: [], stats: { additions: 0, deletions: 0, unchanged: 0 } },
        summary: {
          stats: { additions: 0, deletions: 0, unchanged: 0 },
          changes: [],
          change_distribution: { additions: 0, deletions: 0, modifications: 0, moves: 0, renames: 0 },
          affected_regions: [],
          ai_summary: null,
        },
        file_type: { type: 'Text', encoding: 'UTF-8', line_ending: '\n' },
        preview: null,
      })

      await comparePromise

      expect(useArchiveStore.getState().loading).toBe(false)
    })
  })
})
