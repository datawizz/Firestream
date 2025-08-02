# node.nix - Node.js development environment
{ pkgs, self ? null, system ? null }:

let
  # Node.js version to use
  nodejs = pkgs.nodejs_22;
  
  # Create a wrapper script for npm configuration
  npmConfigScript = pkgs.writeScriptBin "configure-npm" ''
    #!${pkgs.bash}/bin/bash
    
    # Create directory for global npm packages if it doesn't exist
    mkdir -p "$HOME/.npm-global"
    
    # Configure npm to use the home directory for global packages
    ${nodejs}/bin/npm config set prefix "$HOME/.npm-global"
    
    # Ensure cache is also in user directory
    ${nodejs}/bin/npm config set cache "$HOME/.npm-cache"
    
    echo "npm configured to use ~/.npm-global for global packages"
  '';

  # Environment variables for Node.js
  envVars = ''
    # Add npm global bin directory to PATH
    export PATH="$HOME/.npm-global/bin:$PATH"
    
    # Node.js environment variables
    export NODE_PATH="$HOME/.npm-global/lib/node_modules"
    
    # Ensure npm uses the correct directories
    export NPM_CONFIG_PREFIX="$HOME/.npm-global"
    export NPM_CONFIG_CACHE="$HOME/.npm-cache"
  '';

  # Profile script that sets up Node.js environment
  profileScript = ''
    ${envVars}
    
    # Create npm directories on first login if they don't exist
    if [ ! -d "$HOME/.npm-global" ]; then
      mkdir -p "$HOME/.npm-global"
      mkdir -p "$HOME/.npm-cache"
      
      # Configure npm on first use
      if command -v npm &> /dev/null; then
        npm config set prefix "$HOME/.npm-global"
        npm config set cache "$HOME/.npm-cache"
      fi
    fi
  '';

  # Setup script for container initialization
  setupScript = pkgs.writeScript "setup-node" ''
    #!${pkgs.bash}/bin/bash
    set -e
    
    echo "Setting up Node.js environment..."
    
    # Run the npm configuration
    ${npmConfigScript}/bin/configure-npm
    
    echo "Node.js environment setup complete!"
    echo "Node.js version: ${nodejs.version}"
  '';

in
{
  # Packages to be included
  packages = [
    nodejs
    npmConfigScript
  ];
  
  # Shell hook for development
  shellHook = ''
    ${envVars}
    echo "Node.js ${nodejs.version} environment loaded"
  '';
  
  # Profile script for persistent environment
  profileScript = profileScript;
  
  # Setup script for initialization
  setupScript = setupScript;
  
  # Apps for Node.js module
  apps = {
    configure-npm = {
      type = "app";
      program = "${npmConfigScript}/bin/configure-npm";
    };
  };
}
