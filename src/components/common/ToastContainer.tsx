/**
 * Toast 通知容器 — 堆叠式浮窗
 */
import { useToastStore, type Toast, type ToastType } from '../../stores/toastStore';
import { X, CheckCircle, AlertCircle, AlertTriangle, Info } from 'lucide-react';

const iconMap: Record<ToastType, typeof CheckCircle> = {
  success: CheckCircle,
  error: AlertCircle,
  warning: AlertTriangle,
  info: Info,
};

const colorMap: Record<ToastType, { bg: string; border: string; icon: string; text: string }> = {
  success: { bg: 'bg-green-50', border: 'border-green-200', icon: 'text-green-500', text: 'text-green-800' },
  error: { bg: 'bg-red-50', border: 'border-red-200', icon: 'text-red-500', text: 'text-red-800' },
  warning: { bg: 'bg-amber-50', border: 'border-amber-200', icon: 'text-amber-500', text: 'text-amber-800' },
  info: { bg: 'bg-blue-50', border: 'border-blue-200', icon: 'text-blue-500', text: 'text-blue-800' },
};

function ToastItem({ toast }: { toast: Toast }) {
  const { removeToast } = useToastStore();
  const Icon = iconMap[toast.type];
  const colors = colorMap[toast.type];

  return (
    <div
      className={`flex items-start gap-3 ${colors.bg} ${colors.border} border rounded-xl px-4 py-3 shadow-lg backdrop-blur-sm animate-slide-in-right max-w-sm`}
      role="alert"
    >
      <Icon className={`w-5 h-5 ${colors.icon} flex-shrink-0 mt-0.5`} />
      <div className="flex-1 min-w-0">
        <p className={`text-sm font-medium ${colors.text}`}>{toast.title}</p>
        {toast.message && (
          <p className="text-xs text-gray-500 dark:text-gray-400 mt-0.5 line-clamp-2">{toast.message}</p>
        )}
      </div>
      {toast.dismissible && (
        <button
          onClick={() => removeToast(toast.id)}
          className="p-0.5 hover:bg-black/5 rounded transition flex-shrink-0"
        >
          <X className="w-3.5 h-3.5 text-gray-400 dark:text-gray-500" />
        </button>
      )}
    </div>
  );
}

export function ToastContainer() {
  const { toasts } = useToastStore();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed top-4 right-4 z-[9999] flex flex-col gap-2 pointer-events-none">
      {toasts.map((t) => (
        <div key={t.id} className="pointer-events-auto">
          <ToastItem toast={t} />
        </div>
      ))}
    </div>
  );
}
