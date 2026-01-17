# Example: Building a Kafka container using the Firestream module system
# This demonstrates a real-world usage pattern for container builds

{ pkgs ? import <nixpkgs> {} }:

let
  # Import the Firestream module system
  firestream = import ./default.nix { inherit pkgs; };

  # Create a Kafka application module
  kafkaModule = firestream.mkAppModule {
    name = "kafka";
    version = "4.0.1";
    user = "kafka";

    # Enable all necessary library modules
    enabledLibraries = [
      "log"           # Logging functions
      "validations"   # Input validation
      "fs"            # Filesystem operations
      "os"            # OS detection
      "net"           # Network utilities
      "service"       # Service management
      "persistence"   # Data persistence
    ];

    # Custom environment variables for Kafka
    customEnv = {
      # Kafka Configuration
      KAFKA_CFG_ZOOKEEPER_CONNECT = "zookeeper:2181";
      KAFKA_CFG_LISTENERS = "PLAINTEXT://:9092";
      KAFKA_CFG_ADVERTISED_LISTENERS = "PLAINTEXT://localhost:9092";
      KAFKA_CFG_LOG_DIRS = "/firestream/kafka/data";
      KAFKA_CFG_AUTO_CREATE_TOPICS_ENABLE = "true";
      KAFKA_CFG_LOG_RETENTION_HOURS = "168";
      KAFKA_CFG_NUM_PARTITIONS = "3";
      KAFKA_CFG_DEFAULT_REPLICATION_FACTOR = "1";

      # Firestream paths
      FIRESTREAM_APP_NAME = "kafka";
      FIRESTREAM_IMAGE_VERSION = "4.0.1";
    };

    # Load environment from files
    loadEnvFiles = [
      "/opt/firestream/kafka/conf/.env"
      "/firestream/kafka/conf/.env.local"
    ];
  };

  # Create entrypoint script that uses the library functions
  entrypointScript = pkgs.writeTextFile {
    name = "entrypoint.sh";
    executable = true;
    text = ''
      #!/bin/bash
      set -e

      # Source the Firestream library functions
      source /opt/firestream/scripts/liblog.sh
      source /opt/firestream/scripts/libfs.sh
      source /opt/firestream/scripts/libos.sh
      source /opt/firestream/scripts/libservice.sh
      source /opt/firestream/scripts/libkafka-env.sh

      # Load environment from files
      ${kafkaModule.envLoader}

      # Log startup
      log_print "info" "Starting Kafka ${kafkaModule.env.KAFKA_VERSION or "4.0.1"}..."

      # Ensure directories exist
      ensure_dir_exists "/firestream/kafka/data"
      ensure_dir_exists "/opt/firestream/kafka/logs"

      # Validate configuration
      if [[ -z "$KAFKA_CFG_ZOOKEEPER_CONNECT" ]]; then
        log_print "error" "KAFKA_CFG_ZOOKEEPER_CONNECT is required"
        exit 1
      fi

      # Start Kafka
      log_print "info" "Starting Kafka broker..."
      exec /opt/firestream/kafka/bin/kafka-server-start.sh \
        /opt/firestream/kafka/config/server.properties
    '';
  };

in

# Example 1: Just the module for inspection
{
  # The application module
  module = kafkaModule;

  # Individual components
  environment = kafkaModule.env;
  scripts = kafkaModule.scripts;
  runtimeInputs = kafkaModule.runtimeInputs;

  # The entrypoint
  entrypoint = entrypointScript;

  # Demonstrate convenience features
  allFunctions = firestream.allFunctions;
  allDeps = firestream.allRuntimeDeps;

  # Example 2: Build a Docker image (commented out - requires Linux)
  # dockerImage = pkgs.dockerTools.buildImage {
  #   name = "firestream/kafka";
  #   tag = "4.0.1-debian-12";
  #
  #   contents = [
  #     pkgs.bash
  #     pkgs.coreutils
  #     pkgs.findutils
  #   ] ++ kafkaModule.runtimeInputs;
  #
  #   config = {
  #     Env = pkgs.lib.mapAttrsToList (k: v: "${k}=${v}") kafkaModule.env;
  #     User = "kafka";
  #     WorkingDir = "/opt/firestream/kafka";
  #     Cmd = [ "${entrypointScript}" ];
  #     ExposedPorts = {
  #       "9092/tcp" = {};
  #     };
  #     Volumes = {
  #       "/firestream/kafka" = {};
  #     };
  #   };
  #
  #   extraCommands = ''
  #     # Create directory structure
  #     mkdir -p opt/firestream/scripts
  #     mkdir -p opt/firestream/kafka/{bin,config,data,logs}
  #     mkdir -p firestream/kafka/{data,conf}
  #
  #     # Copy library scripts
  #     ${builtins.concatStringsSep "\n" (
  #       map (script: "cp -r ${script}/* opt/firestream/scripts/") kafkaModule.scripts
  #     )}
  #
  #     # Copy entrypoint
  #     cp ${entrypointScript} opt/firestream/scripts/entrypoint.sh
  #     chmod +x opt/firestream/scripts/entrypoint.sh
  #
  #     # Set ownership (when building as root)
  #     # chown -R 1001:1001 opt/firestream firestream
  #   '';
  # };

  # Metadata
  meta = {
    description = "Kafka container built with Firestream module system";
    kafkaVersion = "4.0.1";
    debianVersion = "12";
    firestreamModules = kafkaModule.enabledLibraries;
  };
}
