#!/bin/bash
set -e
### JVM Config ###

#TODO make this parameterized for JDK vs JRE

# Install SDKMAN
# echo "Installing SDKMAN..."

# source /workspace/bin/install_scripts/sdk-man.sh

# Apply changes to the current shell
source ~/.bashrc

# Install Java 11 from OpenJDK
echo "Installing Java 11 from OpenJDK..."
sdk install java 11.0.12-open

# Set JAVA_HOME in bashrc
echo "Setting JAVA_HOME in bashrc..."
echo "export JAVA_HOME=\"$(sdk home java 11.0.12-open)\"" >> ~/.bashrc

# Update PATH with Java home
echo "Updating PATH with Java home..."
echo "export PATH=\$JAVA_HOME/bin:\$PATH" >> ~/.bashrc

# Apply changes to the current shell
source ~/.bashrc

# Install Maven
echo "Installing Maven..."
sdk install maven

# Install Scala
echo "Installing Scala..."
sdk install scala

echo "Installation complete. Java, Maven, and Scala have been installed and configured."
