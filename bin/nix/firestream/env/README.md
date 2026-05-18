# Environment Generation Modules

This directory contains Nix modules for generating environment configuration scripts for Firestream applications. These modules create shell scripts that can be sourced to set up application environments with proper defaults and Docker secrets support.

## Modules

### defaults.nix

Generates environment variable default scripts for applications.

**Function:** `mkEnvDefaults`

**Parameters:**
- `appName` (string, required): Application name (e.g., "kafka", "postgresql")
- `envVars` (attrset, optional): Environment variables with default values
- `paths` (attrset, optional): Application directory paths
  - `base`: Base directory (default: `/opt/firestream/${appName}`)
  - `conf`: Configuration directory (default: `/opt/firestream/${appName}/config`)
  - `data`: Data directory (default: `/firestream/${appName}/data`)
  - `logs`: Logs directory (default: `/opt/firestream/${appName}/logs`)
- `user` (attrset, optional): Daemon user/group
  - `name`: User name (default: `appName`)
  - `group`: Group name (default: `appName`)

**Returns:** A derivation containing a shell script at `lib/${appName}-env-defaults.sh`

**Features:**
- Sets standard Firestream directory structure
- Configurable debug settings
- Application-specific path exports (uppercase app name)
- Environment variables with fallback defaults using `${VAR:-default}` pattern

**Example:**
```nix
let
  defaults = import ./defaults.nix { inherit pkgs lib; };

  kafkaEnv = defaults.mkEnvDefaults {
    appName = "kafka";
    envVars = {
      KAFKA_HEAP_OPTS = "-Xmx1024m -Xms1024m";
      KAFKA_CFG_PROCESS_ROLES = "";
    };
  };
in
  kafkaEnv  # => /nix/store/...-kafka-env-defaults.sh
```

**Generated Script Usage:**
```bash
source /nix/store/...-kafka-env-defaults.sh/lib/kafka-env-defaults.sh
# Now KAFKA_BASE_DIR, KAFKA_HEAP_OPTS, etc. are set
```

### file-loader.nix

Generates Docker secrets loader scripts that read environment variables from files.

**Function:** `mkFileLoader`

**Parameters:**
- `appName` (string, required): Application name (e.g., "kafka", "postgresql")
- `secretVars` (list of strings, optional): Variable names that support `_FILE` pattern

**Returns:** A derivation containing a shell script at `lib/${appName}-load-env-files.sh`

**Features:**
- Implements Docker secrets pattern: `VAR_FILE` → reads file → sets `VAR`
- Checks file readability before loading
- Warns if secret files are not readable
- Unsets `_FILE` variables after loading
- Cleans up temporary array variables

**Example:**
```nix
let
  fileLoader = import ./file-loader.nix { inherit pkgs lib; };

  kafkaLoader = fileLoader.mkFileLoader {
    appName = "kafka";
    secretVars = [ "KAFKA_PASSWORD" "KAFKA_INTER_BROKER_PASSWORD" ];
  };
in
  kafkaLoader  # => /nix/store/...-kafka-load-env-files.sh
```

**Generated Script Usage:**
```bash
# Set file path
export KAFKA_PASSWORD_FILE=/run/secrets/kafka_password

# Source the loader
source /nix/store/...-kafka-load-env-files.sh/lib/kafka-load-env-files.sh

# Now KAFKA_PASSWORD contains the contents of /run/secrets/kafka_password
# and KAFKA_PASSWORD_FILE is unset
```

## Usage Pattern

A typical application environment setup combines both modules:

```nix
{ pkgs, lib }:

let
  defaults = import ./env/defaults.nix { inherit pkgs lib; };
  fileLoader = import ./env/file-loader.nix { inherit pkgs lib; };

  # Create environment scripts
  kafkaEnvDefaults = defaults.mkEnvDefaults {
    appName = "kafka";
    envVars = {
      KAFKA_HEAP_OPTS = "-Xmx1024m -Xms1024m";
      KAFKA_CFG_PROCESS_ROLES = "";
    };
  };

  kafkaFileLoader = fileLoader.mkFileLoader {
    appName = "kafka";
    secretVars = [ "KAFKA_PASSWORD" "KAFKA_INTER_BROKER_PASSWORD" ];
  };

  # Combine into entrypoint
  entrypoint = pkgs.writeShellScript "kafka-entrypoint.sh" ''
    #!/bin/bash
    set -e

    # Load defaults
    source ${kafkaEnvDefaults}/lib/kafka-env-defaults.sh

    # Load secrets from files
    source ${kafkaFileLoader}/lib/kafka-load-env-files.sh

    # Start application
    exec kafka-server-start.sh "$@"
  '';
in
  entrypoint
```

## Environment Variable Naming Conventions

### Standard Variables (set by defaults.nix)

All applications get these standard variables:
- `FIRESTREAM_ROOT_DIR`: `/opt/firestream`
- `FIRESTREAM_VOLUME_DIR`: `/firestream`
- `MODULE`: Application name

### Debug Variables

- `FIRESTREAM_DEBUG`: Debug mode (default: `false`)
- `FIRESTREAM_QUIET`: Quiet mode (default: `false`)
- `FIRESTREAM_COLOR`: Color output (default: `true`)

### Application-Specific Variables

For an application named `kafka` (converted to uppercase):
- `KAFKA_BASE_DIR`: Base installation directory
- `KAFKA_CONF_DIR`: Configuration directory
- `KAFKA_DATA_DIR`: Data directory
- `KAFKA_LOG_DIR`: Logs directory
- `KAFKA_HOME`: Home directory (same as BASE_DIR)
- `KAFKA_DAEMON_USER`: Daemon user name
- `KAFKA_DAEMON_GROUP`: Daemon group name

## Docker Secrets Pattern

The file loader implements the standard Docker secrets pattern:

1. Set `VAR_FILE` to path of secret file: `KAFKA_PASSWORD_FILE=/run/secrets/password`
2. Source the loader script: `source kafka-load-env-files.sh`
3. Script reads file contents into `VAR`: `KAFKA_PASSWORD=$(cat /run/secrets/password)`
4. Script unsets `VAR_FILE` for security
5. Application uses `VAR` normally

This allows passwords and sensitive data to be stored in files (Docker/Kubernetes secrets) rather than environment variables.

## Testing

See `example-usage.nix` for comprehensive examples of Kafka, PostgreSQL, Redis, and Airflow configurations.

To test:
```bash
# Build example
nix-build example-usage.nix -A kafka.envDefaults

# View generated script
cat $(nix-build example-usage.nix -A kafka.envDefaults --no-out-link)/lib/kafka-env-defaults.sh

# Test file loader
cat $(nix-build example-usage.nix -A kafka.fileLoader --no-out-link)/lib/kafka-load-env-files.sh
```

## Integration with Applications

These modules are designed to integrate with the `apps/` directory where individual application Nix modules will use them to generate their entrypoint scripts and environment configurations.

Example application module structure:
```
apps/
  kafka/
    default.nix       # Main module using env modules
    config.nix        # Application-specific configuration
    entrypoint.nix    # Uses env defaults + file loader
```

## Copyright

Copyright Firestream. MIT License.
