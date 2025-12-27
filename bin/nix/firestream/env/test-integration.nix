# Integration Test for Environment Modules
# This demonstrates the complete usage pattern combining defaults + file loader

{ pkgs ? import <nixpkgs> {}, lib ? pkgs.lib }:

let
  defaults = import ./defaults.nix { inherit pkgs lib; };
  fileLoader = import ./file-loader.nix { inherit pkgs lib; };

  # Create environment components for Kafka
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

  # Create a complete entrypoint that uses both
  kafkaEntrypoint = pkgs.writeShellScript "kafka-entrypoint.sh" ''
    #!/bin/bash
    set -euo pipefail

    echo "=== Kafka Container Entrypoint ==="
    echo ""

    # Load environment defaults
    echo "Loading environment defaults..."
    source ${kafkaEnvDefaults}/lib/kafka-env-defaults.sh

    # Load secrets from files
    echo "Loading secrets from files..."
    source ${kafkaFileLoader}/lib/kafka-load-env-files.sh

    # Display environment (for testing)
    echo ""
    echo "=== Environment Configuration ==="
    echo "MODULE: $MODULE"
    echo "KAFKA_BASE_DIR: $KAFKA_BASE_DIR"
    echo "KAFKA_CONF_DIR: $KAFKA_CONF_DIR"
    echo "KAFKA_DATA_DIR: $KAFKA_DATA_DIR"
    echo "KAFKA_LOG_DIR: $KAFKA_LOG_DIR"
    echo "KAFKA_HEAP_OPTS: $KAFKA_HEAP_OPTS"
    echo "KAFKA_CFG_PROCESS_ROLES: $KAFKA_CFG_PROCESS_ROLES"
    echo "KAFKA_DAEMON_USER: $KAFKA_DAEMON_USER"
    echo ""

    # Check if password was loaded
    if [[ -n "''${KAFKA_PASSWORD:-}" ]]; then
      echo "KAFKA_PASSWORD: [LOADED FROM FILE]"
    else
      echo "KAFKA_PASSWORD: [NOT SET]"
    fi

    echo ""
    echo "=== Ready to start Kafka ==="
    # In real usage: exec kafka-server-start.sh "$@"
  '';

  # Create a test runner that simulates Docker secrets
  testRunner = pkgs.writeShellScript "test-env-modules.sh" ''
    #!/bin/bash
    set -euo pipefail

    echo "=== Testing Environment Modules ==="
    echo ""

    # Create temporary secret files
    TEMP_DIR=$(mktemp -d)
    trap "rm -rf $TEMP_DIR" EXIT

    echo "supersecret123" > "$TEMP_DIR/kafka_password"
    echo "interbroker456" > "$TEMP_DIR/inter_broker_password"

    echo "Created test secret files in $TEMP_DIR"
    echo ""

    # Set environment for file loading
    export KAFKA_PASSWORD_FILE="$TEMP_DIR/kafka_password"
    export KAFKA_INTER_BROKER_PASSWORD_FILE="$TEMP_DIR/inter_broker_password"

    # Override some defaults
    export KAFKA_CFG_PROCESS_ROLES="broker,controller"
    export FIRESTREAM_DEBUG="true"

    # Run the entrypoint
    ${kafkaEntrypoint}

    echo ""
    echo "=== Test Completed Successfully ==="
  '';

in {
  # Export components for inspection
  inherit kafkaEnvDefaults kafkaFileLoader kafkaEntrypoint testRunner;

  # Convenience test runner
  test = testRunner;
}
