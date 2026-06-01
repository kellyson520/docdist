import { useState, useEffect, useRef, useCallback } from 'react';
import { Star } from 'lucide-react';

interface StarDialogProps {
  open: boolean;
  defaultLabel?: string;
  onConfirm: (label: string) => void;
  onCancel: () => void;
}

export function StarDialog({ open, defaultLabel = '', onConfirm, onCancel }: StarDialogProps) {
  const [label, setLabel] = useState(defaultLabel);
  const dialogRef = useRef<HTMLDivElement>(null);
  const inputRef = useRef<HTMLInputElement>(null);
  const confirmBtnRef = useRef<HTMLButtonElement>(null);

  const onCancelRef = useRef(onCancel);
  onCancelRef.current = onCancel;

  // Reset label when dialog opens
  useEffect(() => {
    if (open) {
      setLabel(defaultLabel);
      // Focus input on open
      const timer = setTimeout(() => inputRef.current?.focus(), 50);
      return () => clearTimeout(timer);
    }
  }, [open, defaultLabel]);

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape' && open) {
      onCancelRef.current();
    }
    if (e.key === 'Tab' && open && dialogRef.current) {
      const focusable = Array.from(
        dialogRef.current.querySelectorAll<HTMLElement>(
          'button, [href], input, select, textarea, [tabindex]:not([tabindex="-1"])',
        ),
      );
      if (focusable.length === 0) return;
      const first = focusable[0];
      const last = focusable[focusable.length - 1];
      if (e.shiftKey) {
        if (document.activeElement === first) {
          e.preventDefault();
          last.focus();
        }
      } else {
        if (document.activeElement === last) {
          e.preventDefault();
          first.focus();
        }
      }
    }
  }, [open]);

  useEffect(() => {
    if (!open) return;
    window.addEventListener('keydown', handleKeyDown);
    return () => window.removeEventListener('keydown', handleKeyDown);
  }, [open, handleKeyDown]);

  if (!open) return null;

  return (
    <div
      role="dialog"
      aria-modal="true"
      aria-labelledby="star-dialog-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          onCancel();
        }
      }}
    >
      <div ref={dialogRef} className="bg-white dark:bg-gray-800 rounded-xl shadow-xl p-6 w-[400px] animate-fade-in">
        <div className="flex items-center gap-2 mb-4">
          <Star className="w-5 h-5 text-yellow-500 fill-yellow-500" />
          <h3 id="star-dialog-title" className="text-lg font-semibold dark:text-white">标记重要版本</h3>
        </div>

        <div className="mb-6">
          <label className="block text-sm font-medium text-gray-700 dark:text-gray-300 mb-1">标签名称</label>
          <input
            ref={inputRef}
            type="text"
            value={label}
            onChange={(e) => {
              const val = e.target.value;
              if (val.length <= 100) {
                setLabel(val);
              }
            }}
            onKeyDown={(e) => {
              if (e.key === 'Enter') {
                e.preventDefault();
                const trimmed = label.trim();
                if (trimmed.length > 0) {
                  onConfirm(trimmed);
                }
              }
            }}
            placeholder="例如：发布版本 v1.0（最多100字符）"
            className="w-full px-3 py-2 border border-gray-200 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-100 rounded-lg text-sm focus:outline-none focus:ring-2 focus:ring-primary-500"
          />
          <div className="text-xs text-gray-400 mt-1">{label.length}/100</div>
        </div>

        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
          >
            取消
          </button>
          <button
            ref={confirmBtnRef}
            onClick={() => {
              const trimmed = label.trim();
              if (trimmed.length > 0) {
                onConfirm(trimmed);
              }
            }}
            disabled={label.trim().length === 0}
            className={`px-4 py-2 text-sm rounded-lg transition ${
              label.trim().length === 0
                ? 'bg-gray-300 text-gray-500 cursor-not-allowed'
                : 'bg-primary-500 text-white hover:bg-primary-600'
            }`}
          >
            确认标记
          </button>
        </div>
      </div>
    </div>
  );
}
