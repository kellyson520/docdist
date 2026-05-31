/**
 * useArchive hook 单元测试
 * 覆盖: 空状态、archiveByFile 过滤、totalSize 计算、uniqueFiles 统计、selectedArchives 过滤、filteredArchives 透传
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { renderHook } from '@testing-library/react'
import { useArchive } from '../useArchive'
import { useArchiveStore } from '../../stores/archiveStore'
import type { Archive } from '../../types'

// Mock toastStore (archiveStore 依赖)
vi.mock('../../stores/toastStore', () => ({
  toast: {
    success: vi.fn(),
    error: vi.fn(),
    warning: vi.fn(),
    info: vi.fn(),
  },
}))

// Mock logger (archiveStore 依赖)
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

// ==================== Tests ====================

describe('useArchive', () => {
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
  })

  // ==================== 1. 空 archives 时 totalSize=0, uniqueFiles=0 ====================
  describe('empty archives', () => {
    it('totalSize should be 0 when archives is empty', () => {
      const { result } = renderHook(() => useArchive())
      expect(result.current.totalSize).toBe(0)
    })

    it('uniqueFiles should be 0 when archives is empty', () => {
      const { result } = renderHook(() => useArchive())
      expect(result.current.uniqueFiles).toBe(0)
    })

    it('archives should be an empty array', () => {
      const { result } = renderHook(() => useArchive())
      expect(result.current.archives).toEqual([])
    })

    it('selectedArchives should be an empty array when no selection', () => {
      const { result } = renderHook(() => useArchive())
      expect(result.current.selectedArchives).toEqual([])
    })

    it('filteredArchives should be an empty array', () => {
      const { result } = renderHook(() => useArchive())
      expect(result.current.filteredArchives).toEqual([])
    })
  })

  // ==================== 2. archiveByFile 按 file_path 正确过滤 ====================
  describe('archiveByFile', () => {
    it('should return archives matching the given file_path', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
      })

      const { result } = renderHook(() => useArchive())
      const filtered = result.current.archiveByFile('/home/user/docs/report.pdf')

      expect(filtered).toHaveLength(2)
      expect(filtered).toEqual([mockArchive1, mockArchive2])
    })

    it('should return empty array when no archives match the file_path', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
      })

      const { result } = renderHook(() => useArchive())
      const filtered = result.current.archiveByFile('/nonexistent/path.txt')

      expect(filtered).toHaveLength(0)
      expect(filtered).toEqual([])
    })

    it('should return single archive when only one matches', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
      })

      const { result } = renderHook(() => useArchive())
      const filtered = result.current.archiveByFile('/home/user/docs/notes.txt')

      expect(filtered).toHaveLength(1)
      expect(filtered).toEqual([mockArchive3])
    })

    it('should return empty array when archives list is empty', () => {
      const { result } = renderHook(() => useArchive())
      const filtered = result.current.archiveByFile('/any/path')

      expect(filtered).toHaveLength(0)
    })
  })

  // ==================== 3. totalSize 计算所有 file_size 之和 ====================
  describe('totalSize', () => {
    it('should sum all file_size values', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
      })

      const { result } = renderHook(() => useArchive())
      // 1024 + 2048 + 512 = 3584
      expect(result.current.totalSize).toBe(3584)
    })

    it('should handle single archive', () => {
      useArchiveStore.setState({
        archives: [mockArchive1],
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.totalSize).toBe(1024)
    })

    it('should handle archives with zero file_size', () => {
      const zeroSizeArchive: Archive = { ...mockArchive1, file_size: 0 }
      useArchiveStore.setState({
        archives: [zeroSizeArchive, mockArchive3],
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.totalSize).toBe(512)
    })
  })

  // ==================== 4. uniqueFiles 统计去重 file_path 数量 ====================
  describe('uniqueFiles', () => {
    it('should count unique file_path values', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
      })

      const { result } = renderHook(() => useArchive())
      // mockArchive1 and mockArchive2 share the same file_path, mockArchive3 is different
      expect(result.current.uniqueFiles).toBe(2)
    })

    it('should return 1 when all archives have the same file_path', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2],
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.uniqueFiles).toBe(1)
    })

    it('should return count of all archives when all have different file_paths', () => {
      const archive4: Archive = { ...mockArchive3, id: 'archive-4', file_path: '/other/file.md' }
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive3, archive4],
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.uniqueFiles).toBe(3)
    })
  })

  // ==================== 5. selectedArchives 按 selectedIds 过滤 ====================
  describe('selectedArchives', () => {
    it('should return only archives whose ids are in selectedIds', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
        selectedIds: new Set(['archive-1', 'archive-3']),
      })

      const { result } = renderHook(() => useArchive())

      expect(result.current.selectedArchives).toHaveLength(2)
      expect(result.current.selectedArchives).toEqual([mockArchive1, mockArchive3])
    })

    it('should return empty array when selectedIds is empty', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
        selectedIds: new Set(),
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.selectedArchives).toEqual([])
    })

    it('should return empty array when no archive matches selectedIds', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2],
        selectedIds: new Set(['nonexistent-id']),
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.selectedArchives).toEqual([])
    })

    it('should return single archive when one id matches', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
        selectedIds: new Set(['archive-2']),
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.selectedArchives).toHaveLength(1)
      expect(result.current.selectedArchives).toEqual([mockArchive2])
    })

    it('should select all archives when all ids are selected', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
        selectedIds: new Set(['archive-1', 'archive-2', 'archive-3']),
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.selectedArchives).toHaveLength(3)
      expect(result.current.selectedArchives).toEqual([mockArchive1, mockArchive2, mockArchive3])
    })
  })

  // ==================== 6. filteredArchives 透传 archives ====================
  describe('filteredArchives', () => {
    it('should equal archives (passthrough)', () => {
      useArchiveStore.setState({
        archives: [mockArchive1, mockArchive2, mockArchive3],
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.filteredArchives).toEqual(result.current.archives)
      expect(result.current.filteredArchives).toEqual([mockArchive1, mockArchive2, mockArchive3])
    })

    it('should be empty when archives is empty', () => {
      const { result } = renderHook(() => useArchive())
      expect(result.current.filteredArchives).toEqual([])
      expect(result.current.filteredArchives).toEqual(result.current.archives)
    })

    it('should contain the same elements as archives', () => {
      useArchiveStore.setState({
        archives: [mockArchive1],
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.filteredArchives).toHaveLength(1)
      expect(result.current.filteredArchives[0]).toBe(mockArchive1)
    })
  })

  // ==================== 7. Store actions 透传验证 ====================
  describe('store actions passthrough', () => {
    it('should expose store action functions', () => {
      const { result } = renderHook(() => useArchive())

      expect(typeof result.current.fetchArchives).toBe('function')
      expect(typeof result.current.createArchive).toBe('function')
      expect(typeof result.current.restoreArchive).toBe('function')
      expect(typeof result.current.deleteArchive).toBe('function')
      expect(typeof result.current.selectArchive).toBe('function')
      expect(typeof result.current.setSearchQuery).toBe('function')
    })

    it('should expose store state values', () => {
      const { result } = renderHook(() => useArchive())

      expect(result.current.searchQuery).toBe('')
      expect(result.current.loading).toBe(false)
      expect(result.current.error).toBeNull()
      expect(result.current.selectedArchive).toBeNull()
    })

    it('should expose selectedIds from store', () => {
      useArchiveStore.setState({
        selectedIds: new Set(['archive-1']),
      })

      const { result } = renderHook(() => useArchive())
      expect(result.current.selectedIds).toEqual(new Set(['archive-1']))
    })
  })
})
