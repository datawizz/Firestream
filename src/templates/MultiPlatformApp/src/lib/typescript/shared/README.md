# @multi-platform-app/shared

Shared TypeScript library for multi-platform applications with platform abstraction layer.

## Installation

This package is part of the workspace and will be automatically linked:

```json
{
  "dependencies": {
    "@multi-platform-app/shared": "workspace:*"
  }
}
```

## Usage

### Platform Detection

```typescript
import { usePlatform } from '@multi-platform-app/shared/hooks';

function MyComponent() {
  const { platform, isTauri, isWeb } = usePlatform();

  return (
    <div>
      Running on: {platform}
      {isTauri && <p>Native desktop features available!</p>}
    </div>
  );
}
```

### Storage Service

```typescript
import { useStorage } from '@multi-platform-app/shared/hooks';

function SettingsComponent() {
  const { value, loading, setValue } = useStorage('user-settings', {
    theme: 'light',
    language: 'en'
  });

  if (loading) return <div>Loading...</div>;

  return (
    <button onClick={() => setValue({ ...value, theme: 'dark' })}>
      Switch to Dark Mode
    </button>
  );
}
```

### Direct Service Access

```typescript
import { ServiceFactory } from '@multi-platform-app/shared/services';

async function saveData() {
  const storage = await ServiceFactory.getStorageService();
  await storage.setItem('key', 'value');
  const data = await storage.getItem('key');
}
```

### State Management

```typescript
import { useExampleStore } from '@multi-platform-app/shared/stores';

function Counter() {
  const { count, increment, decrement, reset } = useExampleStore();

  return (
    <div>
      <p>Count: {count}</p>
      <button onClick={increment}>+</button>
      <button onClick={decrement}>-</button>
      <button onClick={reset}>Reset</button>
    </div>
  );
}
```

### UI Components

```typescript
import { Button } from '@multi-platform-app/shared/components';

function App() {
  return (
    <>
      <Button variant="default">Default</Button>
      <Button variant="outline">Outline</Button>
      <Button variant="destructive">Delete</Button>
      <Button size="sm">Small</Button>
      <Button size="lg">Large</Button>
    </>
  );
}
```

### Utilities

```typescript
import { cn } from '@multi-platform-app/shared/utils';

function MyComponent({ className }) {
  return (
    <div className={cn('base-styles', className, {
      'active': isActive,
      'disabled': isDisabled
    })}>
      Content
    </div>
  );
}
```

## Architecture

### Platform Adapters

The library uses a platform adapter pattern to provide unified APIs across web and desktop:

```
Service Interface → Service Factory → Platform Adapter
                         ↓
                    [Tauri Adapter]
                    [Web Adapter]
```

- **Interface**: Defines the contract (e.g., `IStorageService`)
- **Factory**: Detects platform and loads appropriate adapter
- **Adapters**: Platform-specific implementations
  - `tauri/`: Uses Tauri APIs for native functionality
  - `web/`: Uses browser APIs for web functionality

### Adding New Services

1. Define interface in `src/services/api/`:
```typescript
// my-service.interface.ts
export interface IMyService {
  doSomething(): Promise<void>;
}
```

2. Create platform adapters:
```typescript
// adapters/tauri/my-service.adapter.ts
import { invoke } from '@tauri-apps/api/core';
import type { IMyService } from '../../api/my-service.interface';

export class TauriMyServiceAdapter implements IMyService {
  async doSomething(): Promise<void> {
    await invoke('my_command');
  }
}
```

```typescript
// adapters/web/my-service.adapter.ts
import type { IMyService } from '../../api/my-service.interface';

export class WebMyServiceAdapter implements IMyService {
  async doSomething(): Promise<void> {
    // Web implementation
  }
}
```

3. Add to service factory:
```typescript
// services/index.ts
async function loadMyServiceAdapter(): Promise<IMyService> {
  if (typeof window !== 'undefined' && '__TAURI__' in window) {
    const { TauriMyServiceAdapter } = await import('./adapters/tauri/my-service.adapter');
    return new TauriMyServiceAdapter();
  }
  const { WebMyServiceAdapter } = await import('./adapters/web/my-service.adapter');
  return new WebMyServiceAdapter();
}

export class ServiceFactory {
  private static myServiceInstance: IMyService | null = null;

  static async getMyService(): Promise<IMyService> {
    if (!this.myServiceInstance) {
      this.myServiceInstance = await loadMyServiceAdapter();
    }
    return this.myServiceInstance;
  }
}
```

## Development

### Build

```bash
pnpm build
```

Outputs:
- ESM: `dist/**/*.mjs`
- CJS: `dist/**/*.js`
- Types: `dist/**/*.d.ts`
- CSS: `dist/styles.css`

### Watch Mode

```bash
pnpm dev
```

### Type Checking

```bash
pnpm typecheck
```

## Exports

The package provides multiple entry points:

- `@multi-platform-app/shared` - Main exports
- `@multi-platform-app/shared/components` - UI components
- `@multi-platform-app/shared/stores` - Zustand stores
- `@multi-platform-app/shared/hooks` - React hooks
- `@multi-platform-app/shared/services` - Service factory and interfaces
- `@multi-platform-app/shared/utils` - Utility functions
- `@multi-platform-app/shared/styles` - CSS styles

## Dependencies

### Peer Dependencies

Required in consuming apps:
- React 18+ or 19+
- React DOM 18+ or 19+
- @tanstack/react-query ^5.0.0
- zustand ^5.0.0

### Optional Dependencies

- @tauri-apps/api ^2.0.0 (automatically available in Tauri apps)

## License

Private package - part of multi-platform-app template
