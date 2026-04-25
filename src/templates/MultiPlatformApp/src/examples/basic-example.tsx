/**
 * Basic Platform Adapter Example
 *
 * This example demonstrates how to use the platform adapter pattern
 * to create code that works across web and desktop platforms.
 */

import React, { useEffect, useState } from 'react';
import { ServiceFactory } from '@multi-platform-app/shared/services';
import { usePlatform } from '@multi-platform-app/shared/hooks';

export function BasicExample() {
  const { platform, isTauri, isWeb } = usePlatform();
  const [data, setData] = useState<string>('');
  const [loading, setLoading] = useState(false);

  const saveData = async () => {
    setLoading(true);
    try {
      const storage = await ServiceFactory.getStorageService();
      await storage.setItem('example-key', 'Hello from ' + platform);
      setData('Saved!');
    } catch (error) {
      console.error('Failed to save:', error);
    } finally {
      setLoading(false);
    }
  };

  const loadData = async () => {
    setLoading(true);
    try {
      const storage = await ServiceFactory.getStorageService();
      const value = await storage.getItem('example-key');
      setData(value || 'No data found');
    } catch (error) {
      console.error('Failed to load:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <div className="space-y-4">
      <div>
        <h3>Platform: {platform}</h3>
        <p>Tauri: {isTauri ? 'Yes' : 'No'}</p>
        <p>Web: {isWeb ? 'Yes' : 'No'}</p>
      </div>
      <div>
        <button onClick={saveData} disabled={loading}>
          Save Data
        </button>
        <button onClick={loadData} disabled={loading}>
          Load Data
        </button>
      </div>
      {data && <p>Data: {data}</p>}
    </div>
  );
}
