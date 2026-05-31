import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/tauri';
import type { AppConfig } from '../stores/archiveStore';

type Theme = 'light' | 'dark' | 'system';

export function useTheme() {
  const [theme, setThemeState] = useState<Theme>(() => {
    const stored = localStorage.getItem('theme');
    if (stored && ['light', 'dark', 'system'].includes(stored)) {
      return stored as Theme;
    }
    return 'system';
  });

  const [resolvedTheme, setResolvedTheme] = useState<'light' | 'dark'>(() =>
    window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
  );

  // 初始化时同步 localStorage 与后端 config.theme
  useEffect(() => {
    const stored = localStorage.getItem('theme');
    invoke<AppConfig>('get_config').then(config => {
      if (!stored && config.theme) {
        // localStorage 无值，使用后端值
        const backendTheme = config.theme as Theme;
        if (['light', 'dark', 'system'].includes(backendTheme)) {
          setThemeState(backendTheme);
          localStorage.setItem('theme', backendTheme);
        }
      } else if (stored) {
        // localStorage 有值，同步到后端（确保一致）
        invoke('update_config', { newConfig: { ...config, theme: stored } }).catch(() => {});
      }
    }).catch(() => { /* 静默失败 */ });
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // Resolve system theme
  useEffect(() => {
    const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)');
    
    const handleChange = () => {
      if (theme === 'system') {
        setResolvedTheme(mediaQuery.matches ? 'dark' : 'light');
      }
    };

    handleChange();
    mediaQuery.addEventListener('change', handleChange);
    return () => mediaQuery.removeEventListener('change', handleChange);
  }, [theme]);

  // Apply theme to document
  useEffect(() => {
    const effectiveTheme = theme === 'system' ? resolvedTheme : theme;
    document.documentElement.setAttribute('data-theme', effectiveTheme);
  }, [theme, resolvedTheme]);

  const setTheme = useCallback((newTheme: Theme) => {
    setThemeState(newTheme);
    localStorage.setItem('theme', newTheme);
    // 同步到后端配置（尽力而为，失败不阻塞 UI）
    invoke<AppConfig>('get_config').then(config => {
      invoke('update_config', { newConfig: { ...config, theme: newTheme } });
    }).catch(() => { /* 静默失败 */ });
  }, []);

  const toggleTheme = useCallback(() => {
    setTheme(resolvedTheme === 'light' ? 'dark' : 'light');
  }, [resolvedTheme, setTheme]);

  return {
    theme,
    resolvedTheme,
    setTheme,
    toggleTheme,
  };
}
