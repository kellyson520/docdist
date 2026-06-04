import { useEffect, useState, useCallback, useMemo } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import { useArchiveStore } from '../../stores/archiveStore';
import { shallow } from 'zustand/shallow';
import { formatFileSize } from '../../utils/format';
import { formatSmartTime } from '../../utils/time';
import {
  GitBranch,
  FileText,
  RotateCcw,
  ChevronDown,
  ChevronRight,
  ZoomIn,
  ZoomOut,
  Search,
  Layers,
  GitCommit,
  Tag,
} from 'lucide-react';
import type { Archive } from '../../types';

interface TreeNode {
  archive: Archive;
  children: TreeNode[];
}

interface GraphStats {
  total: number;
  roots: number;
  branches: number;
  maxDepth: number;
  tagged: number;
}

function compareArchiveOrder(a: Archive, b: Archive): number {
  const byDate = a.created_at.localeCompare(b.created_at);
  return byDate === 0 ? a.id.localeCompare(b.id) : byDate;
}

function buildTree(archives: Archive[]): TreeNode[] {
  const sorted = [...archives].sort(compareArchiveOrder);
  const idSet = new Set(sorted.map((archive) => archive.id));
  const attached = new Set<string>();
  const childrenByParent = new Map<string, Archive[]>();

  for (const archive of sorted) {
    if (archive.parent_id && archive.parent_id !== archive.id && idSet.has(archive.parent_id)) {
      const children = childrenByParent.get(archive.parent_id) ?? [];
      children.push(archive);
      childrenByParent.set(archive.parent_id, children);
      attached.add(archive.id);
    }
  }

  const visited = new Set<string>();
  const materialize = (archive: Archive, path: Set<string>): TreeNode => {
    visited.add(archive.id);
    const nextPath = new Set(path);
    nextPath.add(archive.id);

    const children = (childrenByParent.get(archive.id) ?? [])
      .filter((child) => !nextPath.has(child.id))
      .map((child) => materialize(child, nextPath));

    return { archive, children };
  };

  const rootArchives = sorted.filter((archive) => !attached.has(archive.id));
  const roots = (rootArchives.length > 0 ? rootArchives : sorted.slice(0, 1))
    .map((archive) => materialize(archive, new Set()));

  for (const archive of sorted) {
    if (!visited.has(archive.id)) {
      roots.push(materialize(archive, new Set()));
    }
  }

  return roots;
}

function matchesArchive(archive: Archive, query: string): boolean {
  const value = query.trim().toLowerCase();
  if (!value) return true;

  return [
    archive.file_name,
    archive.file_path,
    archive.note,
    archive.created_at,
    ...(archive.tags ?? []),
  ]
    .filter(Boolean)
    .some((field) => field.toLowerCase().includes(value));
}

function filterTree(nodes: TreeNode[], query: string): TreeNode[] {
  if (!query.trim()) return nodes;

  return nodes.flatMap((node) => {
    const children = filterTree(node.children, query);
    if (matchesArchive(node.archive, query) || children.length > 0) {
      return [{ ...node, children }];
    }
    return [];
  });
}

function collectExpandableIds(nodes: TreeNode[]): Set<string> {
  const ids = new Set<string>();
  const visit = (node: TreeNode) => {
    if (node.children.length > 0) {
      ids.add(node.archive.id);
      node.children.forEach(visit);
    }
  };
  nodes.forEach(visit);
  return ids;
}

function calculateStats(nodes: TreeNode[]): GraphStats {
  const stats: GraphStats = {
    total: 0,
    roots: nodes.length,
    branches: 0,
    maxDepth: 0,
    tagged: 0,
  };

  const visit = (node: TreeNode, depth: number) => {
    stats.total += 1;
    stats.maxDepth = Math.max(stats.maxDepth, depth + 1);
    if (node.children.length > 0) stats.branches += 1;
    if ((node.archive.tags ?? []).length > 0) stats.tagged += 1;
    node.children.forEach((child) => visit(child, depth + 1));
  };

  nodes.forEach((node) => visit(node, 0));
  return stats;
}

interface TreeNodeComponentProps {
  node: TreeNode;
  depth?: number;
  index: number;
  siblingCount: number;
  onSelect: (archive: Archive) => void;
  onRestore: (id: string) => void;
  expandedNodes: Set<string>;
  toggleNode: (id: string) => void;
  latestId?: string;
  selectedId?: string;
}

