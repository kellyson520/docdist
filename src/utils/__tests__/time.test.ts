import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest';
import {
  formatRelativeTime,
  formatAbsoluteTime,
  formatDateOnly,
  formatTimeOnly,
  isToday,
  isYesterday,
  formatSmartTime,
} from '../time';

describe('formatRelativeTime', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-01-15T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns "刚刚" for very recent times', () => {
    const date = new Date('2024-01-15T11:59:55Z').toISOString();
    expect(formatRelativeTime(date)).toBe('刚刚');
  });

  it('returns seconds ago', () => {
    const date = new Date('2024-01-15T11:59:30Z').toISOString();
    expect(formatRelativeTime(date)).toBe('30 秒前');
  });

  it('returns minutes ago', () => {
    const date = new Date('2024-01-15T11:55:00Z').toISOString();
    expect(formatRelativeTime(date)).toBe('5 分钟前');
  });

  it('returns hours ago', () => {
    const date = new Date('2024-01-15T09:00:00Z').toISOString();
    expect(formatRelativeTime(date)).toBe('3 小时前');
  });

  it('returns days ago', () => {
    const date = new Date('2024-01-13T12:00:00Z').toISOString();
    expect(formatRelativeTime(date)).toBe('2 天前');
  });

  it('returns weeks ago', () => {
    const date = new Date('2024-01-01T12:00:00Z').toISOString();
    expect(formatRelativeTime(date)).toBe('2 周前');
  });

  it('returns months ago', () => {
    const date = new Date('2023-11-15T12:00:00Z').toISOString();
    expect(formatRelativeTime(date)).toBe('2 个月前');
  });

  it('returns years ago', () => {
    const date = new Date('2022-01-15T12:00:00Z').toISOString();
    expect(formatRelativeTime(date)).toBe('2 年前');
  });
});

describe('formatAbsoluteTime', () => {
  it('formats date correctly', () => {
    const date = '2024-01-15T14:30:45Z';
    const result = formatAbsoluteTime(date);
    expect(result).toContain('2024');
    expect(result).toContain('01');
    expect(result).toContain('15');
  });
});

describe('formatDateOnly', () => {
  it('formats date without time', () => {
    const date = '2024-01-15T14:30:45Z';
    const result = formatDateOnly(date);
    expect(result).toContain('2024');
    expect(result).not.toContain('14');
  });
});

describe('formatTimeOnly', () => {
  it('formats time without date', () => {
    const date = '2024-01-15T14:30:45Z';
    const result = formatTimeOnly(date);
    const expectedHour = String(new Date(date).getHours()).padStart(2, '0');
    expect(result).toContain(expectedHour);
    expect(result).toContain('30');
    expect(result).not.toContain('2024');
  });
});

describe('isToday', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-01-15T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns true for today', () => {
    expect(isToday('2024-01-15T14:30:00Z')).toBe(true);
  });

  it('returns false for yesterday', () => {
    expect(isToday('2024-01-14T14:30:00Z')).toBe(false);
  });
});

describe('isYesterday', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-01-15T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns true for yesterday', () => {
    expect(isYesterday('2024-01-14T14:30:00Z')).toBe(true);
  });

  it('returns false for today', () => {
    expect(isYesterday('2024-01-15T14:30:00Z')).toBe(false);
  });

  it('returns false for two days ago', () => {
    expect(isYesterday('2024-01-13T14:30:00Z')).toBe(false);
  });
});

describe('formatSmartTime', () => {
  beforeEach(() => {
    vi.useFakeTimers();
    vi.setSystemTime(new Date('2024-01-15T12:00:00Z'));
  });

  afterEach(() => {
    vi.useRealTimers();
  });

  it('returns relative time for today', () => {
    const date = '2024-01-15T11:55:00Z';
    expect(formatSmartTime(date)).toBe('5 分钟前');
  });

  it('returns "昨天 HH:mm" for yesterday', () => {
    const date = '2024-01-14T14:30:00Z';
    const result = formatSmartTime(date);
    const expectedHour = String(new Date(date).getHours()).padStart(2, '0');
    expect(result).toContain('昨天');
    expect(result).toContain(expectedHour);
  });

  it('returns weekday for this week', () => {
    // Jan 12, 2024 is Friday (day 5), 3 days before Jan 15
    const date = '2024-01-12T14:30:00Z';
    const result = formatSmartTime(date);
    expect(result).toContain('周五');
  });

  it('returns MM-DD for this year', () => {
    // Jan 2, 2024 is 13 days before Jan 15, should show MM-DD
    const date = '2024-01-02T14:30:00Z';
    const result = formatSmartTime(date);
    expect(result).toContain('01');
    expect(result).toContain('02');
  });

  it('returns full date for other years', () => {
    const date = '2023-01-15T14:30:00Z';
    const result = formatSmartTime(date);
    expect(result).toContain('2023');
  });
});
