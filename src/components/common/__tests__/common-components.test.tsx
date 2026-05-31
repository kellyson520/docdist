/**
 * Common 组件单元测试
 * 覆盖: ToastContainer, SearchBar, TagBadge, ConfirmDialog, ThemeToggle
 */
import { describe, it, expect, vi, beforeEach } from 'vitest'
import { render, screen, fireEvent } from '@testing-library/react'
import { ToastContainer } from '../ToastContainer'
import { SearchBar } from '../SearchBar'
import { TagBadge } from '../TagBadge'
import { ConfirmDialog } from '../ConfirmDialog'
import { ThemeToggle } from '../ThemeToggle'

// ─── Mock stores ───────────────────────────────────────────────
const mockRemoveToast = vi.fn()
let mockToasts: any[] = []

vi.mock('../../../stores/toastStore', () => ({
  useToastStore: vi.fn(() => ({
    toasts: mockToasts,
    removeToast: mockRemoveToast,
  })),
}))

// ─── Mock hooks ────────────────────────────────────────────────
const mockSetTheme = vi.fn()
let mockTheme: string = 'light'

vi.mock('../../../hooks/useTheme', () => ({
  useTheme: vi.fn(() => ({
    theme: mockTheme,
    setTheme: mockSetTheme,
  })),
}))

// ─── Mock getTagColor (used by TagBadge) ───────────────────────
vi.mock('../../../utils/format', () => ({
  getTagColor: vi.fn(() => 'bg-blue-100 text-blue-700'),
}))

// ─── Tests ─────────────────────────────────────────────────────

describe('Common Components', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    mockToasts = []
    mockTheme = 'light'
  })

  // ── ToastContainer ─────────────────────────────────────────

  describe('ToastContainer', () => {
    it('renders nothing when toasts is empty', () => {
      const { container } = render(<ToastContainer />)
      expect(container.innerHTML).toBe('')
    })

    it('renders toasts from toastStore', () => {
      mockToasts = [
        { id: '1', type: 'success', title: '操作成功', message: '文件已保存', duration: 3000, dismissible: true },
        { id: '2', type: 'error', title: '操作失败', duration: 5000, dismissible: true },
      ]
      render(<ToastContainer />)

      expect(screen.getByText('操作成功')).toBeInTheDocument()
      expect(screen.getByText('文件已保存')).toBeInTheDocument()
      expect(screen.getByText('操作失败')).toBeInTheDocument()
    })

    it('calls removeToast when close button is clicked', () => {
      mockToasts = [
        { id: 'toast-1', type: 'info', title: '提示', duration: 3000, dismissible: true },
      ]
      render(<ToastContainer />)

      const closeButton = screen.getByRole('button')
      fireEvent.click(closeButton)

      expect(mockRemoveToast).toHaveBeenCalledWith('toast-1')
    })
  })

  // ── SearchBar ──────────────────────────────────────────────

  describe('SearchBar', () => {
    it('displays the current value', () => {
      render(<SearchBar value="test query" onChange={() => {}} />)
      const input = screen.getByPlaceholderText('搜索存档...')
      expect(input).toHaveValue('test query')
    })

    it('calls onChange when user types', () => {
      const handleChange = vi.fn()
      render(<SearchBar value="" onChange={handleChange} />)

      const input = screen.getByPlaceholderText('搜索存档...')
      fireEvent.change(input, { target: { value: 'hello' } })

      expect(handleChange).toHaveBeenCalledWith('hello')
    })

    it('uses custom placeholder when provided', () => {
      render(<SearchBar value="" onChange={() => {}} placeholder="自定义提示" />)
      expect(screen.getByPlaceholderText('自定义提示')).toBeInTheDocument()
    })
  })

  // ── TagBadge ───────────────────────────────────────────────

  describe('TagBadge', () => {
    it('renders tag text', () => {
      render(<TagBadge tag="React" />)
      expect(screen.getByText('React')).toBeInTheDocument()
    })

    it('calls onRemove when delete button is clicked', () => {
      const handleRemove = vi.fn()
      render(<TagBadge tag="TypeScript" onRemove={handleRemove} />)

      const removeBtn = screen.getByText('×')
      fireEvent.click(removeBtn)

      expect(handleRemove).toHaveBeenCalledTimes(1)
    })

    it('does not render remove button when onRemove is not provided', () => {
      render(<TagBadge tag="JavaScript" />)
      expect(screen.queryByText('×')).not.toBeInTheDocument()
    })
  })

  // ── ConfirmDialog ──────────────────────────────────────────

  describe('ConfirmDialog', () => {
    const defaultProps = {
      open: true,
      title: '删除确认',
      message: '确定要删除该文件吗？',
      onConfirm: vi.fn(),
      onCancel: vi.fn(),
    }

    it('renders title and message', () => {
      render(<ConfirmDialog {...defaultProps} />)

      expect(screen.getByText('删除确认')).toBeInTheDocument()
      expect(screen.getByText('确定要删除该文件吗？')).toBeInTheDocument()
    })

    it('renders nothing when open is false', () => {
      const { container } = render(<ConfirmDialog {...defaultProps} open={false} />)
      expect(container.innerHTML).toBe('')
    })

    it('calls onConfirm when confirm button is clicked', () => {
      const onConfirm = vi.fn()
      render(<ConfirmDialog {...defaultProps} onConfirm={onConfirm} />)

      fireEvent.click(screen.getByText('确认'))
      expect(onConfirm).toHaveBeenCalledTimes(1)
    })

    it('calls onCancel when cancel button is clicked', () => {
      const onCancel = vi.fn()
      render(<ConfirmDialog {...defaultProps} onCancel={onCancel} />)

      fireEvent.click(screen.getByText('取消'))
      expect(onCancel).toHaveBeenCalledTimes(1)
    })
  })

  // ── ThemeToggle ────────────────────────────────────────────

  describe('ThemeToggle', () => {
    it('renders all theme option buttons', () => {
      render(<ThemeToggle />)

      expect(screen.getByTitle('浅色')).toBeInTheDocument()
      expect(screen.getByTitle('深色')).toBeInTheDocument()
      expect(screen.getByTitle('系统')).toBeInTheDocument()
    })

    it('calls setTheme with "dark" when dark button is clicked', () => {
      render(<ThemeToggle />)

      fireEvent.click(screen.getByTitle('深色'))
      expect(mockSetTheme).toHaveBeenCalledWith('dark')
    })

    it('calls setTheme with "light" when light button is clicked', () => {
      mockTheme = 'dark'
      render(<ThemeToggle />)

      fireEvent.click(screen.getByTitle('浅色'))
      expect(mockSetTheme).toHaveBeenCalledWith('light')
    })

    it('calls setTheme with "system" when system button is clicked', () => {
      render(<ThemeToggle />)

      fireEvent.click(screen.getByTitle('系统'))
      expect(mockSetTheme).toHaveBeenCalledWith('system')
    })
  })
})
