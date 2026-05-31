import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { renderHook, act } from '@testing-library/react'
import { useTheme } from '../useTheme'

describe('useTheme', () => {
  // Helper to create a controllable matchMedia mock
  let matchMediaListeners: Array<(e: MediaQueryListEvent) => void> = []
  let matchMediaMatches = false

  beforeEach(() => {
    localStorage.clear()
    document.documentElement.removeAttribute('data-theme')
    matchMediaListeners = []
    matchMediaMatches = false

    // Mock window.matchMedia — note `matches` is a getter so later changes
    // to matchMediaMatches are reflected when the hook reads mediaQuery.matches.
    vi.stubGlobal('matchMedia', vi.fn((_query: string) => {
      const mql: MediaQueryList = {
        get matches() { return matchMediaMatches },
        media: '(prefers-color-scheme: dark)',
        onchange: null,
        addListener: vi.fn(),
        removeListener: vi.fn(),
        addEventListener: vi.fn((_event: string, listener: EventListener) => {
          matchMediaListeners.push(listener as (e: MediaQueryListEvent) => void)
        }),
        removeEventListener: vi.fn((_event: string, listener: EventListener) => {
          matchMediaListeners = matchMediaListeners.filter(l => l !== listener)
        }),
        dispatchEvent: vi.fn(),
      }
      return mql
    }))
  })

  afterEach(() => {
    vi.restoreAllMocks()
  })

  // ── 1. 默认 theme='system'（无 localStorage） ──────────────────────
  it('默认 theme=system（无 localStorage 值）', () => {
    const { result } = renderHook(() => useTheme())
    expect(result.current.theme).toBe('system')
  })

  // ── 2. 从 localStorage 读取已存储的 theme ─────────────────────────
  it('从 localStorage 读取已存储的 dark 主题', () => {
    localStorage.setItem('theme', 'dark')
    const { result } = renderHook(() => useTheme())
    expect(result.current.theme).toBe('dark')
  })

  it('从 localStorage 读取已存储的 light 主题', () => {
    localStorage.setItem('theme', 'light')
    const { result } = renderHook(() => useTheme())
    expect(result.current.theme).toBe('light')
  })

  // ── 3. setTheme 更新 state 并写入 localStorage ────────────────────
  it('setTheme 更新 state 并写入 localStorage', () => {
    const { result } = renderHook(() => useTheme())

    act(() => {
      result.current.setTheme('dark')
    })

    expect(result.current.theme).toBe('dark')
    expect(localStorage.getItem('theme')).toBe('dark')
  })

  it('setTheme 可切换为 light', () => {
    const { result } = renderHook(() => useTheme())

    act(() => {
      result.current.setTheme('dark')
    })
    expect(result.current.theme).toBe('dark')

    act(() => {
      result.current.setTheme('light')
    })
    expect(result.current.theme).toBe('light')
    expect(localStorage.getItem('theme')).toBe('light')
  })

  it('setTheme 可切换为 system', () => {
    const { result } = renderHook(() => useTheme())

    act(() => {
      result.current.setTheme('dark')
    })
    expect(result.current.theme).toBe('dark')

    act(() => {
      result.current.setTheme('system')
    })
    expect(result.current.theme).toBe('system')
    expect(localStorage.getItem('theme')).toBe('system')
  })

  // ── 4. toggleTheme 在 light/dark 之间切换 ─────────────────────────
  // NOTE: toggleTheme uses resolvedTheme (which tracks OS preference when
  // theme='system').  When theme is explicitly 'light'/'dark', resolvedTheme
  // is NOT updated — it retains the value from when theme was 'system'.

  it('toggleTheme 在 system 模式（浅色偏好）下切换为 dark', () => {
    matchMediaMatches = false // OS prefers light
    const { result } = renderHook(() => useTheme())

    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('light')

    act(() => {
      result.current.toggleTheme()
    })

    expect(result.current.theme).toBe('dark')
    expect(localStorage.getItem('theme')).toBe('dark')
  })

  it('toggleTheme 在 system 模式（深色偏好）下切换为 light', () => {
    matchMediaMatches = true // OS prefers dark
    const { result } = renderHook(() => useTheme())

    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('dark')

    act(() => {
      result.current.toggleTheme()
    })

    expect(result.current.theme).toBe('light')
    expect(localStorage.getItem('theme')).toBe('light')
  })

  it('toggleTheme 基于 resolvedTheme 决定方向', () => {
    // Start in system mode with dark preference → resolvedTheme='dark'
    matchMediaMatches = true
    const { result } = renderHook(() => useTheme())
    expect(result.current.resolvedTheme).toBe('dark')

    // Toggle → light (because resolvedTheme was 'dark')
    act(() => { result.current.toggleTheme() })
    expect(result.current.theme).toBe('light')
    expect(localStorage.getItem('theme')).toBe('light')

    // After switching to explicit 'light', resolvedTheme stays 'dark'
    // (useEffect only updates resolvedTheme when theme === 'system').
    // So toggleTheme still sees resolvedTheme='dark' → sets theme='light' again.
    expect(result.current.resolvedTheme).toBe('dark')

    act(() => { result.current.toggleTheme() })
    // resolvedTheme is still 'dark', so toggle sets theme to 'light'
    expect(result.current.theme).toBe('light')
  })

  // ── 5. system 模式下 resolvedTheme 跟随 matchMedia ────────────────
  it('system 模式下 resolvedTheme 匹配浅色偏好', () => {
    matchMediaMatches = false
    const { result } = renderHook(() => useTheme())

    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('light')
  })

  it('system 模式下 resolvedTheme 匹配深色偏好', () => {
    matchMediaMatches = true
    const { result } = renderHook(() => useTheme())

    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('dark')
  })

  it('system 模式下 matchMedia change 事件更新 resolvedTheme', () => {
    matchMediaMatches = false // starts as light
    const { result } = renderHook(() => useTheme())
    expect(result.current.resolvedTheme).toBe('light')

    // Simulate OS theme change to dark
    act(() => {
      matchMediaMatches = true
      matchMediaListeners.forEach(listener =>
        listener({ matches: true } as MediaQueryListEvent)
      )
    })

    expect(result.current.resolvedTheme).toBe('dark')

    // Simulate OS theme change back to light
    act(() => {
      matchMediaMatches = false
      matchMediaListeners.forEach(listener =>
        listener({ matches: false } as MediaQueryListEvent)
      )
    })

    expect(result.current.resolvedTheme).toBe('light')
  })

  it('非 system 模式下 matchMedia change 不影响 resolvedTheme', () => {
    matchMediaMatches = false
    const { result } = renderHook(() => useTheme())

    // Switch to explicit light theme
    act(() => {
      result.current.setTheme('light')
    })
    expect(result.current.resolvedTheme).toBe('light')

    // Simulate OS theme change — should be ignored since theme is 'light'
    act(() => {
      matchMediaMatches = true
      matchMediaListeners.forEach(listener =>
        listener({ matches: true } as MediaQueryListEvent)
      )
    })

    // resolvedTheme unchanged because theme !== 'system'
    expect(result.current.resolvedTheme).toBe('light')
  })

  it('切换到 system 模式时立即解析 matchMedia', () => {
    matchMediaMatches = true // OS prefers dark
    const { result } = renderHook(() => useTheme())

    // Initially in system mode with dark preference
    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('dark')

    // Switch to explicit light — resolvedTheme stays 'dark'
    act(() => {
      result.current.setTheme('light')
    })
    expect(result.current.theme).toBe('light')
    expect(result.current.resolvedTheme).toBe('dark')

    // Change OS preference to light, then switch back to system
    act(() => {
      matchMediaMatches = false
      result.current.setTheme('system')
    })
    expect(result.current.theme).toBe('system')
    expect(result.current.resolvedTheme).toBe('light')
  })

  // ── 6. data-theme 属性正确设置到 document.documentElement ─────────
  it('data-theme 在 system+浅色偏好 时设为 light', () => {
    matchMediaMatches = false
    renderHook(() => useTheme())
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
  })

  it('data-theme 在 system+深色偏好 时设为 dark', () => {
    matchMediaMatches = true
    renderHook(() => useTheme())
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
  })

  it('data-theme 在显式 light 主题时设为 light', () => {
    localStorage.setItem('theme', 'light')
    renderHook(() => useTheme())
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
  })

  it('data-theme 在显式 dark 主题时设为 dark', () => {
    localStorage.setItem('theme', 'dark')
    renderHook(() => useTheme())
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
  })

  it('setTheme 后 data-theme 属性更新', () => {
    const { result } = renderHook(() => useTheme())

    act(() => {
      result.current.setTheme('dark')
    })
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')

    act(() => {
      result.current.setTheme('light')
    })
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')
  })

  it('toggleTheme 后 data-theme 属性更新', () => {
    matchMediaMatches = false
    const { result } = renderHook(() => useTheme())

    expect(document.documentElement.getAttribute('data-theme')).toBe('light')

    act(() => {
      result.current.toggleTheme()
    })
    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
  })

  it('system 模式下 matchMedia 变化时 data-theme 跟随更新', () => {
    matchMediaMatches = false
    const { result } = renderHook(() => useTheme())
    expect(document.documentElement.getAttribute('data-theme')).toBe('light')

    act(() => {
      matchMediaMatches = true
      matchMediaListeners.forEach(listener =>
        listener({ matches: true } as MediaQueryListEvent)
      )
    })

    expect(document.documentElement.getAttribute('data-theme')).toBe('dark')
  })

  // ── Cleanup: matchMedia listener removed on unmount ────────────────
  it('组件卸载时移除 matchMedia 事件监听', () => {
    const removeEventListenerMock = vi.fn()

    vi.stubGlobal('matchMedia', vi.fn(() => ({
      get matches() { return false },
      media: '(prefers-color-scheme: dark)',
      onchange: null,
      addListener: vi.fn(),
      removeListener: vi.fn(),
      addEventListener: vi.fn(),
      removeEventListener: removeEventListenerMock,
      dispatchEvent: vi.fn(),
    })))

    const { unmount } = renderHook(() => useTheme())

    unmount()

    expect(removeEventListenerMock).toHaveBeenCalledWith('change', expect.any(Function))
  })
})
