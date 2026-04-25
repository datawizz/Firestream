import { useState, useEffect } from 'react';
import { detectPlatform } from '../utils/platform';

export function usePlatform() {
  const [platform, setPlatform] = useState<ReturnType<typeof detectPlatform>>('unknown');

  useEffect(() => {
    setPlatform(detectPlatform());
  }, []);

  return {
    platform,
    isTauri: platform === 'tauri',
    isWeb: platform === 'web',
  };
}
