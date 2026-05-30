import { useEffect, useState } from 'react';
import { useArchiveStore } from '../../stores/archiveStore';
import { formatFileSize, formatDate } from '../../utils/format';
import { GitBranch, FileText } from 'lucide-react';

interface TreeNode {
  archive: Archive;
  children: TreeNode[];
}

function buildTree(archives: Archive[]): TreeNode[] {
  const map = new Map<string, TreeNode>();
  const roots: TreeNode[] = [];

  for (const archive of archives) {
    map.set(archive.id, { archive, children: [] });
  }

  for (const archive of archives) {
    const node = map.get(archive.id)!;
    if (archive.parent_id && map.has(archive.parent_id)) {
      map.get(archive.parent_id)!.children.push(node);
    } else {
      roots.push(node);
    }
  }

  return roots;
}

function TreeNodeComponent({ node, depth = 0 }: { node: TreeNode; depth?: number }) {
  const { selectArchive, restoreArchive } = useArchiveStore();
  const [expanded, setExpanded] = useState(true);
  const hasChildren = node.children.length > 0;

  return (
    <div style={{ marginLeft: depth > 0 ? 24 : 0 }}>
      <div className="flex items-start gap-2 mb-2 animate-slide-in">
        {/* Tree lines */}
        <div className="flex flex-col items-center">
          <div
            className={`w-6 h-6 rounded-full flex items-center justify-center flex-shrink-0 cursor-pointer
              ${hasChildren ? 'bg-primary-100 text-primary-600' : 'bg-gray-100 text-gray-400'}`}
            onClick={() => hasChildren && setExpanded(!expanded)}
          >
            {hasChildren ? (
              <span className="text-xs font-bold">{node.children.length}</span>
            ) : (
              <FileText className="w-3 h-3" />
            )}
          </div>
          {hasChildren && expanded && (
            <div className="w-0.5 h-full bg-gray-200 mt-1" />
          )}
        </div>

        {/* Node content */}
        <div
          className="flex-1 p-3 bg-white rounded-lg border border-gray-200 hover:border-primary-300 cursor-pointer transition"
          onClick={() => selectArchive(node.archive)}
        >
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium">{node.archive.file_name}</span>
            <span className="text-xs text-gray-400">{formatDate(node.archive.created_at)}</span>
          </div>
          <div className="flex items-center gap-2 mt-1 text-xs text-gray-500">
            <span>{formatFileSize(node.archive.file_size)}</span>
            {node.archive.note && (
              <>
                <span>·</span>
                <span className="truncate">{node.archive.note}</span>
              </>
            )}
          </div>
          <div className="mt-2 flex gap-2">
            <button
              onClick={(e) => { e.stopPropagation(); restoreArchive(node.archive.id); }}
              className="text-xs text-primary-600 hover:underline"
            >
              恢复
            </button>
          </div>
        </div>
      </div>

      {hasChildren && expanded && (
        <div className="ml-3">
          {node.children.map((child) => (
            <TreeNodeComponent key={child.archive.id} node={child} depth={depth + 1} />
          ))}
        </div>
      )}
    </div>
  );
}

export function IterationGraph() {
  const { archives, fetchArchives } = useArchiveStore();
  const [tree, setTree] = useState<TreeNode[]>([]);

  useEffect(() => {
    fetchArchives();
  }, [fetchArchives]);

  useEffect(() => {
    setTree(buildTree(archives));
  }, [archives]);

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center gap-2 p-4 border-b border-gray-100">
        <GitBranch className="w-5 h-5 text-primary-500" />
        <h2 className="font-semibold text-lg">迭代关系图</h2>
      </div>

      <div className="flex-1 overflow-y-auto p-4">
        {tree.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-gray-400">
            <GitBranch className="w-12 h-12 mb-3 opacity-30" />
            <p className="text-sm">暂无存档关系</p>
          </div>
        ) : (
          <div className="space-y-2">
            {tree.map((node) => (
              <TreeNodeComponent key={node.archive.id} node={node} />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
