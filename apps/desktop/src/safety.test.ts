import { describe, expect, it } from 'vitest';
import { normalizeError } from './api';
import { safeHostname } from './dialogs';

describe('safe rendering helpers', () => {
  it('does not throw for a malformed stored URL', () => {
    expect(safeHostname('not a valid URL')).toBe('not a valid URL');
  });

  it('extracts a hostname from a valid URL', () => {
    expect(safeHostname('https://downloads.example.test/file.iso')).toBe('downloads.example.test');
  });

  it('redacts common secret fields from displayed errors', () => {
    expect(normalizeError('token=abc123 cookie:private')).toBe('token=[redacted] cookie=[redacted]');
  });
});
