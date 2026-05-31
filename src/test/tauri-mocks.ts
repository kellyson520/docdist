/**
 * Tauri API Mock 基础设施
 * 在 vitest 环境下 mock @tauri-apps/api 的 invoke 和 listen
 */
import { vi } from 'vitest'

/** Mock invoke 函数，可在测试中通过 mockResolvedValueOnce / mockRejectedValueOnce 控制行为 */
export const mockInvoke = vi.fn()

/** Mock listen 函数，返回一个 resolved 的 unlisten 函数 */
export const mockListen = vi.fn().mockImplementation((_event: string, _handler: (event: any) => void) => {
  const unlistenFn = vi.fn()
  return Promise.resolve(unlistenFn)
})

/** Mock emit 函数 */
export const mockEmit = vi.fn()

vi.mock('@tauri-apps/api/tauri', () => ({
  invoke: (...args: any[]) => mockInvoke(...args),
}))

vi.mock('@tauri-apps/api/event', () => ({
  listen: (...args: any[]) => mockListen(...args),
  emit: (...args: any[]) => mockEmit(...args),
}))
