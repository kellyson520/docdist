import { describe, it, expect } from 'vitest';
import type { Archive, DiffResult, DiffStats } from '../index';

describe('Type definitions', () => {
  it('Archive interface is correctly shaped', () => {
    const archive: Archive = {
      id: 'test-id',
      file_path: '/path/to/file.txt',
      file_name: 'file.txt',
      file_size: 1024,
      checksum: 'abc123',
      chunk_count: 1,
      note: 'test note',
      tags: ['important'],
      parent_id: null,
      created_at: '2024-01-01 00:00:00',
    };

    expect(archive.id).toBe('test-id');
    expect(archive.file_name).toBe('file.txt');
    expect(archive.tags).toContain('important');
  });

  it('DiffStats interface is correctly shaped', () => {
    const stats: DiffStats = {
      additions: 10,
      deletions: 5,
      unchanged: 100,
    };

    expect(stats.additions).toBe(10);
    expect(stats.deletions).toBe(5);
    expect(stats.unchanged).toBe(100);
  });
});
