import { useEffect, useRef, useCallback } from 'react';

interface ConfirmDialogProps {
  open: boolean;
  title: string;
  message: string;
  onConfirm: () => void;
  onCancel: () => void;
}

export function ConfirmDialog({ open, title, message, onConfirm, onCancel }: ConfirmDialogProps) {
  const dialogRef = useRef<HTMLDivElement>(null);

  const onCancelRef = useRef(onCancel);
  onCancelRef.current = onCancel;

  const handleKeyDown = useCallback((e: KeyboardEvent) => {
    if (e.key === 'Escape' && open) {
      onCancelRef.current();
    }
  }, [open]);

  // Focus the confirm button on open for keyboard users
  const confirmBtnRef = useRef<HTMLButtonElement>(null);
  useEffect(() => {
    if (open) {
      confirmBtnRef.current?.focus();
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
      aria-labelledby="confirm-dialog-title"
      className="fixed inset-0 z-50 flex items-center justify-center bg-black/30"
      onClick={(e) => {
        if (e.target === e.currentTarget) {
          onCancel();
        }
      }}
    >
      <div ref={dialogRef} className="bg-white dark:bg-gray-800 rounded-xl shadow-xl p-6 w-96 animate-fade-in">
        <h3 id="confirm-dialog-title" className="text-lg font-semibold mb-2 dark:text-white">{title}</h3>
        <p className="text-gray-600 dark:text-gray-300 text-sm mb-6">{message}</p>
        <div className="flex justify-end gap-3">
          <button
            onClick={onCancel}
            className="px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
          >
            取消
          </button>
          <button
            onClick={onConfirm}
            ref={confirmBtnRef}
            className="px-4 py-2 text-sm bg-red-500 text-white rounded-lg hover:bg-red-600 transition"
          >
            确认
          </button>
        </div>
      </div>
    </div>
  );
}
