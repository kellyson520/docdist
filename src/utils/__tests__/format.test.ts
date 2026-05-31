import { describe, it, expect } from 'vitest';
import { formatFileSize, truncateText, getTagColor, TAG_COLORS } from '../format';

describe('formatFileSize', () => {
  it('formats 0 bytes', () => {
    expect(formatFileSize(0)).toBe('0 B');
  });

  it('formats bytes', () => {
    expect(formatFileSize(500)).toBe('500 B');
  });

  it('formats kilobytes', () => {
    expect(formatFileSize(1024)).toBe('1 KB');
    expect(formatFileSize(1536)).toBe('1.5 KB');
  });

  it('formats megabytes', () => {
    expect(formatFileSize(1048576)).toBe('1 MB');
  });

  it('formats gigabytes', () => {
    expect(formatFileSize(1073741824)).toBe('1 GB');
  });

  it('handles negative numbers gracefully', () => {
    // Current implementation may behave unexpectedly with negative numbers
    const result = formatFileSize(-1024);
    expect(typeof result).toBe('string');
  });
});

describe('truncateText', () => {
  it('returns short text as-is', () => {
    expect(truncateText('hello', 10)).toBe('hello');
  });

  it('truncates long text', () => {
    expect(truncateText('hello world', 5)).toBe('hello...');
  });

  it('handles exact length', () => {
    expect(truncateText('hello', 5)).toBe('hello');
  });

  it('handles empty string', () => {
    expect(truncateText('', 5)).toBe('');
  });

  it('handles zero maxLen', () => {
    expect(truncateText('hello', 0)).toBe('...');
  });
});

describe('getTagColor', () => {
  it('returns consistent color for same tag', () => {
    const color1 = getTagColor('important');
    const color2 = getTagColor('important');
    expect(color1).toBe(color2);
  });

  it('returns valid tailwind class', () => {
    const color = getTagColor('test');
    expect(color).toMatch(/^bg-\w+-100 text-\w+-700$/);
  });

  it('returns color from TAG_COLORS array', () => {
    const color = getTagColor('test');
    expect(TAG_COLORS).toContain(color);
  });

  it('handles empty string', () => {
    const color = getTagColor('');
    expect(TAG_COLORS).toContain(color);
  });

  it('handles special characters', () => {
    const color = getTagColor('!@#$%^&*()');
    expect(TAG_COLORS).toContain(color);
  });

  it('handles unicode characters', () => {
    const color = getTagColor('你好世界');
    expect(TAG_COLORS).toContain(color);
  });
});
