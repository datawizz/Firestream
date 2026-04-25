# Protocol Buffer Definitions

This directory contains Protocol Buffer (.proto) definitions for type-safe cross-platform communication.

## Structure

```
proto/
└── example/
    └── v1/
        └── messages.proto
```

## Adding New Definitions

1. Create a new directory: `proto/your-feature/v1/`
2. Add your proto file: `your-feature.proto`
3. Run `make proto-generate` to generate TypeScript types

## Generated Types

TypeScript types are generated in:
```
src/lib/typescript/proto-types/src/generated/
```

## Usage

```typescript
import { ExampleRequest } from '@multi-platform-app/proto-types';

const request = new ExampleRequest({
  id: '123',
  data: 'Hello',
  timestamp: BigInt(Date.now()),
});
```

## Versioning

Use versioned packages (v1, v2, etc.) to maintain backwards compatibility.
