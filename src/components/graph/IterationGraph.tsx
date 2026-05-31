import { useEffect, useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useArchiveStore } from '../../stores/archiveStore';
import { formatFileSize } from '../../utils/format';
import { formatSmartTime } from '../../utils/time';
import { GitBranch, FileText, RotateCcw, ChevronDown, ChevronRight, ZoomIn, ZoomOut } from 'lucide-react';
import type { Archive } from '../../types';

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

interface TreeNodeComponentProps {
  node: TreeNode;
  depth?: number;
  onSelect: (archive: Archive) => void;
  onRestore: (id: string) => void;
  expandedNodes: Set<string>;
  toggleNode: (id: string) => void;
  zoom: number;
}

function TreeNodeComponent({ 
  node, 
  depth = 0, 
  onSelect, 
  onRestore, 
  expandedNodes, 
  toggleNode,
  zoom 
}: TreeNodeComponentProps) {
  const hasChildren = node.children.length > 0;
  const isExpanded = expandedNodes.has(node.archive.id);

  return (
    <div style={{ marginLeft: depth > 0 ? 24 * zoom : 0 }}>
      <div className="flex items-start gap-2 mb-2 animate-slide-in">
        {/* Tree lines */}
        <div className="flex flex-col items-center">
          <div
            className={`w-6 h-6 rounded-full flex items-center justify-center flex-shrink-0 cursor-pointer transition-all
              ${hasChildren ? 'bg-primary-100 dark:bg-primary-900/30 text-primary-600 dark:text-primary-400 hover:bg-primary-200 dark:hover:bg-primary-800/40' : 'bg-gray-100 dark:bg-gray-700 text-gray-400 dark:text-gray-500'}`}
            onClick={() => hasChildren && toggleNode(node.archive.id)}
          >
            {hasChildren ? (
              isExpanded ? (
                <ChevronDown className="w-3 h-3" />
              ) : (
                <ChevronRight className="w-3 h-3" />
              )
            ) : (
              <FileText className="w-3 h-3" />
            )}
          </div>
          {hasChildren && isExpanded && (
            <div className="w-0.5 h-full bg-gray-200 dark:bg-gray-700 mt-1" />
          )}
        </div>

        {/* Node content */}
        <div
          className="flex-1 p-3 bg-white dark:bg-gray-800 rounded-lg border border-gray-200 dark:border-gray-700 hover:border-primary-300 dark:hover:border-primary-600 cursor-pointer transition group"
          onClick={() => onSelect(node.archive)}
        >
          <div className="flex items-center justify-between">
            <span className="text-sm font-medium truncate dark:text-gray-200">{node.archive.file_name}</span>
            <span className="text-xs text-gray-400 dark:text-gray-500">{formatSmartTime(node.archive.created_at)}</span>
          </div>
          <div className="flex items-center gap-2 mt-1 text-xs text-gray-500 dark:text-gray-400">
            <span>{formatFileSize(node.archive.file_size)}</span>
            {node.archive.note && (
              <>
                <span>·</span>
                <span className="truncate">{node.archive.note}</span>
              </>
            )}
          </div>
          <div className="mt-2 flex gap-2 opacity-0 group-hover:opacity-100 transition-opacity">
            <button
              onClick={(e) => { e.stopPropagation(); onRestore(node.archive.id); }}
              className="flex items-center gap-1 text-xs text-primary-600 dark:text-primary-400 hover:underline"
            >
              <RotateCcw className="w-3 h-3" />
              恢复
            </button>
            {hasChildren && (
              <span className="text-xs text-gray-400 dark:text-gray-500">
                {node.children.length} 个子版本
              </span>
            )}
          </div>
        </div>
      </div>

      {hasChildren && isExpanded && (
        <div className="ml-3">
          {node.children.map((child) => (
            <TreeNodeComponent 
              key={child.archive.id} 
              node={child} 
              depth={depth + 1}
              onSelect={onSelect}
              onRestore={onRestore}
              expandedNodes={expandedNodes}
              toggleNode={toggleNode}
              zoom={zoom}
            />
          ))}
        </div>
      )}
    </div>
  );
}