function TreeNodeComponent({
  node,
  depth = 0,
  index,
  siblingCount,
  onSelect,
  onRestore,
  expandedNodes,
  toggleNode,
  latestId,
  selectedId,
}: TreeNodeComponentProps) {
  const hasChildren = node.children.length > 0;
  const isExpanded = expandedNodes.has(node.archive.id);
  const isLatest = node.archive.id === latestId;
  const isSelected = node.archive.id === selectedId;
  const isLastSibling = index === siblingCount - 1;

  return (
    <div className="relative">
      {depth > 0 && (
        <>
          <div
            className={`absolute left-0 top-0 w-px bg-gray-200 dark:bg-gray-700 ${
              isLastSibling ? 'h-6' : 'h-full'
            }`}
          />
          <div className="absolute left-0 top-6 h-px w-6 bg-gray-200 dark:bg-gray-700" />
        </>
      )}

      <div className={depth > 0 ? 'pl-6' : ''}>
        <div className="flex items-start gap-3 pb-3">
          <button
            onClick={() => hasChildren && toggleNode(node.archive.id)}
            aria-label={hasChildren ? (isExpanded ? '折叠分支' : '展开分支') : '版本节点'}
            className={`mt-3 flex h-7 w-7 flex-shrink-0 items-center justify-center rounded-lg border transition ${
              hasChildren
                ? 'border-primary-200 bg-primary-50 text-primary-600 hover:bg-primary-100 dark:border-primary-800 dark:bg-primary-900/30 dark:text-primary-300'
                : 'border-gray-200 bg-white text-gray-400 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-500'
            }`}
          >
            {hasChildren ? (
              isExpanded ? <ChevronDown className="h-3.5 w-3.5" /> : <ChevronRight className="h-3.5 w-3.5" />
            ) : (
              <GitCommit className="h-3.5 w-3.5" />
            )}
          </button>

          <div
            onClick={() => onSelect(node.archive)}
            className={`group min-w-[280px] max-w-2xl flex-1 cursor-pointer rounded-lg border bg-white p-3 shadow-sm transition dark:bg-gray-800 ${
              isSelected
                ? 'border-primary-400 ring-2 ring-primary-100 dark:border-primary-500 dark:ring-primary-900/40'
                : 'border-gray-200 hover:border-primary-300 hover:shadow-md dark:border-gray-700 dark:hover:border-primary-600'
            }`}
          >
            <div className="flex items-start justify-between gap-3">
              <div className="min-w-0">
                <div className="flex min-w-0 items-center gap-2">
                  <FileText className="h-4 w-4 flex-shrink-0 text-gray-400 dark:text-gray-500" />
                  <span className="truncate text-sm font-semibold text-gray-800 dark:text-gray-100">
                    {node.archive.file_name}
                  </span>
                  {isLatest && (
                    <span className="rounded-full bg-accent-50 px-2 py-0.5 text-[11px] font-medium text-accent-600 dark:bg-accent-600/15 dark:text-accent-300">
                      最新
                    </span>
                  )}
                </div>
                <div className="mt-1 flex flex-wrap items-center gap-x-2 gap-y-1 text-xs text-gray-500 dark:text-gray-400">
                  <span>{formatSmartTime(node.archive.created_at)}</span>
                  <span>{formatFileSize(node.archive.file_size)}</span>
                  {hasChildren && <span>{node.children.length} 个后续版本</span>}
                </div>
              </div>

              <button
                onClick={(event) => {
                  event.stopPropagation();
                  onRestore(node.archive.id);
                }}
                className="flex h-8 w-8 flex-shrink-0 items-center justify-center rounded-lg text-gray-400 opacity-0 transition hover:bg-primary-50 hover:text-primary-600 group-hover:opacity-100 dark:hover:bg-primary-900/30 dark:hover:text-primary-300"
                title="恢复此版本"
                aria-label="恢复此版本"
              >
                <RotateCcw className="h-4 w-4" />
              </button>
            </div>

            {(node.archive.note || (node.archive.tags ?? []).length > 0) && (
              <div className="mt-3 flex flex-wrap items-center gap-2 border-t border-gray-100 pt-2 text-xs dark:border-gray-700">
                {node.archive.note && (
                  <span className="max-w-full truncate text-gray-600 dark:text-gray-300">
                    {node.archive.note}
                  </span>
                )}
                {(node.archive.tags ?? []).slice(0, 4).map((tag) => (
                  <span
                    key={tag}
                    className="inline-flex items-center gap-1 rounded-md bg-gray-100 px-1.5 py-0.5 text-gray-600 dark:bg-gray-700 dark:text-gray-300"
                  >
                    <Tag className="h-3 w-3" />
                    {tag}
                  </span>
                ))}
              </div>
            )}
          </div>
        </div>

        {hasChildren && isExpanded && (
          <div className="ml-3">
            {node.children.map((child, childIndex) => (
              <TreeNodeComponent
                key={child.archive.id}
                node={child}
                depth={depth + 1}
                index={childIndex}
                siblingCount={node.children.length}
                onSelect={onSelect}
                onRestore={onRestore}
                expandedNodes={expandedNodes}
                toggleNode={toggleNode}
                latestId={latestId}
                selectedId={selectedId}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}

export function IterationGraph() {
  const { selectArchive, restoreArchive, selectedArchive, treeRevision } = useArchiveStore(
    (s) => ({
      selectArchive: s.selectArchive,
      restoreArchive: s.restoreArchive,
      selectedArchive: s.selectedArchive,
      treeRevision: s.treeRevision,
    }),
    shallow,
  );
  const [tree, setTree] = useState<TreeNode[]>([]);
  const [expandedNodes, setExpandedNodes] = useState<Set<string>>(new Set());
  const [zoom, setZoom] = useState(1);
  const [query, setQuery] = useState('');
  const [loading, setLoading] = useState(false);

  const loadTree = useCallback(async () => {
    setLoading(true);
    try {
      const archives = await invoke<Archive[]>('get_archive_tree');
      const builtTree = buildTree(archives);
      setTree(builtTree);
      setExpandedNodes(collectExpandableIds(builtTree));
    } catch (err) {
      console.error('Failed to load archive tree:', err);
    } finally {
      setLoading(false);
    }
  }, []);

  useEffect(() => {
    loadTree();
  }, [loadTree, treeRevision]);

  const visibleTree = useMemo(() => filterTree(tree, query), [tree, query]);
  const stats = useMemo(() => calculateStats(tree), [tree]);
  const visibleStats = useMemo(() => calculateStats(visibleTree), [visibleTree]);
  const latestId = useMemo(() => {
    const allArchives: Archive[] = [];
    const collect = (nodes: TreeNode[]) => {
      for (const node of nodes) {
        allArchives.push(node.archive);
        collect(node.children);
      }
    };
    collect(tree);
    return allArchives.sort((a, b) => b.created_at.localeCompare(a.created_at))[0]?.id;
  }, [tree]);

  useEffect(() => {
    if (query.trim()) {
      setExpandedNodes(collectExpandableIds(visibleTree));
    }
  }, [query, visibleTree]);

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
    setExpandedNodes(collectExpandableIds(visibleTree));
  }, [visibleTree]);

  const collapseAll = useCallback(() => {
    setExpandedNodes(new Set());
  }, []);

  const handleZoomIn = useCallback(() => {
    setZoom(prev => Math.min(Math.round((prev + 0.1) * 10) / 10, 1.5));
  }, []);

  const handleZoomOut = useCallback(() => {
    setZoom(prev => Math.max(Math.round((prev - 0.1) * 10) / 10, 0.7));
  }, []);

  const statItems = [
    { label: '版本', value: stats.total, icon: GitCommit },
    { label: '根节点', value: stats.roots, icon: GitBranch },
    { label: '分支', value: stats.branches, icon: Layers },
    { label: '深度', value: stats.maxDepth, icon: GitBranch },
  ];

  return (
    <div className="flex h-full flex-col overflow-hidden rounded-lg border border-gray-200 bg-white shadow-sm dark:border-gray-700 dark:bg-gray-800">
      <div className="border-b border-gray-100 bg-gray-50/80 px-4 py-3 dark:border-gray-700 dark:bg-gray-900/30">
        <div className="flex flex-col gap-3 xl:flex-row xl:items-center xl:justify-between">
          <div className="flex min-w-0 items-center gap-3">
            <div className="flex h-10 w-10 flex-shrink-0 items-center justify-center rounded-lg bg-primary-50 text-primary-600 dark:bg-primary-900/30 dark:text-primary-300">
              <GitBranch className="h-5 w-5" />
            </div>
            <div className="min-w-0">
              <h2 className="text-base font-semibold text-gray-900 dark:text-white">迭代图谱</h2>
              <p className="truncate text-xs text-gray-500 dark:text-gray-400">
                按父子版本关系展示分支路径和关键节点
              </p>
            </div>
          </div>

          <div className="flex flex-wrap items-center gap-2">
            <div className="relative min-w-[220px] flex-1 sm:flex-none">
              <Search className="absolute left-2.5 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400" />
              <input
                value={query}
                onChange={(event) => setQuery(event.target.value)}
                placeholder="搜索备注、标签、文件名"
                className="h-9 w-full rounded-lg border border-gray-200 bg-white pl-8 pr-3 text-sm text-gray-700 outline-none transition focus:border-primary-300 focus:ring-2 focus:ring-primary-100 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-200 dark:focus:border-primary-600 dark:focus:ring-primary-900/40"
              />
            </div>

            <div className="flex items-center gap-1 rounded-lg border border-gray-200 bg-white p-0.5 dark:border-gray-700 dark:bg-gray-800">
              <button
                onClick={handleZoomOut}
                className="flex h-8 w-8 items-center justify-center rounded-md text-gray-500 transition hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700"
                title="缩小"
                aria-label="缩小"
              >
                <ZoomOut className="h-4 w-4" />
              </button>
              <span className="w-10 text-center text-xs text-gray-500 dark:text-gray-400">{Math.round(zoom * 100)}%</span>
              <button
                onClick={handleZoomIn}
                className="flex h-8 w-8 items-center justify-center rounded-md text-gray-500 transition hover:bg-gray-100 dark:text-gray-400 dark:hover:bg-gray-700"
                title="放大"
                aria-label="放大"
              >
                <ZoomIn className="h-4 w-4" />
              </button>
            </div>

            <button
              onClick={expandAll}
              className="h-9 rounded-lg border border-gray-200 bg-white px-3 text-sm text-gray-600 transition hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
            >
              展开
            </button>
            <button
              onClick={collapseAll}
              className="h-9 rounded-lg border border-gray-200 bg-white px-3 text-sm text-gray-600 transition hover:bg-gray-50 dark:border-gray-700 dark:bg-gray-800 dark:text-gray-300 dark:hover:bg-gray-700"
            >
              折叠
            </button>
          </div>
        </div>

        <div className="mt-3 grid grid-cols-2 gap-2 md:grid-cols-4">
          {statItems.map(({ label, value, icon: Icon }) => (
            <div
              key={label}
              className="flex items-center gap-2 rounded-lg border border-gray-200 bg-white px-3 py-2 dark:border-gray-700 dark:bg-gray-800"
            >
              <Icon className="h-4 w-4 text-gray-400" />
              <span className="text-xs text-gray-500 dark:text-gray-400">{label}</span>
              <span className="ml-auto text-sm font-semibold text-gray-900 dark:text-gray-100">{value}</span>
            </div>
          ))}
        </div>
      </div>

      <div className="flex-1 overflow-auto bg-[linear-gradient(#f3f4f6_1px,transparent_1px),linear-gradient(90deg,#f3f4f6_1px,transparent_1px)] bg-[size:28px_28px] p-4 dark:bg-[linear-gradient(#243041_1px,transparent_1px),linear-gradient(90deg,#243041_1px,transparent_1px)]">
        {loading ? (
          <div className="flex h-64 items-center justify-center text-sm text-gray-400 dark:text-gray-500">
            正在加载迭代关系
          </div>
        ) : visibleTree.length === 0 ? (
          <div className="flex h-64 flex-col items-center justify-center text-gray-400 dark:text-gray-500">
            <GitBranch className="mb-3 h-12 w-12 opacity-30" />
            <p className="text-sm">{query.trim() ? '没有匹配的版本节点' : '暂无存档关系'}</p>
            <p className="mt-1 text-xs">创建存档后会自动形成可追踪的迭代路径</p>
          </div>
        ) : (
          <div
            className="inline-block min-w-full rounded-lg bg-white/70 p-3 shadow-sm backdrop-blur-sm dark:bg-gray-900/50"
            style={{ transform: `scale(${zoom})`, transformOrigin: 'top left' }}
          >
            {query.trim() && (
              <div className="mb-3 rounded-lg border border-primary-100 bg-primary-50 px-3 py-2 text-xs text-primary-700 dark:border-primary-800 dark:bg-primary-900/30 dark:text-primary-300">
                当前显示 {visibleStats.total} 个匹配节点，已自动展开相关路径。
              </div>
            )}
            {visibleTree.map((node, index) => (
              <TreeNodeComponent
                key={node.archive.id}
                node={node}
                index={index}
                siblingCount={visibleTree.length}
                onSelect={selectArchive}
                onRestore={restoreArchive}
                expandedNodes={expandedNodes}
                toggleNode={toggleNode}
                latestId={latestId}
                selectedId={selectedArchive?.id}
              />
            ))}
          </div>
        )}
      </div>
    </div>
  );
}
