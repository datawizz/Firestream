# TypeScript Libraries

Shared TypeScript packages for cross-platform UI and business logic.

## Purpose

This directory contains TypeScript packages that provide:
- Shared UI components (React)
- State management stores
- Platform-agnostic business logic
- Type definitions and utilities
- Configuration packages (Tailwind, TypeScript, ESLint, Prettier)

## Structure

```
src/lib/typescript/
├── README.md               # This file
├── shared/                 # Main shared library
│   ├── package.json
│   └── src/
│       ├── components/     # React components
│       ├── hooks/          # Custom React hooks
│       ├── services/       # Business logic services
│       ├── stores/         # Zustand state stores
│       ├── types/          # TypeScript type definitions
│       └── utils/          # Utility functions
├── proto-types/            # Generated Protocol Buffer types
│   ├── package.json
│   └── src/generated/
├── tailwind-config/        # Shared Tailwind configuration
├── typescript-config/      # Shared TypeScript configurations
├── eslint-config/          # Shared ESLint configuration
└── prettier-config/        # Shared Prettier configuration
```

## Package Overview

### @multi-platform-app/shared
Main shared library with components, hooks, stores, and utilities.

```typescript
import { Button, Counter } from '@multi-platform-app/shared/components';
import { usePlatform, useStorage } from '@multi-platform-app/shared/hooks';
import { exampleStore } from '@multi-platform-app/shared/stores';
```

### @multi-platform-app/proto-types
Generated TypeScript types from Protocol Buffer definitions.

```typescript
import { ExampleRequest, User } from '@multi-platform-app/proto-types';
```

### Configuration Packages
Shared configurations extended by app packages:

```javascript
// tailwind.config.js
module.exports = require('@multi-platform-app/tailwind-config');

// tsconfig.json
{ "extends": "@multi-platform-app/typescript-config/react.json" }
```

## Creating a New Package

1. Create package directory:
   ```bash
   mkdir -p src/lib/typescript/my-package/src
   ```

2. Create `package.json`:
   ```json
   {
     "name": "@multi-platform-app/my-package",
     "version": "0.1.0",
     "private": true,
     "main": "./src/index.ts",
     "types": "./src/index.ts"
   }
   ```

3. Add to `pnpm-workspace.yaml` (already included via `src/lib/typescript/*`)

4. Install dependencies:
   ```bash
   pnpm install
   ```

## Development

### Building
```bash
# Build all packages
pnpm build

# Build specific package
pnpm --filter @multi-platform-app/shared build
```

### Type Checking
```bash
pnpm typecheck
```

### Linting
```bash
pnpm lint
```

## Platform Adapters

The shared library uses the adapter pattern for platform-specific implementations:

```typescript
// services/api/storage.interface.ts
interface StorageService {
  get(key: string): Promise<string | null>;
  set(key: string, value: string): Promise<void>;
}

// services/adapters/web/storage.adapter.ts
export class WebStorageAdapter implements StorageService { ... }

// services/adapters/tauri/storage.adapter.ts
export class TauriStorageAdapter implements StorageService { ... }
```

## Best Practices

1. **Barrel Exports**: Use index.ts files for clean imports
2. **Tree Shaking**: Export named exports, not default exports
3. **Component Design**: Follow atomic design principles
4. **Type Safety**: Strict TypeScript, no `any`
5. **Documentation**: JSDoc comments for public APIs