export function IterationGraph() {
  const { selectArchive, restoreArchive } = useArchiveStore();
  const [tree, setTree] = useState<TreeNode[]>([]);
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());
  const [zoom, setZoom] = useState(1);

  const loadTree = useCallback(async () => {
    try {
      const archives = await invoke<Archive[]>('get_archive_tree');
      const builtTree = buildTree(archives);
      setTree(builtTree);

      // Auto-expand root nodes
      const rootIds = new Set(builtTree.map(n => n.archive.id));
      setExpandedNodes(rootIds);
    } catch (err) {
      console.error('Failed to load archive tree:', err);
    }
  }, []);

  useEffect(() => {
    loadTree();
  }, [loadTree]);

  const toggleNode = useCallback((id: string) => {
    setExpandedNodes(prev => {
      const next = new Set(prev);
      if (next.has(id)) {
        next.delete(id);
      } else {
        next.add(id);
      }
      return next;
    });
  }, []);

  const expandAll = useCallback(() => {
    const allIds = new Set<string>();
    const collectIds = (nodes: TreeNode[]) => {
      for (const node of nodes) {
        if (node.children.length > 0) {
          allIds.add(node.archive.id);
          collectIds(node.children);
        }
      }
    };
    collectIds(tree);
    setExpandedNodes(allIds);
  }, [tree]);

  const collapseAll = useCallback(() => {
    setExpandedNodes(new Set());
  }, []);

  const handleZoomIn = useCallback(() => {
    setZoom(prev => Math.min(Math.round((prev + 0.1) * 10) / 10, 1.5));
  }, []);

  const handleZoomOut = useCallback(() => {
    setZoom(prev => Math.max(Math.round((prev - 0.1) * 10) / 10, 0.5));
  }, []);

  return (
    <div className="flex flex-col h-full">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b border-gray-100 dark:border-gray-700">
        <div className="flex items-center gap-2">
          <GitBranch className="w-5 h-5 text-primary-500" />
          <h2 className="font-semibold text-lg dark:text-white">迭代关系图</h2>
          <span className="text-xs text-gray-400 dark:text-gray-500 bg-gray-100 dark:bg-gray-700 px-2 py-0.5 rounded-full">
            {tree.length} 个根节点
          </span>
        </div>

        <div className="flex items-center gap-2">
          {/* Zoom Controls */}
          <div className="flex items-center gap-1 bg-gray-100 dark:bg-gray-700 rounded-lg p-0.5">
            <button
              onClick={handleZoomOut}
              className="p-1 hover:bg-white dark:hover:bg-gray-600 rounded transition"
              title="缩小"
            >
              <ZoomOut className="w-3.5 h-3.5 text-gray-600 dark:text-gray-400" />
            </button>
            <span className="text-xs text-gray-600 dark:text-gray-400 px-1">{Math.round(zoom * 100)}%</span>
            <button
              onClick={handleZoomIn}
              className="p-1 hover:bg-white dark:hover:bg-gray-600 rounded transition"
              title="放大"
            >
              <ZoomIn className="w-3.5 h-3.5 text-gray-600 dark:text-gray-400" />
            </button>
          </div>

          {/* Expand/Collapse */}
          <button
            onClick={expandAll}
            className="px-2 py-1.5 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
          >
            展开全部
          </button>
          <button
            onClick={collapseAll}
            className="px-2 py-1.5 text-xs text-gray-600 dark:text-gray-400 hover:bg-gray-100 dark:hover:bg-gray-700 rounded transition"
          >
            折叠全部
          </button>
        </div>
      </div>

      {/* Graph */}
      <div className="flex-1 overflow-auto p-4">
        {tree.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-64 text-gray-400 dark:text-gray-500">
            <GitBranch className="w-12 h-12 mb-3 opacity-30" />
            <p className="text-sm">暂无存档关系</p>
            <p className="text-xs mt-1">创建存档时可以指定父存档，形成迭代关系</p>
          </div>
        ) : (
          <div className="space-y-2" style={{ transform: `scale(${zoom})`, transformOrigin: 'top left' }}>
            {tree.map((node) => (
              <TreeNodeComponent 
                key={node.archive.id} 
                node={node}
                onSelect={selectArchive}
                onRestore={restoreArchive}
                expandedNodes={expandedNodes}
                toggleNode={toggleNode}
                zoom={zoom}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
