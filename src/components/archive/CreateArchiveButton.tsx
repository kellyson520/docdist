import { useState } from "react";
import { Plus, FileUp, X } from "lucide-react";
import { open } from "@tauri-apps/api/dialog";
import { useArchiveStore } from "../../stores/archiveStore";
import { TagBadge } from "../common/TagBadge";

interface CreateArchiveButtonProps {
  className?: string;
}

export default function CreateArchiveButton({ className }: CreateArchiveButtonProps) {
  const [dialogOpen, setDialogOpen] = useState(false);
  const [selectedPath, setSelectedPath] = useState("");
  const [note, setNote] = useState("");
  const [tags, setTags] = useState<string[]>([]);
  const [tagInput, setTagInput] = useState("");
  const { createArchive, loading } = useArchiveStore();

  const handleSelectFile = async () => {
    const path = await open({
      multiple: false,
      title: "选择要存档的文件",
    });
    if (path && typeof path === "string") {
      setSelectedPath(path);
    }
  };

  const handleAddTag = () => {
    const trimmed = tagInput.trim();
    if (trimmed && !tags.includes(trimmed)) {
      setTags([...tags, trimmed]);
      setTagInput("");
    }
  };

  const handleRemoveTag = (tag: string) => {
    setTags(tags.filter((t) => t !== tag));
  };

  const handleSubmit = async () => {
    if (!selectedPath) return;
    await createArchive(selectedPath, note, tags);
    setDialogOpen(false);
    setSelectedPath("");
    setNote("");
    setTags([]);
  };

  return (
    <>
      <button onClick={() => setDialogOpen(true)} className={`btn-primary ${className}`}>
        <Plus className="h-4 w-4" />
        新建存档
      </button>

      {dialogOpen && (
        <div className="fixed inset-0 z-50 flex items-center justify-center">
          <div className="fixed inset-0 bg-black/30" onClick={() => setDialogOpen(false)} />
          <div className="relative w-full max-w-lg rounded-xl bg-white p-6 shadow-xl">
            <button
              onClick={() => setDialogOpen(false)}
              className="absolute right-4 top-4 text-gray-400 hover:text-gray-600"
            >
              <X className="h-5 w-5" />
            </button>

            <h2 className="text-lg font-semibold text-gray-900 mb-4">新建存档</h2>

            <div className="space-y-4">
              {/* File Selector */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">选择文件</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={selectedPath}
                    readOnly
                    placeholder="点击右侧按钮选择文件..."
                    className="flex-1 rounded-lg border border-gray-300 bg-gray-50 px-3 py-2 text-sm"
                  />
                  <button onClick={handleSelectFile} className="btn-secondary">
                    <FileUp className="h-4 w-4" />
                    浏览
                  </button>
                </div>
              </div>

              {/* Note */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">备注</label>
                <textarea
                  value={note}
                  onChange={(e) => setNote(e.target.value)}
                  placeholder="为此次存档添加备注..."
                  rows={3}
                  className="w-full rounded-lg border border-gray-300 px-3 py-2 text-sm placeholder-gray-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
                />
              </div>

              {/* Tags */}
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">标签</label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={tagInput}
                    onChange={(e) => setTagInput(e.target.value)}
                    onKeyDown={(e) => {
                      if (e.key === "Enter") {
                        e.preventDefault();
                        handleAddTag();
                      }
                    }}
                    placeholder="输入标签后按回车添加..."
                    className="flex-1 rounded-lg border border-gray-300 px-3 py-2 text-sm placeholder-gray-400 focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
                  />
                  <button onClick={handleAddTag} className="btn-secondary">
                    添加
                  </button>
                </div>
                {tags.length > 0 && (
                  <div className="mt-2 flex flex-wrap gap-1.5">
                    {tags.map((tag) => (
                      <TagBadge key={tag} tag={tag} onRemove={() => handleRemoveTag(tag)} />
                    ))}
                  </div>
                )}
              </div>
            </div>

            <div className="mt-6 flex justify-end gap-3">
              <button onClick={() => setDialogOpen(false)} className="btn-secondary">
                取消
              </button>
              <button
                onClick={handleSubmit}
                disabled={!selectedPath || loading}
                className="btn-primary"
              >
                {loading ? "存档中..." : "创建存档"}
              </button>
            </div>
          </div>
        </div>
      )}
    </>
  );
}
