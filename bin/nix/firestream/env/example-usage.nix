# Example Usage of Environment Modules
# This file demonstrates how to use the env modules for different applications

{ pkgs ? import <nixpkgs> {}, lib ? pkgs.lib }:

let
  # Import the environment modules
  defaults = import ./defaults.nix { inherit pkgs lib; };
  fileLoader = import ./file-loader.nix { inherit pkgs lib; };

in {
  # Example 1: Kafka environment
  kafka = {
    envDefaults = defaults.mkEnvDefaults {
      appName = "kafka";
      envVars = {
        KAFKA_HEAP_OPTS = "-Xmx1024m -Xms1024m";
        KAFKA_CFG_PROCESS_ROLES = "";
        KAFKA_CFG_NODE_ID = "0";
        KAFKA_CFG_CONTROLLER_QUORUM_VOTERS = "";
      };
      paths = {
        base = "/opt/firestream/kafka";
        conf = "/opt/firestream/kafka/config";
        data = "/firestream/kafka/data";
        logs = "/opt/firestream/kafka/logs";
      };
      user = {
        name = "kafka";
        group = "kafka";
      };
    };

    fileLoader = fileLoader.mkFileLoader {
      appName = "kafka";
      secretVars = [
        "KAFKA_PASSWORD"
        "KAFKA_INTER_BROKER_PASSWORD"
        "KAFKA_CLIENT_PASSWORD"
        "KAFKA_ZOOKEEPER_PASSWORD"
      ];
    };
  };

  # Example 2: PostgreSQL environment
  postgresql = {
    envDefaults = defaults.mkEnvDefaults {
      appName = "postgresql";
      envVars = {
        POSTGRESQL_PORT_NUMBER = "5432";
        POSTGRESQL_VOLUME_DIR = "/firestream/postgresql";
        POSTGRESQL_DATA_DIR = "/firestream/postgresql/data";
        POSTGRESQL_ENABLE_LDAP = "no";
        POSTGRESQL_ENABLE_TLS = "no";
      };
      paths = {
        base = "/opt/firestream/postgresql";
        conf = "/opt/firestream/postgresql/conf";
        data = "/firestream/postgresql/data";
        logs = "/opt/firestream/postgresql/logs";
      };
      user = {
        name = "postgres";
        group = "postgres";
      };
    };

    fileLoader = fileLoader.mkFileLoader {
      appName = "postgresql";
      secretVars = [
        "POSTGRESQL_PASSWORD"
        "POSTGRESQL_POSTGRES_PASSWORD"
        "POSTGRESQL_REPLICATION_PASSWORD"
      ];
    };
  };

  # Example 3: Redis environment
  redis = {
    envDefaults = defaults.mkEnvDefaults {
      appName = "redis";
      envVars = {
        REDIS_PORT_NUMBER = "6379";
        REDIS_DISABLE_COMMANDS = "";
        REDIS_DATABASE = "0";
        REDIS_AOF_ENABLED = "yes";
      };
      paths = {
        base = "/opt/firestream/redis";
        conf = "/opt/firestream/redis/mounted-etc";
        data = "/firestream/redis/data";
        logs = "/opt/firestream/redis/logs";
      };
      user = {
        name = "redis";
        group = "redis";
      };
    };

    fileLoader = fileLoader.mkFileLoader {
      appName = "redis";
      secretVars = [
        "REDIS_PASSWORD"
      ];
    };
  };

  # Example 4: Airflow environment
  airflow = {
    envDefaults = defaults.mkEnvDefaults {
      appName = "airflow";
      envVars = {
        AIRFLOW_EXECUTOR = "SequentialExecutor";
        AIRFLOW_FERNET_KEY = "";
        AIRFLOW_SECRET_KEY = "";
        AIRFLOW_WEBSERVER_PORT_NUMBER = "8080";
        AIRFLOW_LOAD_EXAMPLES = "no";
      };
      paths = {
        base = "/opt/firestream/airflow";
        conf = "/opt/firestream/airflow";
        data = "/firestream/airflow";
        logs = "/opt/firestream/airflow/logs";
      };
      user = {
        name = "airflow";
        group = "airflow";
      };
    };

    fileLoader = fileLoader.mkFileLoader {
      appName = "airflow";
      secretVars = [
        "AIRFLOW_PASSWORD"
        "AIRFLOW_FERNET_KEY"
        "AIRFLOW_SECRET_KEY"
        "REDIS_PASSWORD"
        "AIRFLOW_DATABASE_PASSWORD"
      ];
    };
  };
}
