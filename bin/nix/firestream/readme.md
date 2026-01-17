# Firestream Nix Shell Module System

A modular, composable system for building container initialization scripts and environment configurations using Nix. This system provides reusable bash library functions, environment generation, and application-specific configuration in a type-safe, declarative manner.

## Overview

The Firestream Nix Shell Module System consists of:

1. **Core Library Modules** - Reusable bash functions for common operations
2. **Environment Modules** - Environment variable generation and .env file loading
3. **Application Factory** - Composable application module builder
4. **Main Entry Point** - Single import for the entire system

## Quick Start

```nix
# Import the module system
let
  pkgs = import <nixpkgs> {};
  firestream = import ./bin/nix/firestream { inherit pkgs; };
in

# Create an application module
firestream.mkAppModule {
  name = "my-app";
  version = "1.0.0";
  user = "myuser";
  enabledLibraries = [ "log" "fs" "net" ];
}
```

## Project Structure

```
bin/nix/firestream/
├── default.nix                    # Main entry point - import this
├── lib/                           # Core library modules
│   ├── log.nix                   # Logging functions
│   ├── validations.nix           # Input validation
│   ├── fs.nix                    # Filesystem operations
│   ├── os.nix                    # OS detection
│   ├── net.nix                   # Network utilities
│   ├── service.nix               # Service management
│   ├── file.nix                  # File operations
│   └── persistence.nix           # Data persistence
├── env/                           # Environment modules
│   ├── defaults.nix              # Default environment generation
│   └── file-loader.nix           # .env file loading
├── apps/                          # Application factory
│   └── base.nix                  # Application module builder
├── test-default.nix              # Comprehensive test suite
├── example-kafka-container.nix   # Real-world example
└── README-MAIN.md                # This file
```

## Core Library Modules

### Available Modules

| Module | Description | Key Functions |
|--------|-------------|---------------|
| `log` | Logging utilities | `log_print`, `log_error`, `log_debug` |
| `validations` | Input validation | `validate_not_empty`, `validate_number`, `validate_bool` |
| `fs` | Filesystem operations | `ensure_dir_exists`, `ensure_file_exists` |
| `os` | OS detection | `get_os_type`, `is_debian`, `is_redhat` |
| `net` | Network utilities | `wait_for_port`, `check_host_reachable` |
| `service` | Service management | `is_service_running`, `wait_for_service` |
| `file` | File operations | `file_exists`, `read_file_content` |
| `persistence` | Data persistence | `persist_data`, `load_persisted_data` |

### Usage Example

```nix
let
  firestream = import ./bin/nix/firestream { inherit pkgs; };

  # Access individual library
  logLib = firestream.lib.log;

  # Use in a script
  myScript = pkgs.writeShellScript "my-script.sh" ''
    source ${logLib.script}/opt/firestream/scripts/liblog.sh

    log_print "info" "Starting application..."
  '';
in
myScript
```

## Environment Modules

### mkEnvDefaults

Generates default environment variables for an application:

```nix
firestream.env.mkEnvDefaults {
  appName = "kafka";
  appVersion = "4.0.1";
  user = "kafka";

  # Returns attribute set of default env vars:
  # {
  #   FIRESTREAM_APP_NAME = "kafka";
  #   APP_VERSION = "4.0.1";
  #   FIRESTREAM_ROOT_DIR = "/opt/firestream";
  #   ...
  # }
}
```

### mkFileLoader

Generates bash script to load .env files:

```nix
firestream.env.mkFileLoader {
  envFiles = [
    "/opt/firestream/app/conf/.env"
    "/firestream/app/conf/.env.local"
  ];

  # Returns bash script that sources env files
}
```

## Application Factory

### mkAppModule

The main function to create application modules:

```nix
firestream.mkAppModule {
  # Required
  name = "kafka";
  version = "4.0.1";
  user = "kafka";

  # Optional - which library modules to include
  enabledLibraries = [
    "log"
    "validations"
    "fs"
    "os"
    "net"
    "service"
    "persistence"
  ];

  # Optional - custom environment variables
  customEnv = {
    KAFKA_CFG_ZOOKEEPER_CONNECT = "zookeeper:2181";
    KAFKA_CFG_LISTENERS = "PLAINTEXT://:9092";
  };

  # Optional - .env files to load
  loadEnvFiles = [
    "/opt/firestream/kafka/conf/.env"
  ];
}
```

Returns an attribute set with:

```nix
{
  # Environment variables (merged defaults + custom)
  env = { ... };

  # List of script derivations
  scripts = [ ... ];

  # List of runtime dependencies (packages)
  runtimeInputs = [ pkgs.bash pkgs.coreutils ... ];

  # Bash script to load .env files
  envLoader = "...";

  # Original config
  enabledLibraries = [ ... ];
}
```

## Convenience Functions

### allFunctions

Get all library functions as a single string:

```nix
firestream.allFunctions
# Returns: concatenated bash functions from all modules
```

### allRuntimeDeps

Get all runtime dependencies as a deduplicated list:

```nix
firestream.allRuntimeDeps
# Returns: [ pkgs.bash pkgs.coreutils pkgs.gnugrep ... ]
```

