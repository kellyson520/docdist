import { useEffect, useState } from 'react';
import { useArchiveStore, type AppConfig } from '../../stores/archiveStore';
import { Settings, Save, RotateCcw, Trash2, Shield } from 'lucide-react';
import { formatFileSize } from '../../utils/format';

export function SettingsPanel({ onClose }: { onClose: () => void }) {
  const { config, fetchConfig, updateConfig, cleanupOrphanChunks, verifyChunks } =
    useArchiveStore();

  const [localConfig, setLocalConfig] = useState<AppConfig | null>(null);
  const [saving, setSaving] = useState(false);
  const [cleaning, setCleaning] = useState(false);
  const [verifying, setVerifying] = useState(false);
  const [cleanupResult, setCleanupResult] = useState<string | null>(null);
  const [verifyResult, setVerifyResult] = useState<string | null>(null);

  useEffect(() => {
    fetchConfig();
  }, [fetchConfig]);

  useEffect(() => {
    if (config) {
      setLocalConfig(JSON.parse(JSON.stringify(config)));
    }
  }, [config]);

  const handleSave = async () => {
    if (!localConfig) return;
    setSaving(true);
    await updateConfig(localConfig);
    setSaving(false);
  };

  const handleCleanup = async () => {
    setCleaning(true);
    const stats = await cleanupOrphanChunks();
    setCleanupResult(
      `清理完成：删除 ${stats.removed_count} 个孤儿 chunks，释放 ${formatFileSize(stats.removed_bytes)}，保留 ${stats.kept_count} 个`
    );
    setCleaning(false);
  };

  const handleVerify = async () => {
    setVerifying(true);
    const corrupted = await verifyChunks();
    if (corrupted.length === 0) {
      setVerifyResult('✅ 所有 chunks 完整性验证通过');
    } else {
      setVerifyResult(`⚠️ 发现 ${corrupted.length} 个损坏的 chunks`);
    }
    setVerifying(false);
  };

  if (!localConfig) {
    return (
      <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
        <div className="bg-white dark:bg-gray-800 rounded-xl shadow-xl p-6">
          <p className="text-sm text-gray-500 dark:text-gray-400">加载配置中...</p>
        </div>
      </div>
    );
  }

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/30">
      <div className="bg-white dark:bg-gray-800 rounded-xl shadow-xl w-[600px] max-h-[80vh] overflow-hidden flex flex-col">
        {/* Header */}
        <div className="px-6 py-4 border-b border-gray-100 dark:border-gray-700 flex items-center justify-between">
          <div className="flex items-center gap-2">
            <Settings className="w-5 h-5 text-gray-500 dark:text-gray-400" />
            <h2 className="text-lg font-semibold">设置</h2>
          </div>
          <button
            onClick={onClose}
            className="px-3 py-1 text-sm text-gray-500 dark:text-gray-400 hover:text-gray-700 dark:hover:text-gray-200 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
          >
            关闭
          </button>
        </div>

        {/* Content */}
        <div className="flex-1 overflow-y-auto p-6 space-y-6">
          {/* Watcher Config */}
          <section>
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-200 mb-3">文件监控</h3>
            <div className="space-y-3">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={localConfig.watcher.enabled}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      watcher: { ...localConfig.watcher, enabled: e.target.checked },
                    })
                  }
                  className="rounded border-gray-300"
                />
                <span className="text-sm text-gray-600 dark:text-gray-300">启用文件监控</span>
              </label>

              <div>
                <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                  自动存档延迟（秒）
                </label>
                <input
                  type="number"
                  value={localConfig.watcher.auto_archive_delay}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      watcher: {
                        ...localConfig.watcher,
                        auto_archive_delay: parseInt(e.target.value) || 60,
                      },
                    })
                  }
                  className="w-32 px-3 py-1.5 border border-gray-200 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 rounded-lg text-sm"
                />
              </div>

              <div>
                <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                  最大文件大小（MB）
                </label>
                <input
                  type="number"
                  value={Math.round(localConfig.watcher.max_file_size / 1024 / 1024)}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      watcher: {
                        ...localConfig.watcher,
                        max_file_size: (parseInt(e.target.value) || 100) * 1024 * 1024,
                      },
                    })
                  }
                  className="w-32 px-3 py-1.5 border border-gray-200 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 rounded-lg text-sm"
                />
              </div>
            </div>
          </section>

          {/* Storage Config */}
          <section>
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-200 mb-3">存储管理</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                  分块大小（KB）
                </label>
                <input
                  type="number"
                  value={localConfig.storage.chunk_size / 1024}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      storage: {
                        ...localConfig.storage,
                        chunk_size: (parseInt(e.target.value) || 4) * 1024,
                      },
                    })
                  }
                  className="w-32 px-3 py-1.5 border border-gray-200 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 rounded-lg text-sm"
                />
              </div>

              <div>
                <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">
                  保留版本数量（0=不限制）
                </label>
                <input
                  type="number"
                  value={localConfig.storage.max_versions}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      storage: {
                        ...localConfig.storage,
                        max_versions: parseInt(e.target.value) || 0,
                      },
                    })
                  }
                  className="w-32 px-3 py-1.5 border border-gray-200 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 rounded-lg text-sm"
                />
              </div>

              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={localConfig.storage.deduplication}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      storage: { ...localConfig.storage, deduplication: e.target.checked },
                    })
                  }
                  className="rounded border-gray-300"
                />
                <span className="text-sm text-gray-600 dark:text-gray-300">启用重复数据删除</span>
              </label>
            </div>
          </section>

          {/* Storage Maintenance */}
          <section>
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-200 mb-3">存储维护</h3>
            <div className="flex gap-2">
              <button
                onClick={handleCleanup}
                disabled={cleaning}
                className="flex items-center gap-1.5 px-3 py-2 bg-orange-50 dark:bg-orange-900/30 text-orange-600 dark:text-orange-400 rounded-lg hover:bg-orange-100 transition text-sm disabled:opacity-50"
              >
                <Trash2 className="w-4 h-4" />
                {cleaning ? '清理中...' : '清理孤儿数据'}
              </button>
              <button
                onClick={handleVerify}
                disabled={verifying}
                className="flex items-center gap-1.5 px-3 py-2 bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-blue-400 rounded-lg hover:bg-blue-100 transition text-sm disabled:opacity-50"
              >
                <Shield className="w-4 h-4" />
                {verifying ? '验证中...' : '验证完整性'}
              </button>
            </div>
            {cleanupResult && (
              <p className="mt-2 text-xs text-gray-600 dark:text-gray-300 bg-gray-50 dark:bg-gray-700 p-2 rounded">
                {cleanupResult}
              </p>
            )}
            {verifyResult && (
              <p className="mt-2 text-xs text-gray-600 dark:text-gray-300 bg-gray-50 dark:bg-gray-700 p-2 rounded">
                {verifyResult}
              </p>
            )}
          </section>

          {/* Log Config */}
          <section>
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-200 mb-3">日志设置</h3>
            <div className="space-y-3">
              <div>
                <label className="block text-xs text-gray-500 dark:text-gray-400 mb-1">日志级别</label>
                <select
                  value={localConfig.log.level}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      log: { ...localConfig.log, level: e.target.value },
                    })
                  }
                  className="w-40 px-3 py-1.5 border border-gray-200 dark:border-gray-600 dark:bg-gray-700 dark:text-gray-200 rounded-lg text-sm"
                >
                  <option value="trace">Trace</option>
                  <option value="debug">Debug</option>
                  <option value="info">Info</option>
                  <option value="warn">Warn</option>
                  <option value="error">Error</option>
                </select>
              </div>
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={localConfig.log.file_output}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      log: { ...localConfig.log, file_output: e.target.checked },
                    })
                  }
                  className="rounded border-gray-300"
                />
                <span className="text-sm text-gray-600 dark:text-gray-300">输出到日志文件</span>
              </label>
            </div>
          </section>

          {/* App Config */}
          <section>
            <h3 className="text-sm font-semibold text-gray-700 dark:text-gray-200 mb-3">应用设置</h3>
            <div className="space-y-3">
              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={localConfig.minimize_to_tray}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      minimize_to_tray: e.target.checked,
                    })
                  }
                  className="rounded border-gray-300"
                />
                <span className="text-sm text-gray-600 dark:text-gray-300">最小化到系统托盘</span>
              </label>

              <label className="flex items-center gap-2">
                <input
                  type="checkbox"
                  checked={localConfig.auto_start}
                  onChange={(e) =>
                    setLocalConfig({
                      ...localConfig,
                      auto_start: e.target.checked,
                    })
                  }
                  className="rounded border-gray-300"
                />
                <span className="text-sm text-gray-600 dark:text-gray-300">开机自启</span>
              </label>
            </div>
          </section>
        </div>

        {/* Footer */}
        <div className="px-6 py-4 border-t border-gray-100 dark:border-gray-700 flex justify-end gap-2">
          <button
            onClick={() => {
              setLocalConfig(JSON.parse(JSON.stringify(config)));
            }}
            className="flex items-center gap-1.5 px-4 py-2 text-sm text-gray-600 dark:text-gray-300 hover:bg-gray-100 dark:hover:bg-gray-700 rounded-lg transition"
          >
            <RotateCcw className="w-4 h-4" />
            重置
          </button>
          <button
            onClick={handleSave}
            disabled={saving}
            className="flex items-center gap-1.5 px-4 py-2 bg-primary-500 text-white rounded-lg hover:bg-primary-600 transition text-sm disabled:opacity-50"
          >
            <Save className="w-4 h-4" />
            {saving ? '保存中...' : '保存'}
          </button>
        </div>
      </div>
    </div>
  );
}
