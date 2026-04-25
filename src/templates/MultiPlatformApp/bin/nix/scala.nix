# scala.nix - Scala development environment
{ pkgs, system ? null, fenixPackages ? null, firestreamLib ? null }:

let
  # Java and Scala versions
  jdk = pkgs.jdk11;
  scala = pkgs.scala_2_13;
  sbt = pkgs.sbt.override { jre = jdk; };

  # Scala packages
  scalaPackages = with pkgs; [
    jdk
    scala
    sbt
    scalafmt
    metals
    maven
  ];

  # Environment variables for Scala
  envVars = ''
    # Java environment
    export JAVA_HOME="${jdk}"
    export PATH="$JAVA_HOME/bin:$PATH"

    # Scala environment
    export SCALA_HOME="${scala}"
    export PATH="$SCALA_HOME/bin:$PATH"

    # SBT options
    export SBT_OPTS="-Xmx2G -XX:+UseG1GC -XX:ReservedCodeCacheSize=256m"

    # Ivy and Coursier cache directories
    export IVY_HOME="$HOME/.ivy2"
    export COURSIER_CACHE="$HOME/.cache/coursier"
  '';

  # Profile script for Scala setup
  profileScript = ''
    ${envVars}

    # Create Scala/SBT directories if they don't exist
    mkdir -p "$HOME/.ivy2" 2>/dev/null || true
    mkdir -p "$HOME/.sbt" 2>/dev/null || true
    mkdir -p "$HOME/.cache/coursier" 2>/dev/null || true
  '';

  # Setup script for initial configuration
  setupScript = pkgs.writeScript "setup-scala" ''
    #!${pkgs.bash}/bin/bash
    set -e

    echo "Setting up Scala development environment..."

    # Create directories
    mkdir -p "$HOME/.ivy2"
    mkdir -p "$HOME/.sbt"
    mkdir -p "$HOME/.sbt/1.0/plugins"
    mkdir -p "$HOME/.cache/coursier"

    # Create a basic SBT configuration
    cat > "$HOME/.sbt/1.0/global.sbt" <<EOF
// Global SBT settings
ThisBuild / useCoursier := true

// Increase memory for SBT
javaOptions ++= Seq(
  "-Xmx2G",
  "-XX:+UseG1GC",
  "-XX:ReservedCodeCacheSize=256m"
)
EOF

    # Create SBT repositories configuration
    mkdir -p "$HOME/.sbt"
    cat > "$HOME/.sbt/repositories" <<EOF
[repositories]
  local
  maven-central
  sonatype-snapshots: https://oss.sonatype.org/content/repositories/snapshots
EOF

    echo "Scala environment setup complete!"
    echo "Java version: ${jdk.version}"
    echo "Scala version: ${scala.version}"
    echo "SBT version: $(${sbt}/bin/sbt --version | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' | head -1)"
  '';

in
{
  # Packages to be included
  packages = scalaPackages;

  # Shell hook for development
  shellHook = ''
    ${envVars}
  '';

  # Profile script for persistent environment
  profileScript = profileScript;

  # Setup script for initialization
  setupScript = setupScript;

  # No apps for Scala module
  apps = {};
}
