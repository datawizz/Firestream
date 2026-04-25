import { invoke } from '@tauri-apps/api/core';

export async function secureStorageSet(key: string, value: string): Promise<void> {
  return invoke('secure_storage_set', { key, value });
}

export async function secureStorageGet(key: string): Promise<string> {
  return invoke('secure_storage_get', { key });
}

export async function secureStorageRemove(key: string): Promise<void> {
  return invoke('secure_storage_remove', { key });
}

export async function secureStorageHas(key: string): Promise<boolean> {
  return invoke('secure_storage_has', { key });
}

export async function storageGet(key: string): Promise<string | null> {
  return invoke('storage_get', { key });
}

export async function storageSet(key: string, value: string): Promise<void> {
  return invoke('storage_set', { key, value });
}

export async function storageRemove(key: string): Promise<void> {
  return invoke('storage_remove', { key });
}

export async function storageClear(): Promise<void> {
  return invoke('storage_clear');
}
