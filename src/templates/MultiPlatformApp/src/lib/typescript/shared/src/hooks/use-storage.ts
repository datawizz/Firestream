import { useEffect, useState } from 'react';
import { ServiceFactory } from '../services';

export function useStorage<T>(key: string, initialValue: T) {
  const [value, setValue] = useState<T>(initialValue);
  const [loading, setLoading] = useState(true);

  useEffect(() => {
    let mounted = true;

    async function loadValue() {
      try {
        const storage = await ServiceFactory.getStorageService();
        const item = await storage.getItem(key);

        if (mounted && item) {
          setValue(JSON.parse(item));
        }
      } catch (error) {
        console.error('Failed to load storage value:', error);
      } finally {
        if (mounted) {
          setLoading(false);
        }
      }
    }

    loadValue();
    return () => { mounted = false; };
  }, [key]);

  async function updateValue(newValue: T) {
    try {
      const storage = await ServiceFactory.getStorageService();
      await storage.setItem(key, JSON.stringify(newValue));
      setValue(newValue);
    } catch (error) {
      console.error('Failed to save storage value:', error);
    }
  }

  return { value, loading, setValue: updateValue };
}
