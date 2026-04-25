import { invoke } from '@tauri-apps/api/core';

/**
 * Tauri command wrappers with proper TypeScript types
 */

/**
 * Store a value securely in the OS keychain
 */
export async function secureStorageSet(key: string, value: string): Promise<void> {
  return invoke('secure_storage_set', { key, value });
}

/**
 * Retrieve a value from the OS keychain
 */
export async function secureStorageGet(key: string): Promise<string> {
  return invoke('secure_storage_get', { key });
}

/**
 * Remove a value from the OS keychain
 */
export async function secureStorageRemove(key: string): Promise<void> {
  return invoke('secure_storage_remove', { key });
}

/**
 * Check if a key exists in the OS keychain
 */
export async function secureStorageHas(key: string): Promise<boolean> {
  return invoke('secure_storage_has', { key });
}

/**
 * Get a value from in-memory storage
 */
export async function storageGet(key: string): Promise<string | null> {
  return invoke('storage_get', { key });
}

/**
 * Set a value in in-memory storage
 */
export async function storageSet(key: string, value: string): Promise<void> {
  return invoke('storage_set', { key, value });
}

/**
 * Remove a value from in-memory storage
 */
export async function storageRemove(key: string): Promise<void> {
  return invoke('storage_remove', { key });
}

/**
 * Clear all in-memory storage
 */
export async function storageClear(): Promise<void> {
  return invoke('storage_clear');
}
