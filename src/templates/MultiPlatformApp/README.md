# Multi-Platform App Template

A comprehensive template for building multi-platform applications with shared TypeScript code.

## Platforms

- **Web** - Next.js 15 with App Router and Turbopack
- **Desktop** - Tauri v2 (Windows, macOS, Linux)
- **iOS** - Native app with SwiftUI and XcodeGen

## Features

- ✅ Turbo monorepo with pnpm workspaces
- ✅ Shared TypeScript library with platform adapters
- ✅ Authentication & subscription management (Supabase + Stripe)
- ✅ Nix-based reproducible development environment
- ✅ Docker devcontainer support
- ✅ Protocol Buffer type generation
- ✅ Opinionated UI stack (Radix UI, Zustand, TanStack Query, Tailwind CSS)
- ✅ Working examples and documentation

## Quick Start

```bash
# Clone and navigate to template
cd /path/to/multi_platform_app

# Initialize project
make init

# Install dependencies
make install

# Start development
make run-web-dev      # Web app (http://localhost:3000)
make run-desktop-dev  # Desktop app
make run-ios-dev      # iOS app (macOS only)
```

## Prerequisites

### All Platforms
- Node.js 18+
- pnpm 9.0.0

### Desktop Development
- Rust 1.77+
- Platform-specific dependencies (via Nix or manual install)

### iOS Development (macOS only)
- Xcode 15+
- XcodeGen 2.39+

### Optional
- Docker Desktop (for devcontainer)
- Nix package manager (for reproducible environment)

## Project Structure

```
multi_platform_app/
├── src/
│   ├── app/
│   │   ├── desktop/         # Tauri desktop app
│   │   ├── web/             # Next.js web app
│   │   └── ios/             # iOS app
│   └── lib/
│       ├── proto/           # Protocol Buffer definitions
│       └── typescript/
│           ├── shared/      # Main shared library
│           ├── auth/        # Auth & subscription package
│           ├── proto-types/ # Generated types
│           ├── tailwind-config/
│           └── typescript-config/
├── bin/nix/                 # Nix modules
├── docker/                  # Docker configuration
├── scripts/                 # Build scripts
├── Makefile                 # Build automation
└── turbo.json              # Turbo configuration
```

## Available Commands

```bash
# Setup
make init             # Initialize project
make install          # Install dependencies
make devcontainer     # Build Docker container

# Development
make run-web-dev      # Start Next.js dev server
make run-desktop-dev  # Start Tauri development
make run-ios-dev      # Build and run iOS (macOS only)

# Build
make build            # Build all packages
make build-libs       # Build shared libraries
make build-web        # Build web app
make build-desktop    # Build desktop app
make build-ios        # Build iOS app

# Utilities
make proto-generate   # Generate proto types
make clean            # Clean build artifacts
make lint             # Lint code
make test             # Run tests
make help             # Show all commands
```

## Platform Adapter Pattern

The template uses a platform adapter pattern for shared code:

```typescript
import { ServiceFactory } from '@multi-platform-app/shared/services';

// Automatically loads Tauri or Web adapter based on platform
const storage = await ServiceFactory.getStorageService();
await storage.setItem('key', 'value');
```

See `src/examples/` for more examples.

## Authentication & Subscriptions

The template includes a comprehensive authentication and subscription management package at `src/lib/typescript/auth/`.

### Quick Start

```typescript
import { useAuth, useSubscription } from '@multi-platform-app/auth/hooks';

function MyApp() {
  const { user, signInWithOAuth, signOut } = useAuth();
  const { subscription } = useSubscription();

  return (
    <div>
      {user ? (
        <>
          <p>Welcome, {user.email}!</p>
          {subscription && <p>Plan: {subscription.prices?.products?.name}</p>}
          <button onClick={signOut}>Sign Out</button>
        </>
      ) : (
        <button onClick={() => signInWithOAuth({ provider: 'github' })}>
          Sign In with GitHub
        </button>
      )}
    </div>
  );
}
```

### Features

- **Multi-Platform**: Works on web (Next.js) and desktop (Tauri)
- **Authentication**: Email/password, magic links, OAuth (GitHub, Google)
- **Subscriptions**: Stripe integration for recurring billing
- **Type-Safe**: Full TypeScript support with generated database types
- **SSR Ready**: Built-in server-side rendering support

See [Auth Package Documentation](src/lib/typescript/auth/README.md) for detailed usage.

## Development Workflow

### Web Development
```bash
make run-web-dev
# Opens http://localhost:3000
```

### Desktop Development
```bash
make run-desktop-dev
# Starts Tauri with hot reload
```

### iOS Development (macOS)
```bash
make run-ios-dev
# Generates Xcode project, builds, and launches simulator
```

## Adding Features

### 1. Define Protocol Buffer (Optional)
```protobuf
// src/lib/proto/feature/v1/messages.proto
syntax = "proto3";
package feature.v1;

message FeatureRequest {
  string id = 1;
}
```

### 2. Generate Types
```bash
make proto-generate
```

### 3. Create Service Interface
```typescript
// src/lib/typescript/shared/src/services/api/feature.interface.ts
export interface IFeatureService {
  doSomething(id: string): Promise<void>;
}
```

### 4. Implement Platform Adapters
```typescript
// Tauri adapter
export class TauriFeatureAdapter implements IFeatureService {
  async doSomething(id: string) {
    await invoke('feature_do_something', { id });
  }
}

// Web adapter
export class WebFeatureAdapter implements IFeatureService {
  async doSomething(id: string) {
    await fetch('/api/feature', {
      method: 'POST',
      body: JSON.stringify({ id }),
    });
  }
}
```

### 5. Add to Service Factory
```typescript
static async getFeatureService(): Promise<IFeatureService> {
  if ('__TAURI__' in window) {
    const { TauriFeatureAdapter } = await import('./adapters/tauri/feature.adapter');
    return new TauriFeatureAdapter();
  }
  const { WebFeatureAdapter } = await import('./adapters/web/feature.adapter');
  return new WebFeatureAdapter();
}
```

## Troubleshooting

### Desktop app won't start
- Ensure port 1420 is available: `lsof -ti:1420 | xargs kill -9`
- Rebuild shared library: `make build-libs`

### iOS build fails
- Ensure Xcode is installed: `xcode-select --install`
- Accept Xcode license: `sudo xcodebuild -license accept`
- Regenerate project: `make setup-ios-project`

### Proto generation fails
- Ensure protoc is installed
- Check proto file syntax
- Run `make clean` and try again

## Documentation

- [Architecture Overview](docs/architecture/README.md)
- [Development Guide](docs/guides/development.md)
- [API Reference](docs/api/README.md)
- [Examples](src/examples/README.md)

## License

MIT
