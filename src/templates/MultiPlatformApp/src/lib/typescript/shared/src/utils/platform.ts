import type { Platform } from '../types/platform';

export function detectPlatform(): Platform {
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    return 'tauri';
  }
  if (typeof window !== 'undefined' && typeof document !== 'undefined') {
    return 'web';
  }
  return 'unknown';
}
