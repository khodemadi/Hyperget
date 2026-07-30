import { invoke } from '@tauri-apps/api/core';
import type { Download, GlobalStatus, Priority, Settings } from './types';

export interface WildcardRange { start: number; end: number; step: number; padding: number }
export interface BrowserEnvelope { type: string; payload?: { url?: string; links?: Array<{ url: string }> } }
export interface ProbeResult { final_url: string; filename: string; total: number | null; ranges: boolean; etag: string | null; last_modified: string | null }

export class AppCommandError extends Error {
  constructor(public operation: string, message: string, public technicalDetails?: string) {
    super(message); this.name = 'AppCommandError';
  }
}
export function normalizeError(value: unknown): string {
  const raw = value instanceof Error ? value.message : typeof value === 'string' ? value : 'An unexpected application error occurred.';
  return raw.replace(/(authorization|cookie|token|password|secret)\s*[:=]\s*[^\s,;]+/gi, '$1=[redacted]').slice(0, 1000);
}
export async function safeInvoke<T>(command: string, args?: Record<string, unknown>, operation = command): Promise<T> {
  try { return await invoke<T>(command, args); }
  catch (reason) { throw new AppCommandError(operation, normalizeError(reason), normalizeError(reason)); }
}
const settings = () => safeInvoke<Settings>('get_settings', undefined, 'Load settings');
const chooseDirectory = () => safeInvoke<string | null>('choose_download_directory', undefined, 'Choose download directory');
async function destination(requested: string | null) {
  if (requested) return requested;
  const current = await settings();
  if (current.ask_where_to_save) {
    const selected = await chooseDirectory();
    if (!selected) throw new AppCommandError('Choose download directory', 'Download cancelled because no folder was selected.');
    return selected;
  }
  return current.default_download_directory || null;
}
export const api = {
  list: () => safeInvoke<Download[]>('list_downloads', undefined, 'Refresh downloads'), status: () => safeInvoke<GlobalStatus>('get_global_status', undefined, 'Refresh global status'), settings,
  systemDownloads: () => safeInvoke<string>('get_system_download_directory', undefined, 'Find system Downloads folder'), chooseDirectory,
  openLogs: () => safeInvoke<void>('open_logs_folder', undefined, 'Open logs folder'),
  validateDirectory: (path: string) => safeInvoke<void>('validate_download_directory', { path }, 'Validate download directory'),
  browserInbox: () => safeInvoke<BrowserEnvelope[]>('receive_browser_links', undefined, 'Read browser inbox'),
  updateSettings: (value: Settings) => safeInvoke<void>('update_settings', { settings: value }, 'Save settings'),
  probe: (url: string) => safeInvoke<ProbeResult>('probe_download_url', { url }, 'Inspect download URL'),
  add: async (url: string, startImmediately = true, connections = 8, requested: string | null = null) => safeInvoke<string>('add_download', { request: { url, output: null, destination_directory: await destination(requested), connections, start_immediately: startImmediately, checksum_sha256: null } }, 'Add download'),
  addBatch: async (urls: string[], connections: number, startImmediately: boolean, requested: string | null = null) => safeInvoke<string[]>('add_batch_downloads', { urls, connections, startImmediately, destination: await destination(requested) }, 'Add batch downloads'),
  previewBatch: (pattern: string, ranges: WildcardRange[], maximum = 10_000) => safeInvoke<string[]>('preview_batch_download', { request: { pattern, ranges, maximum } }, 'Preview batch'),
  discoverBatch: (pattern: string, padding = 0, maximum = 10_000) => safeInvoke<string[]>('discover_batch_download', { pattern, padding, maximum }, 'Detect batch files'),
  start: (id: string) => safeInvoke<void>('start_download', { id }, 'Start download'), pause: (id: string) => safeInvoke<void>('pause_download', { id }, 'Pause download'), resume: (id: string) => safeInvoke<void>('resume_download', { id }, 'Resume download'), cancel: (id: string) => safeInvoke<void>('cancel_download', { id }, 'Cancel download'),
  remove: (id: string, deleteData: boolean) => safeInvoke<void>('remove_download', { id, deleteData }, deleteData ? 'Delete download' : 'Remove download'),
  clear: () => safeInvoke<number>('clear_downloads', undefined, 'Clear all downloads'),
  priority: (id: string, priority: Priority) => safeInvoke<void>('set_download_priority', { id, priority }, 'Set priority'), top: (id: string) => safeInvoke<void>('move_download_to_top', { id }, 'Move download up'), bottom: (id: string) => safeInvoke<void>('move_download_to_bottom', { id }, 'Move download down'), reorder: (ids: string[]) => safeInvoke<void>('reorder_downloads', { ids }, 'Reorder downloads'),
  startAll: () => safeInvoke<void>('start_all', undefined, 'Start all downloads'), pauseAll: () => safeInvoke<void>('pause_all', undefined, 'Pause all downloads'),
};
