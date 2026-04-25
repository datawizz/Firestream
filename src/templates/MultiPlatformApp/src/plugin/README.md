# Plugins

Extensible plugin system for the application.

## Purpose

Plugins allow extending the application's functionality without modifying core code. Each plugin is a self-contained module that implements a defined interface.

## Structure

```
src/plugin/
├── README.md           # This file
├── plugin-interface/   # Core plugin interfaces (shared library)
│   └── src/
│       └── index.ts
├── example-plugin/     # Example plugin implementation
│   ├── package.json
│   └── src/
│       └── index.ts
└── another-plugin/
    └── ...
```

## Plugin Interface

Plugins must implement the `Plugin` interface defined in `@multi-platform-app/shared`:

```typescript
interface Plugin {
  name: string;
  version: string;

  // Lifecycle hooks
  onInit(): Promise<void>;
  onDestroy(): Promise<void>;

  // Optional hooks
  onActivate?(): Promise<void>;
  onDeactivate?(): Promise<void>;
}
```

## Creating a Plugin

1. Create a new directory under `src/plugin/`
2. Initialize a new package:
   ```bash
   cd src/plugin/my-plugin
   pnpm init
   ```
3. Add to `pnpm-workspace.yaml`
4. Implement the Plugin interface
5. Register in the plugin registry

## Plugin Discovery

Plugins are discovered at runtime by:
1. Scanning the `src/plugin/` directory
2. Loading each plugin's manifest (`package.json`)
3. Dynamically importing the plugin module

## Best Practices

1. **Isolation**: Plugins should be isolated and not affect other plugins
2. **Versioning**: Use semver for plugin versions
3. **Dependencies**: Minimize external dependencies
4. **Error Handling**: Gracefully handle errors without crashing the host
5. **Documentation**: Include README and API documentation
