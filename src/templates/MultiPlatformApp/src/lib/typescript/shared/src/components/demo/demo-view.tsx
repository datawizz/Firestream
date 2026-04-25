'use client';

import { useState, type ReactNode } from 'react';
import { usePlatform } from '../../hooks/use-platform';
import { useStorage } from '../../hooks/use-storage';
import { useExampleStore } from '../../stores/example.store';
import { Button } from '../ui/button';

export interface DemoViewProps {
  platformLabel: string;
  headerExtra?: ReactNode;
  footerExtra?: ReactNode;
}

export function DemoView({ platformLabel, headerExtra, footerExtra }: DemoViewProps) {
  const { platform, isTauri, isWeb } = usePlatform();
  const { count, increment, decrement, reset } = useExampleStore();
  const { value: storedName, setValue: setStoredName, loading } = useStorage<string>('userName', 'Guest');
  const [name, setName] = useState('');

  const handleSaveName = () => {
    if (name.trim()) {
      setStoredName(name);
      setName('');
    }
  };

  return (
    <div className="min-h-screen bg-background text-foreground">
      <div className="container mx-auto p-8">
        {/* Header */}
        <div className="flex justify-between items-center mb-8">
          <h1 className="text-4xl font-bold">Multi-Platform App - {platformLabel}</h1>
          {headerExtra && <div className="flex gap-2">{headerExtra}</div>}
        </div>

        {/* Platform Detection */}
        <div className="mb-8 p-6 border rounded-lg">
          <h2 className="text-2xl font-semibold mb-4">Platform Detection</h2>
          <p className="text-lg">
            Current Platform: <span className="font-mono font-bold">{platform}</span>
          </p>
          <p className="text-lg">
            Running on {isTauri ? 'Tauri' : 'Web'}:{' '}
            <span className="font-mono">{isTauri || isWeb ? '\u2705 Yes' : '\u274C No'}</span>
          </p>
        </div>

        {/* Counter Example */}
        <div className="mb-8 p-6 border rounded-lg">
          <h2 className="text-2xl font-semibold mb-4">Zustand Store Example</h2>
          <p className="text-lg mb-4">Count: {count}</p>
          <div className="flex gap-2">
            <Button onClick={increment}>Increment</Button>
            <Button onClick={decrement} variant="outline">Decrement</Button>
            <Button onClick={reset} variant="ghost">Reset</Button>
          </div>
        </div>

        {/* Storage Example */}
        <div className="mb-8 p-6 border rounded-lg">
          <h2 className="text-2xl font-semibold mb-4">Platform Storage Example</h2>
          {loading ? (
            <p>Loading stored value...</p>
          ) : (
            <>
              <p className="text-lg mb-4">
                Stored Name: <span className="font-mono font-bold">{storedName}</span>
              </p>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Enter your name"
                  className="px-4 py-2 border rounded"
                />
                <Button onClick={handleSaveName}>Save</Button>
              </div>
              <p className="text-sm text-muted-foreground mt-2">
                Storage adapter: {isTauri ? 'Tauri IPC' : 'Web localStorage'}
              </p>
            </>
          )}
        </div>

        {/* Platform-specific footer content */}
        {footerExtra}
      </div>
    </div>
  );
}
