# Examples

This directory contains example code demonstrating the platform adapter pattern and other features of the multi-platform app template.

## Files

- **basic-example.tsx** - Simple platform adapter usage
- **query-example.tsx** - TanStack Query integration

## Usage

These examples are for reference only. Copy code from these files into your applications to see the patterns in action.

## Platform Adapter Pattern

The key to multi-platform development is the adapter pattern:

1. Define an interface (`IStorageService`)
2. Create platform-specific implementations (Tauri, Web)
3. Use a service factory to load the correct adapter at runtime
4. Write platform-agnostic code that uses the interface

See `basic-example.tsx` for a complete working example.
