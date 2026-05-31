import { describe, it, expect, vi, beforeEach } from 'vitest'
import { useToastStore, toast } from '../toastStore'

describe('useToastStore', () => {
  beforeEach(() => {
    useToastStore.setState({ toasts: [] })
    vi.useRealTimers()
  })

  it('初始状态 toasts 为空', () => {
    const { toasts } = useToastStore.getState()
    expect(toasts).toEqual([])
  })

  it('addToast 添加 toast 并生成唯一 id', () => {
    useToastStore.getState().addToast({ type: 'success', title: 'test', duration: 3000, dismissible: true })
    const { toasts } = useToastStore.getState()
    expect(toasts).toHaveLength(1)
    expect(toasts[0].id).toMatch(/^toast-/)
    expect(toasts[0].title).toBe('test')
    expect(toasts[0].type).toBe('success')
  })

  it('addToast 自动设置默认 duration=3000', () => {
    useToastStore.getState().addToast({ type: 'info', title: 'default duration' } as any)
    const { toasts } = useToastStore.getState()
    expect(toasts[0].duration).toBe(3000)
  })

  it('addToast 超过 5 条自动截断旧的', () => {
    const store = useToastStore.getState()
    for (let i = 0; i < 7; i++) {
      store.addToast({ type: 'info', title: `toast-${i}`, duration: 0, dismissible: true })
    }
    const { toasts } = useToastStore.getState()
    expect(toasts).toHaveLength(5)
    expect(toasts[0].title).toBe('toast-2')
    expect(toasts[4].title).toBe('toast-6')
  })

  it('removeToast 按 id 移除', () => {
    useToastStore.getState().addToast({ type: 'info', title: 'a', duration: 0, dismissible: true })
    useToastStore.getState().addToast({ type: 'info', title: 'b', duration: 0, dismissible: true })
    const { toasts } = useToastStore.getState()
    expect(toasts).toHaveLength(2)
    useToastStore.getState().removeToast(toasts[0].id)
    const after = useToastStore.getState().toasts
    expect(after).toHaveLength(1)
    expect(after[0].title).toBe('b')
  })

  it('clearAll 清空所有 toast', () => {
    useToastStore.getState().addToast({ type: 'info', title: 'a', duration: 0, dismissible: true })
    useToastStore.getState().addToast({ type: 'info', title: 'b', duration: 0, dismissible: true })
    expect(useToastStore.getState().toasts).toHaveLength(2)
    useToastStore.getState().clearAll()
    expect(useToastStore.getState().toasts).toEqual([])
  })

  describe('toast 便捷方法', () => {
    it('toast.success 创建 type=success', () => {
      toast.success('ok')
      const t = useToastStore.getState().toasts[0]
      expect(t.type).toBe('success')
      expect(t.title).toBe('ok')
      expect(t.duration).toBe(3000)
    })

    it('toast.error 创建 type=error，duration=5000', () => {
      toast.error('fail')
      const t = useToastStore.getState().toasts[0]
      expect(t.type).toBe('error')
      expect(t.duration).toBe(5000)
    })

    it('toast.warning 创建 type=warning', () => {
      toast.warning('warn')
      const t = useToastStore.getState().toasts[0]
      expect(t.type).toBe('warning')
      expect(t.duration).toBe(4000)
    })

    it('toast.info 创建 type=info', () => {
      toast.info('info')
      const t = useToastStore.getState().toasts[0]
      expect(t.type).toBe('info')
      expect(t.duration).toBe(3000)
    })
  })

  it('自动消失 — 使用 vi.useFakeTimers 验证 setTimeout', () => {
    vi.useFakeTimers()
    useToastStore.getState().addToast({ type: 'success', title: 'auto-dismiss', duration: 3000, dismissible: true })
    expect(useToastStore.getState().toasts).toHaveLength(1)

    vi.advanceTimersByTime(2999)
    expect(useToastStore.getState().toasts).toHaveLength(1)

    vi.advanceTimersByTime(1)
    expect(useToastStore.getState().toasts).toHaveLength(0)
  })
})
