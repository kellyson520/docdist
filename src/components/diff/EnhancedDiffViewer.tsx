import React, { useState } from 'react';
import { FileText, Copy, Check, FileSearch } from 'lucide-react';
import { useArchiveStore } from '../../stores/archiveStore';
import { DiffDetailView } from './DiffDetailView';
import { SummaryView } from './SummaryView';
import { RegionsView } from './RegionsView';
import type { FileType } from '../../types/diff';

export function EnhancedDiffViewer() {
  const { enhancedDiffResult, loading, clearDiff } = useArchiveStore();
  const [activeTab, setActiveTab] = useState<'diff' | 'summary' | 'regions'>('diff');
  const [copied, setCopied] = useState(false);

  const handleCopy = async () => {
    if (!enhancedDiffResult) return;

    const text = enhancedDiffResult.summary.changes
      .map(c => `${c.change_type}: ${c.description}`)
      .join('\n');

    try {
      await navigator.clipboard.writeText(text);
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (err) {
      console.error('复制失败:', err);
    }
  };

  if (!enhancedDiffResult) {
    return (
      <div className="flex flex-col items-center justify-center h-full text-gray-400 gap-4">
        <FileSearch className="w-20 h-20 opacity-20" />
        <div className="text-center">
          <p className="text-lg font-medium text-gray-500 dark:text-gray-400">暂无差异数据</p>
          <p className="text-sm mt-2 max-w-xs">在存档列表中选择一个存档，然后点击另一个的「对比」按钮开始分析</p>
        </div>
      </div>
    );
  }

  if (loading) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="animate-spin w-8 h-8 border-4 border-primary-400 border-t-transparent rounded-full" />
        <span className="ml-3">对比分析中...</span>
      </div>
    );
  }

  return (
    <div className="flex flex-col h-full bg-white dark:bg-gray-800 rounded-lg shadow">
      {/* Header */}
      <div className="flex items-center justify-between p-4 border-b">
        <div className="flex items-center gap-3">
          <FileText className="w-5 h-5 text-primary-500" />
          <h2 className="text-lg font-semibold">差异对比</h2>
          <FileTypeBadge fileType={enhancedDiffResult.file_type} />
        </div>

        <div className="flex items-center gap-2">
          <button onClick={handleCopy} className="btn-secondary">
            {copied ? <Check className="w-4 h-4" /> : <Copy className="w-4 h-4" />}
            <span>复制摘要</span>
          </button>
          <button onClick={clearDiff} className="btn-secondary">
            关闭
          </button>
        </div>
      </div>

      {/* Tab Navigation */}
      <div className="flex border-b">
        <button
          onClick={() => setActiveTab('diff')}
          className={`px-4 py-2 text-sm font-medium ${
            activeTab === 'diff'
              ? 'border-b-2 border-primary-500 text-primary-600'
              : 'text-gray-600 hover:text-gray-800'
          }`}
        >
          详细差异
        </button>
        <button
          onClick={() => setActiveTab('summary')}
          className={`px-4 py-2 text-sm font-medium ${
            activeTab === 'summary'
              ? 'border-b-2 border-primary-500 text-primary-600'
              : 'text-gray-600 hover:text-gray-800'
          }`}
        >
          变更摘要 ({enhancedDiffResult.summary.changes.length})
        </button>
        <button
          onClick={() => setActiveTab('regions')}
          className={`px-4 py-2 text-sm font-medium ${
            activeTab === 'regions'
              ? 'border-b-2 border-primary-500 text-primary-600'
              : 'text-gray-600 hover:text-gray-800'
          }`}
        >
          受影响区域 ({enhancedDiffResult.summary.affected_regions.length})
        </button>
      </div>

      {/* Tab Content */}
      <div className="flex-1 overflow-auto p-4">
        {activeTab === 'diff' && (
          <DiffDetailView result={enhancedDiffResult} />
        )}
        {activeTab === 'summary' && (
          <SummaryView summary={enhancedDiffResult.summary} />
        )}
        {activeTab === 'regions' && (
          <RegionsView regions={enhancedDiffResult.summary.affected_regions} />
        )}
      </div>
    </div>
  );
}

// 文件类型徽章组件
function FileTypeBadge({ fileType }: { fileType: FileType }) {
  const getLabel = () => {
    if (fileType?.type === 'Text') return `${fileType.encoding} 文本`;
    if (fileType?.type === 'Pdf') return `PDF (${fileType.page_count}页)`;
    if (fileType?.type === 'Cad') return `CAD (${fileType.format})`;
    if (fileType?.type === 'Image') return `${fileType.width}x${fileType.height}`;
    return '二进制';
  };

  return (
    <span className="inline-flex items-center gap-1 px-2 py-1 bg-gray-100 dark:bg-gray-700 rounded text-xs">
      {getLabel()}
    </span>
  );
}
