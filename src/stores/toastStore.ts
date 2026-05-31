/**
 * Toast 通知系统
 * 轻量级、支持多条堆叠、自动消失
 */
import { create } from 'zustand';
import { createLogger } from '../utils/logger';

const log = createLogger('toast');

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export interface Toast {
  id: string;
  type: ToastType;
  title: string;
  message?: string;
  duration: number;
  dismissible: boolean;
}

interface ToastState {
  toasts: Toast[];
  addToast: (toast: Omit<Toast, 'id'>) => void;
  removeToast: (id: string) => void;
  clearAll: () => void;
}

export const useToastStore = create<ToastState>((set) => ({
  toasts: [],

  addToast: (toast) => {
    const id = `toast-${Date.now()}-${Math.random().toString(36).slice(2, 6)}`;
    const newToast: Toast = { id, ...toast };
    if (!newToast.duration) newToast.duration = 3000;
    if (newToast.dismissible === undefined) newToast.dismissible = true;

    log.debug(`Toast: [${toast.type}] ${toast.title}`);
    set((state) => ({
      toasts: [...state.toasts, newToast].slice(-5), // 最多5条
    }));

    // 自动消失
    if (newToast.duration > 0) {
      setTimeout(() => {
        set((state) => ({
          toasts: state.toasts.filter(t => t.id !== id),
        }));
      }, newToast.duration);
    }
  },

  removeToast: (id) => {
    set((state) => ({
      toasts: state.toasts.filter(t => t.id !== id),
    }));
  },

  clearAll: () => set({ toasts: [] }),
}));

/** 便捷方法 */
export const toast = {
  success: (title: string, message?: string) =>
    useToastStore.getState().addToast({ type: 'success', title, message, duration: 3000, dismissible: true }),
  error: (title: string, message?: string) =>
    useToastStore.getState().addToast({ type: 'error', title, message, duration: 5000, dismissible: true }),
  warning: (title: string, message?: string) =>
    useToastStore.getState().addToast({ type: 'warning', title, message, duration: 4000, dismissible: true }),
  info: (title: string, message?: string) =>
    useToastStore.getState().addToast({ type: 'info', title, message, duration: 3000, dismissible: true }),
};
