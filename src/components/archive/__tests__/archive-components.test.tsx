/**
 * Archive 组件单元测试
 * 覆盖 ArchiveCard、ArchiveList、CreateArchiveDialog
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ArchiveCard } from '../ArchiveCard'
import { ArchiveList } from '../ArchiveList'
import { CreateArchiveDialog } from '../CreateArchiveDialog'
import { useArchiveStore } from '../../../stores/archiveStore'
import type { Archive } from '../../../types'

// ==================== Mocks ====================

// Mock archiveStore for ArchiveList
const mockFetchArchives = vi.fn()
const mockFetchArchivesPaginated = vi.fn()
const mockCreateArchive = vi.fn()
const mockRestoreArchive = vi.fn()
const mockDeleteArchive = vi.fn()
const mockDeleteArchivesBatch = vi.fn()
const mockCompareArchives = vi.fn()
const mockSelectArchive = vi.fn()
const mockSetSearchQuery = vi.fn()
const mockToggleSelect = vi.fn()
const mockSelectAll = vi.fn()
const mockClearSelection = vi.fn()

vi.mock('../../../stores/archiveStore', () => ({
  useArchiveStore: vi.fn(() => ({
    archives: [],
    selectedArchive: null,
    loading: false,
    searchQuery: '',
    selectedIds: new Set<string>(),
    page: 1,
    hasMore: false,
    fetchArchives: mockFetchArchives,
    fetchArchivesPaginated: mockFetchArchivesPaginated,
    createArchive: mockCreateArchive,
    restoreArchive: mockRestoreArchive,
    deleteArchive: mockDeleteArchive,
    deleteArchivesBatch: mockDeleteArchivesBatch,
    compareArchives: mockCompareArchives,
    selectArchive: mockSelectArchive,
    setSearchQuery: mockSetSearchQuery,
    toggleSelect: mockToggleSelect,
    selectAll: mockSelectAll,
    clearSelection: mockClearSelection,
  })),
}))

// Mock @tauri-apps/api/dialog
vi.mock('@tauri-apps/api/dialog', () => ({
  open: vi.fn(),
}))

// ==================== Fixtures ====================

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
  file_path: '/home/user/docs/notes.txt',
  file_name: 'notes.txt',
  file_size: 2048,
  checksum: 'def456',
  chunk_count: 3,
  note: 'second version',
  tags: ['review', 'draft'],
  parent_id: 'archive-1',
  created_at: '2025-01-02T00:00:00Z',
}

const mockArchiveNoNote: Archive = {
  id: 'archive-3',
  file_path: '/home/user/code/main.ts',
  file_name: 'main.ts',
  file_size: 512,
  checksum: 'ghi789',
  chunk_count: 1,
  note: '',
  tags: [],
  parent_id: null,
  created_at: '2025-01-03T00:00:00Z',
}

// Default props for ArchiveCard
const defaultCardProps = {
  archive: mockArchive1,
  isSelected: false,
  isMultiSelected: false,
  onSelect: vi.fn(),
  onRestore: vi.fn(),
  onDelete: vi.fn(),
  onCompare: vi.fn(),
  onToggleSelect: vi.fn(),
}

// ==================== ArchiveCard Tests ====================

describe('ArchiveCard', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  // 测试 1: 渲染文件名、大小、时间
  it('renders file name, size, and time', () => {
    render(<ArchiveCard {...defaultCardProps} />)

    // 文件名
    expect(screen.getByText('report.pdf')).toBeTruthy()
    // 文件路径
    expect(screen.getByText('/home/user/docs/report.pdf')).toBeTruthy()
    // 文件大小 (formatFileSize: 1024 → "1 KB")
    expect(screen.getByText('1 KB')).toBeTruthy()
    // 块数
    expect(screen.getByText('2 块')).toBeTruthy()
    // note
    expect(screen.getByText('first version')).toBeTruthy()
    // tag
    expect(screen.getByText('important')).toBeTruthy()
  })

  // 测试 2: 选中状态样式
  it('applies selected style when isSelected is true', () => {
    const { container } = render(
      <ArchiveCard {...defaultCardProps} isSelected={true} />
    )

    // 选中时卡片有 ring-2 class 和 border-primary-300
    const card = container.firstChild as HTMLElement
    expect(card.className).toContain('ring-2')
    expect(card.className).toContain('border-primary-300')

    // 选中指示条存在 (absolute left-0 top-3 bottom-3 w-0.5 bg-primary-500)
    const indicator = container.querySelector('.bg-primary-500')
    expect(indicator).toBeTruthy()
  })

  it('applies normal style when isSelected is false', () => {
    const { container } = render(
      <ArchiveCard {...defaultCardProps} isSelected={false} />
    )

    const card = container.firstChild as HTMLElement
    expect(card.className).toContain('border-gray-100')
    expect(card.className).not.toContain('ring-2')

    // 无选中指示条
    const indicator = container.querySelector('.bg-primary-500')
    expect(indicator).toBeNull()
  })

  // 测试 3: 点击选中回调
  it('calls onSelect when card is clicked', () => {
    const onSelect = vi.fn()
    render(<ArchiveCard {...defaultCardProps} onSelect={onSelect} />)

    const card = screen.getByText('report.pdf').closest('div[class*="cursor-pointer"]')!
    fireEvent.click(card)

    expect(onSelect).toHaveBeenCalledTimes(1)
  })

  // 测试 4: 恢复按钮回调
  it('calls onRestore when restore button is clicked', () => {
    const onRestore = vi.fn()
    const { container } = render(
      <ArchiveCard {...defaultCardProps} onRestore={onRestore} />
    )

    // 恢复按钮有 title="恢复"
    const restoreBtn = container.querySelector('button[title="恢复"]')!
    fireEvent.click(restoreBtn)

    expect(onRestore).toHaveBeenCalledTimes(1)
  })

  // 测试 5: 删除按钮回调
  it('calls onDelete when delete button is clicked from menu', () => {
    const onDelete = vi.fn()
    const { container } = render(
      <ArchiveCard {...defaultCardProps} onDelete={onDelete} />
    )

    // 先点击更多按钮（MoreVertical）打开菜单
    const moreBtn = container.querySelector('button:not([title])')!
    // 找到 MoreVertical 按钮 — 它是 actions 区域最后一个没有 title 的按钮
    const actionButtons = container.querySelectorAll('.opacity-0 button')
    const moreButton = actionButtons[actionButtons.length - 1]
    fireEvent.click(moreButton)

    // 菜单出现后点击删除
    const deleteBtn = screen.getByText('删除')
    fireEvent.click(deleteBtn)

    expect(onDelete).toHaveBeenCalledTimes(1)
  })
})

// ==================== ArchiveList Tests ====================

describe('ArchiveList', () => {
  const mockedUseArchiveStore = vi.mocked(useArchiveStore)

  beforeEach(() => {
    vi.clearAllMocks()
    // Default: empty store
    mockedUseArchiveStore.mockReturnValue({
      archives: [],
      selectedArchive: null,
      loading: false,
      searchQuery: '',
      selectedIds: new Set<string>(),
      page: 1,
      hasMore: false,
      fetchArchives: mockFetchArchives,
      fetchArchivesPaginated: mockFetchArchivesPaginated,
      createArchive: mockCreateArchive,
      restoreArchive: mockRestoreArchive,
      deleteArchive: mockDeleteArchive,
      deleteArchivesBatch: mockDeleteArchivesBatch,
      compareArchives: mockCompareArchives,
      selectArchive: mockSelectArchive,
      setSearchQuery: mockSetSearchQuery,
      toggleSelect: mockToggleSelect,
      selectAll: mockSelectAll,
      clearSelection: mockClearSelection,
    })
  })

  // 测试 6: 空状态显示占位提示
  it('shows empty state placeholder when no archives', () => {
    render(<ArchiveList />)

    expect(screen.getByText('暂无存档')).toBeTruthy()
    expect(screen.getByText(/点击「新建存档」开始追踪文件历史/)).toBeTruthy()
  })

  // 测试 7: 列表渲染所有 archive 项
  it('renders all archive items in the list', () => {
    mockedUseArchiveStore.mockReturnValue({
      archives: [mockArchive1, mockArchive2],
      selectedArchive: null,
      loading: false,
      searchQuery: '',
      selectedIds: new Set<string>(),
      page: 1,
      hasMore: false,
      fetchArchives: mockFetchArchives,
      fetchArchivesPaginated: mockFetchArchivesPaginated,
      createArchive: mockCreateArchive,
      restoreArchive: mockRestoreArchive,
      deleteArchive: mockDeleteArchive,
      deleteArchivesBatch: mockDeleteArchivesBatch,
      compareArchives: mockCompareArchives,
      selectArchive: mockSelectArchive,
      setSearchQuery: mockSetSearchQuery,
      toggleSelect: mockToggleSelect,
      selectAll: mockSelectAll,
      clearSelection: mockClearSelection,
    })

    render(<ArchiveList />)

    // 两个文件名都应出现
    expect(screen.getByText('report.pdf')).toBeTruthy()
    expect(screen.getByText('notes.txt')).toBeTruthy()

    // 标题
    expect(screen.getByText('存档管理')).toBeTruthy()

    // 数量标签
    expect(screen.getByText('2')).toBeTruthy()
  })

  // 测试 8: 加载中显示 loading
  it('shows loading skeleton when loading and no archives', () => {
    mockedUseArchiveStore.mockReturnValue({
      archives: [],
      selectedArchive: null,
      loading: true,
      searchQuery: '',
      selectedIds: new Set<string>(),
      page: 1,
      hasMore: false,
      fetchArchives: mockFetchArchives,
      fetchArchivesPaginated: mockFetchArchivesPaginated,
      createArchive: mockCreateArchive,
      restoreArchive: mockRestoreArchive,
      deleteArchive: mockDeleteArchive,
      deleteArchivesBatch: mockDeleteArchivesBatch,
      compareArchives: mockCompareArchives,
      selectArchive: mockSelectArchive,
      setSearchQuery: mockSetSearchQuery,
      toggleSelect: mockToggleSelect,
      selectAll: mockSelectAll,
      clearSelection: mockClearSelection,
    })

    const { container } = render(<ArchiveList />)

    // Loading 骨架屏有 animate-pulse-slow class
    const skeletons = container.querySelectorAll('.animate-pulse-slow')
    expect(skeletons.length).toBe(3)

    // 空状态提示不应出现
    expect(screen.queryByText('暂无存档')).toBeNull()
  })
})

// ==================== CreateArchiveDialog Tests ====================

describe('CreateArchiveDialog', () => {
  beforeEach(() => {
    vi.clearAllMocks()
  })

  // 测试 9: 表单输入路径/备注/标签
  it('renders form with path, note, and tag inputs', () => {
    render(
      <CreateArchiveDialog
        defaultPath="/home/user/test.txt"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />
    )

    // 标题
    expect(screen.getByText('新建存档')).toBeTruthy()

    // 文件路径输入框 — 有默认值
    const pathInput = screen.getByDisplayValue('/home/user/test.txt')
    expect(pathInput).toBeTruthy()

    // 备注 textarea
    const noteTextarea = screen.getByPlaceholderText('添加备注说明...')
    expect(noteTextarea).toBeTruthy()

    // 标签输入框
    const tagInput = screen.getByPlaceholderText('输入标签...')
    expect(tagInput).toBeTruthy()

    // 取消和创建按钮
    expect(screen.getByText('取消')).toBeTruthy()
    expect(screen.getByText('创建存档')).toBeTruthy()
  })

  it('allows typing in note and tag fields', () => {
    render(
      <CreateArchiveDialog
        defaultPath="/path"
        onConfirm={vi.fn()}
        onCancel={vi.fn()}
      />
    )

    // 输入备注
    const noteTextarea = screen.getByPlaceholderText('添加备注说明...')
    fireEvent.change(noteTextarea, { target: { value: 'my note' } })
    expect((noteTextarea as HTMLTextAreaElement).value).toBe('my note')

    // 输入标签并按 Enter 添加
    const tagInput = screen.getByPlaceholderText('输入标签...')
    fireEvent.change(tagInput, { target: { value: 'bugfix' } })
    fireEvent.keyDown(tagInput, { key: 'Enter' })

    // 标签应显示
    expect(screen.getByText('bugfix')).toBeTruthy()
    // 输入框应清空
    expect((tagInput as HTMLInputElement).value).toBe('')
  })

  // 测试 10: 提交调用 onConfirm（即 createArchive 的入口）
  it('calls onConfirm with path, note, and tags on submit', () => {
    const onConfirm = vi.fn()
    render(
      <CreateArchiveDialog
        defaultPath="/home/user/file.txt"
        onConfirm={onConfirm}
        onCancel={vi.fn()}
      />
    )

    // 填写备注
    const noteTextarea = screen.getByPlaceholderText('添加备注说明...')
    fireEvent.change(noteTextarea, { target: { value: 'test note' } })

    // 添加标签
    const tagInput = screen.getByPlaceholderText('输入标签...')
    fireEvent.change(tagInput, { target: { value: 'v1' } })
    fireEvent.keyDown(tagInput, { key: 'Enter' })
    fireEvent.change(tagInput, { target: { value: 'release' } })
    fireEvent.keyDown(tagInput, { key: 'Enter' })

    // 点击创建
    const createBtn = screen.getByText('创建存档')
    fireEvent.click(createBtn)

    expect(onConfirm).toHaveBeenCalledWith(
      '/home/user/file.txt',
      'test note',
      ['v1', 'release']
    )
  })

  it('calls onCancel when cancel button is clicked', () => {
    const onCancel = vi.fn()
    render(
      <CreateArchiveDialog
        defaultPath="/path"
        onConfirm={vi.fn()}
        onCancel={onCancel}
      />
    )

    fireEvent.click(screen.getByText('取消'))
    expect(onCancel).toHaveBeenCalledTimes(1)
  })
})
