// Service exports
import type { IStorageService } from './api/storage.interface';

// Dynamic platform detection and adapter loading
async function loadStorageAdapter(): Promise<IStorageService> {
  // Check if running in Tauri
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    const { TauriStorageAdapter } = await import('./adapters/tauri/storage.adapter');
    return new TauriStorageAdapter();
  }

  // Fall back to web adapter
  const { WebStorageAdapter } = await import('./adapters/web/storage.adapter');
  return new WebStorageAdapter();
}

// Service factory
export class ServiceFactory {
  private static storageInstance: IStorageService | null = null;

  static async getStorageService(): Promise<IStorageService> {
    if (!this.storageInstance) {
      this.storageInstance = await loadStorageAdapter();
    }
    return this.storageInstance;
  }
}

export * from './api/storage.interface';
