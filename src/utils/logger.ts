/**
 * DocDist 前端日志系统
 * 支持 debug/info/warn/error 级别，持久化到 localStorage，可导出
 */

export type LogLevel = 'debug' | 'info' | 'warn' | 'error';

export interface LogEntry {
  id: string;
  level: LogLevel;
  message: string;
  data?: unknown;
  timestamp: string;
  source: string;
}

const LOG_STORAGE_KEY = 'docdist_logs';
const MAX_LOGS = 500;

export class Logger {
  private source: string;
  private static allLogs: LogEntry[] = [];
  private static listeners: Array<(entry: LogEntry) => void> = [];

  constructor(source: string) {
    this.source = source;
  }

  private log(level: LogLevel, message: string, data?: unknown) {
    const entry: LogEntry = {
      id: `${Date.now()}-${Math.random().toString(36).slice(2, 8)}`,
      level,
      message,
      data,
      timestamp: new Date().toISOString(),
      source: this.source,
    };

    // 控制台输出
    const prefix = `[${entry.timestamp.slice(11, 19)}] [${level.toUpperCase()}] [${this.source}]`;
    const consoleFn = level === 'error' ? console.error
      : level === 'warn' ? console.warn
      : level === 'debug' ? console.debug
      : console.log;

    if (data !== undefined) {
      consoleFn(`${prefix} ${message}`, data);
    } else {
      consoleFn(`${prefix} ${message}`);
    }

    // 内存存储
    Logger.allLogs.push(entry);
    if (Logger.allLogs.length > MAX_LOGS) {
      Logger.allLogs = Logger.allLogs.slice(-MAX_LOGS);
    }

    // 通知监听器
    for (const listener of Logger.listeners) {
      try {
        listener(entry);
      } catch {
        // 监听器错误不应影响日志系统
      }
    }

    // 持久化（仅 warn/error）
    if (level === 'warn' || level === 'error') {
      this.persist();
    }
  }

  debug(message: string, data?: unknown) {
    this.log('debug', message, data);
  }

  info(message: string, data?: unknown) {
    this.log('info', message, data);
  }

  warn(message: string, data?: unknown) {
    this.log('warn', message, data);
  }

  error(message: string, data?: unknown) {
    this.log('error', message, data);
  }

  /** 创建子 logger */
  child(subSource: string): Logger {
    return new Logger(`${this.source}:${subSource}`);
  }

  // ========== 静态方法 ==========

  static getAll(): LogEntry[] {
    return [...Logger.allLogs];
  }

  static getByLevel(level: LogLevel): LogEntry[] {
    return Logger.allLogs.filter(l => l.level === level);
  }

  static getBySource(source: string): LogEntry[] {
    return Logger.allLogs.filter(l => l.source.includes(source));
  }

  static clear() {
    Logger.allLogs = [];
    try {
      localStorage.removeItem(LOG_STORAGE_KEY);
    } catch {
      // ignore
    }
  }

  static subscribe(listener: (entry: LogEntry) => void): () => void {
    Logger.listeners.push(listener);
    return () => {
      Logger.listeners = Logger.listeners.filter(l => l !== listener);
    };
  }

  static export(): string {
    return JSON.stringify(Logger.allLogs, null, 2);
  }

  private persist() {
    try {
      const errors = Logger.allLogs
        .filter(l => l.level === 'warn' || l.level === 'error')
        .slice(-50);
      localStorage.setItem(LOG_STORAGE_KEY, JSON.stringify(errors));
    } catch {
      // localStorage 满了或不可用
    }
  }
}

/** 通用 logger 实例 */
export const logger = new Logger('app');

/** 创建命名 logger */
export function createLogger(source: string): Logger {
  return new Logger(source);
}
