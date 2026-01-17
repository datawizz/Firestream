# Java Container Module Factory
# Copyright Firestream. Apache-2.0 License.
#
# This module provides mkJavaContainerModule - a specialized factory for
# Java applications with JVM tuning, NSS wrapper support, and classpath management.
#
# Features:
# - Automatic JAVA_HOME configuration
# - JVM tuning options (heap, GC, JMX)
# - NSS wrapper for Kubernetes arbitrary UID support
# - Classpath and JAR directory management
# - Java dev shell with Maven/Gradle
#
# Usage:
#   let
#     container = mkJavaContainerModule {
#       name = "kafka";
#       version = "4.0.1";
#       jdk = pkgs.temurin-bin-17;
#       heapOpts = "-Xmx2g -Xms1g";
#       jarDirs = [ "/opt/kafka/libs" ];
#       runCmd = "kafka-server-start.sh ...";
#     };
#   in container.dockerImage

{ pkgs, lib, mkContainerModule }:

{
  # Factory function for Java container modules
  mkJavaContainerModule = {
    # Basic metadata
    name,
    version ? "1.0.0",

    # Java configuration
    jdk ? pkgs.temurin-bin-17,   # Default JDK (Eclipse Temurin 17 LTS)
    javaHome ? null,             # Override JAVA_HOME (defaults to jdk package path)

    # JVM tuning options
    heapOpts ? "-Xmx1024m -Xms512m",  # Default heap: 1GB max, 512MB initial
    gcOpts ? "-XX:+UseG1GC -XX:MaxGCPauseMillis=20 -XX:InitiatingHeapOccupancyPercent=35",
    jmxOpts ? "",                # JMX configuration (disabled by default)
    extraJavaOpts ? "",          # Additional JAVA_OPTS

    # Classpath management
    classPath ? [],              # Explicit paths to include in CLASSPATH
    jarDirs ? [],                # Directories containing JARs (appended as dir/*)

    # Environment configuration
    envVars ? {},
    envVarsWithSecrets ? [],

    # Paths configuration
    paths ? {
      base = "/opt/firestream/${name}";
      conf = "/opt/firestream/${name}/config";
      data = "/firestream/${name}/data";
      logs = "/opt/firestream/${name}/logs";
    },

    # User/group configuration
    user ? {
      name = name;
      group = name;
      uid = 1001;
      gid = 1001;
    },

    # Application-specific shell functions
    validateFn ? "",
    configFn ? "",
    initFn ? "",
    runCmd ? "",

    # Container dependencies
    systemDeps ? [],
    runtimeBinDeps ? [],
    extraDeps ? [],

    # Docker configuration
    dockerConfig ? {},
    exposedPorts ? [],
    volumes ? [],

    # Build-time (prepopulate phase)
    prepopulateFn ? "",
    prepopulateFiles ? {},
    prepopulateDirs ? [],
    runtimeDirs ? {},

    # Runtime (activate phase)
    activateFn ? "",
    enableStateTracking ? true,

    # NSS wrapper configuration
    enableNssWrapper ? true,     # Enable NSS wrapper for arbitrary UID support

    # Custom scripts and hooks
    customScripts ? {},
    devShellPackages ? [],
    devShellHook ? "",
  }:
  let
    # Resolve JAVA_HOME
    resolvedJavaHome = if javaHome != null then javaHome else "${jdk}";

    # Build combined JAVA_OPTS
    javaOpts = lib.concatStringsSep " " (lib.filter (s: s != "") [
      heapOpts
      gcOpts
      jmxOpts
      extraJavaOpts
    ]);

    # Build CLASSPATH from jarDirs and explicit paths
    buildClassPath = jarDirs: classPath:
      let
        jarPaths = map (d: "${d}/*") jarDirs;
        allPaths = classPath ++ jarPaths;
      in lib.concatStringsSep ":" allPaths;

    resolvedClassPath = buildClassPath jarDirs classPath;

    # NSS wrapper helper functions
    nssWrapperHelpers = ''
      ########################
      # Enable NSS wrapper for arbitrary UID support
      # This is the Bitnami pattern for Kubernetes where pods may run
      # as arbitrary UIDs that don't exist in /etc/passwd
      # Globals:
      #   NSS_WRAPPER_LIB, NSS_WRAPPER_PASSWD, NSS_WRAPPER_GROUP
      # Arguments:
      #   None
      # Returns:
      #   None
      #########################
      java_enable_nss_wrapper() {
          local nss_wrapper_lib="''${NSS_WRAPPER_LIB:-/opt/bitnami/common/lib/libnss_wrapper.so}"

          # Only enable if we can't resolve our UID and the wrapper exists
          if ! getent passwd "$(id -u)" &>/dev/null && [[ -e "$nss_wrapper_lib" ]]; then
              debug "Enabling NSS wrapper for UID $(id -u)"

              export LD_PRELOAD="$nss_wrapper_lib"
              export NSS_WRAPPER_PASSWD="$(mktemp)"
              export NSS_WRAPPER_GROUP="$(mktemp)"

              # Create temporary passwd file with our UID
              echo "${user.name}:x:$(id -u):$(id -g):${user.name}:${paths.base}:/bin/bash" > "$NSS_WRAPPER_PASSWD"
              echo "${user.group}:x:$(id -g):" > "$NSS_WRAPPER_GROUP"

              debug "NSS wrapper enabled: passwd=$NSS_WRAPPER_PASSWD group=$NSS_WRAPPER_GROUP"
          fi
      }
    '';

    # Java-specific system dependencies
    javaSystemDeps = with pkgs; [
      # Java runtime
      jdk
      # SSL/TLS
      cacert openssl
      # Compression (many Java apps need these)
      zlib gzip bzip2 xz zstd
      # Process management
      procps
      # C++ runtime (for native libs)
      stdenv.cc.cc.lib
    ] ++ lib.optionals enableNssWrapper [
      # NSS wrapper for Kubernetes arbitrary UID support
      pkgs.nss_wrapper
    ];

    # Merge with user-provided systemDeps
    allSystemDeps = lib.lists.unique (javaSystemDeps ++ systemDeps);

    # Java runtime binary dependencies
    javaRuntimeBins = [ jdk ] ++ runtimeBinDeps;

    # Combine base prepopulateFn with Java-specific setup
    combinedPrepopulateFn = ''
      ${prepopulateFn}

      # Java-specific prepopulation
      # Ensure JAVA_HOME points to valid location
      if [[ ! -d "${resolvedJavaHome}" ]]; then
        warn "JAVA_HOME directory does not exist: ${resolvedJavaHome}"
      fi
    '';

    # Enhanced activateFn with NSS wrapper
    combinedActivateFn = ''
      ${lib.optionalString enableNssWrapper ''
      # Enable NSS wrapper if running as arbitrary UID
      java_enable_nss_wrapper
      ''}

      ${activateFn}
    '';

    # Enhanced validateFn with Java helpers
    combinedValidateFn = ''
      ${nssWrapperHelpers}

      # Validate Java installation
      if [[ ! -x "${resolvedJavaHome}/bin/java" ]]; then
          error "Java executable not found at ${resolvedJavaHome}/bin/java"
          return 1
      fi

      # Log Java version for debugging
      debug "Java version: $(${jdk}/bin/java -version 2>&1 | head -1)"

      ${validateFn}
    '';

    # Merge Java environment variables with user-provided ones
    javaEnvVars = {
      # Java paths
      JAVA_HOME = resolvedJavaHome;

      # JVM options (can be overridden at runtime)
      JAVA_OPTS = javaOpts;
      JAVA_HEAP_OPTS = heapOpts;
      JAVA_GC_OPTS = gcOpts;
    } // lib.optionalAttrs (jmxOpts != "") {
      JAVA_JMX_OPTS = jmxOpts;
    } // lib.optionalAttrs (resolvedClassPath != "") {
      CLASSPATH = resolvedClassPath;
    } // lib.optionalAttrs enableNssWrapper {
      # NSS wrapper paths (Bitnami compatibility)
      NSS_WRAPPER_LIB = "/opt/bitnami/common/lib/libnss_wrapper.so";
    } // envVars;

    # Create Java entrypoint wrapper
    javaEntrypointWrapper = entrypoint: pkgs.runCommand "${name}-java-entrypoint" {
      nativeBuildInputs = [ pkgs.makeWrapper ];
    } ''
      mkdir -p $out/bin
      makeWrapper ${entrypoint}/bin/${name}-entrypoint $out/bin/${name}-entrypoint \
        --set JAVA_HOME "${resolvedJavaHome}" \
        --set JAVA_OPTS "${javaOpts}" \
        --set PATH "${lib.makeBinPath javaRuntimeBins}:$PATH" \
        --set LD_LIBRARY_PATH "${lib.makeLibraryPath [ pkgs.stdenv.cc.cc.lib ]}:''${LD_LIBRARY_PATH:-}" \
        --set SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" \
        --set NIX_SSL_CERT_FILE "${pkgs.cacert}/etc/ssl/certs/ca-bundle.crt" \
        ${lib.optionalString (resolvedClassPath != "") ''--set CLASSPATH "${resolvedClassPath}"''}
    '';

    # Enhanced Docker config with Java environment
    javaDockerConfig = lib.recursiveUpdate {
      Env = [
        "PATH=${jdk}/bin:/bin:/usr/bin"
        "JAVA_HOME=${resolvedJavaHome}"
        "JAVA_OPTS=${javaOpts}"
      ] ++ lib.optionals (resolvedClassPath != "") [
        "CLASSPATH=${resolvedClassPath}"
      ];
    } dockerConfig;

    # Create the container module
    containerModule = mkContainerModule {
      inherit name version paths user;
      envVars = javaEnvVars;
      inherit envVarsWithSecrets;

      validateFn = combinedValidateFn;
      inherit configFn;
      inherit initFn runCmd;

      # Pass through two-phase lifecycle parameters
      prepopulateFn = combinedPrepopulateFn;
      inherit prepopulateFiles prepopulateDirs runtimeDirs;
      activateFn = combinedActivateFn;
      inherit enableStateTracking;

      systemDeps = allSystemDeps;
      runtimeBinDeps = javaRuntimeBins;
      inherit extraDeps;

      dockerConfig = javaDockerConfig;
      inherit exposedPorts;
      volumes = volumes ++ lib.optionals (jarDirs != []) jarDirs;

      entrypointWrapper = javaEntrypointWrapper;

      inherit customScripts;
      devShellPackages = devShellPackages ++ [
        jdk
        pkgs.maven
        pkgs.gradle
      ];
      devShellHook = ''
        export JAVA_HOME="${resolvedJavaHome}"
        export JAVA_OPTS="${javaOpts}"
        ${lib.optionalString (resolvedClassPath != "") ''export CLASSPATH="${resolvedClassPath}"''}
        echo "Java Development Environment"
        echo "  JDK: ${jdk.name}"
        echo "  JAVA_HOME: ${resolvedJavaHome}"
        echo "  Heap: ${heapOpts}"
        echo "  GC: ${gcOpts}"
        echo ""
        ${devShellHook}
      '';
    };

  in containerModule // {
    # Add Java-specific attributes
    inherit jdk resolvedJavaHome javaOpts;

    # Export Java-specific config
    config = (containerModule.config or {}) // {
      inherit heapOpts gcOpts jmxOpts extraJavaOpts;
      inherit classPath jarDirs resolvedClassPath;
      inherit enableNssWrapper;
    };

    # Override meta with Java info
    meta = containerModule.meta // {
      jdkVersion = jdk.version or "unknown";
      jdkName = jdk.name or "unknown";
      heapConfiguration = heapOpts;
      gcConfiguration = gcOpts;
      nssWrapperEnabled = enableNssWrapper;
    };
  };
}
