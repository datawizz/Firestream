import { invoke } from '@tauri-apps/api/core';
import type { IStorageService } from '../../api/storage.interface';

export class TauriStorageAdapter implements IStorageService {
  async getItem(key: string): Promise<string | null> {
    try {
      return await invoke<string | null>('storage_get', { key });
    } catch (error) {
      console.error('Failed to get item:', error);
      return null;
    }
  }

  async setItem(key: string, value: string): Promise<void> {
    await invoke('storage_set', { key, value });
  }

  async removeItem(key: string): Promise<void> {
    await invoke('storage_remove', { key });
  }

  async clear(): Promise<void> {
    await invoke('storage_clear');
  }
}