### combinedLibScript

Get a single script file with all library functions:

```nix
firestream.combinedLibScript
# Returns: derivation at /nix/store/.../opt/firestream/scripts/libfirestream.sh
```

## Real-World Example: Kafka Container

See `example-kafka-container.nix` for a complete example of building a Kafka container.

```bash
# Test the example
nix-instantiate --eval example-kafka-container.nix

# View the environment configuration
nix-instantiate --eval -E '(import ./example-kafka-container.nix {}).environment'

# View the metadata
nix-instantiate --eval -E '(import ./example-kafka-container.nix {}).meta'
```

## Integration with Docker Builds

```nix
{ pkgs }:

let
  firestream = import ./bin/nix/firestream { inherit pkgs; };

  appModule = firestream.mkAppModule {
    name = "kafka";
    version = "4.0.1";
    user = "kafka";
    enabledLibraries = [ "log" "fs" "net" "service" ];
  };

in pkgs.dockerTools.buildImage {
  name = "firestream/kafka";
  tag = "4.0.1";

  contents = [
    pkgs.bash
    pkgs.coreutils
  ] ++ appModule.runtimeInputs;

  config = {
    Env = pkgs.lib.mapAttrsToList (k: v: "${k}=${v}") appModule.env;
    User = appModule.env.FIRESTREAM_USER;
    Cmd = [ "${pkgs.bash}/bin/bash" ];
  };

  extraCommands = ''
    mkdir -p opt/firestream/scripts

    # Copy all library scripts
    ${builtins.concatStringsSep "\n" (
      map (script: "cp -r ${script}/* opt/firestream/scripts/") appModule.scripts
    )}
  '';
}
```

## Testing

### Run All Tests

```bash
cd bin/nix/firestream
nix-instantiate --eval test-default.nix
```

### Run Individual Tests

```bash
# Test meta information
nix-instantiate --eval -E '(import ./test-default.nix).test_meta'

# Test library modules
nix-instantiate --eval -E '(import ./test-default.nix).test_lib_modules'

# Test module validation
nix-instantiate --eval -E '(import ./test-default.nix).test_all_modules_valid'

# View summary
nix-instantiate --eval -E '(import ./test-default.nix).summary'
```

## API Reference

### Main Entry Point

```nix
{ pkgs }:

{
  # Core library modules
  lib = {
    log = { functions, exports, script, runtimeDeps };
    validations = { ... };
    fs = { ... };
    os = { ... };
    net = { ... };
    service = { ... };
    file = { ... };
    persistence = { ... };
  };

  # Environment generation functions
  env = {
    mkEnvDefaults = { appName, appVersion, user, ... } -> attrset;
    mkFileLoader = { envFiles } -> string;
  };

  # Application module factory
  mkAppModule = { name, version, user, ... } -> {
    env = attrset;
    scripts = [ derivation ];
    runtimeInputs = [ package ];
    envLoader = string;
    enabledLibraries = [ string ];
  };

  # Convenience exports
  allFunctions = string;
  allRuntimeDeps = [ package ];
  combinedLibScript = derivation;

  # Meta information
  meta = {
    name = "firestream-nix-modules";
    version = "1.0.0";
    description = string;
    moduleCount = int;
    modules = [ string ];
  };
}
```

## Design Principles

1. **Composability** - Mix and match modules as needed
2. **Type Safety** - Nix's type system ensures correctness
3. **Dependency Injection** - Modules receive only what they need
4. **Immutability** - All outputs are immutable derivations
5. **Testability** - Comprehensive test coverage
6. **Reusability** - Use across multiple container types
7. **Maintainability** - Clear separation of concerns

## Benefits

1. **Consistency** - Same library functions across all containers
2. **Maintainability** - Update once, apply everywhere
3. **Type Safety** - Nix catches errors at build time
4. **Reproducibility** - Exact same output every time
5. **Documentation** - Self-documenting through types
6. **Testing** - Easy to test individual modules
7. **Version Control** - Everything is in Git

## Development Phases

- ✅ Phase 1: Core Utility Modules (log, validations, fs)
- ✅ Phase 2: Core Utility Modules (os, net, service)
- ✅ Phase 3: Core Utility Modules (file, persistence)
- ✅ Phase 4: Core Library Tests
- ✅ Phase 5: Environment Modules
- ✅ Phase 6: Application Factory
- ✅ Phase 7: Main Entry Point (default.nix)

## Next Steps

1. **Container Integration** - Use in actual container builds
2. **Performance Optimization** - Profile and optimize
3. **Additional Modules** - Add more specialized modules
4. **Documentation** - User guides and tutorials
5. **CI/CD** - Automated testing in GitHub Actions

## Contributing

When adding new modules:

1. Follow the established module structure
2. Include comprehensive tests
3. Update documentation
4. Add examples
5. Ensure dependency injection is correct

## License

Apache 2.0 - See LICENSE file for details

## Support

For issues or questions:
- GitHub Issues: https://github.com/Cogent-Creation-Co/Firestream
- Documentation: See individual PHASE-COMPLETE.md files
