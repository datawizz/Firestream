# python.nix - Python development environment
{ pkgs, system ? null, fenixPackages ? null, firestreamLib ? null }:

let
  # Python version to use
  pythonVersion = pkgs.python312;
  
  # Python packages
  pythonPackages = with pythonVersion.pkgs; [
    # Core development tools
    pip
    virtualenv
    uv
    black
    ipython
    pytest
    pylint
    mypy
    flake8
    jupyter
    
    # Common libraries (add more as needed)
    numpy
    pandas
    requests
    pydantic
  ];

  # Environment variables for Python
  envVars = ''
    # Python environment variables
    export PYTHON="${pythonVersion}/bin/python"
    export PYTHONPATH="$HOME/.python"
    export JUPYTER_PATH="$HOME/.python/share/jupyter"
    export IPYTHONDIR="$HOME/.python"
    
    # Force UV to use Nix Python
    export UV_PYTHON_DOWNLOADS=never
    export UV_PYTHON="${pythonVersion}/bin/python"
    export UV_LINK_MODE=copy
  '';

  # Profile script for Python setup
  profileScript = ''
    ${envVars}
    
    # Create symlink to Python in home directory if it doesn't exist
    if [ ! -L "$HOME/.python" ]; then
      ln -sf ${pythonVersion}/bin/python "$HOME/.python"
    fi
    
    # Auto-detect and use UV when in a project
    alias python='[[ -f pyproject.toml ]] && uv run python || ${pythonVersion}/bin/python'
    alias pip='[[ -f pyproject.toml ]] && uv pip || ${pythonVersion}/bin/python -m pip'
    alias jupyter='[[ -f pyproject.toml ]] && uv run jupyter || jupyter'
  '';

  # Setup script for container initialization
  setupScript = pkgs.writeScript "setup-python" ''
    #!${pkgs.bash}/bin/bash
    set -e
    
    echo "Setting up Python environment..."
    
    # Create Python directories
    mkdir -p "$HOME/.python"
    mkdir -p "$HOME/.cache/pip"
    
    # Create symlink to Python
    ln -sf ${pythonVersion}/bin/python "$HOME/.python"
    
    echo "Python environment setup complete!"
    echo "Python version: ${pythonVersion.version}"
  '';

in
{
  # Packages to be included
  packages = [ pythonVersion ] ++ pythonPackages;
  
  # Shell hook for development
  shellHook = ''
    ${envVars}
  '';
  
  # Profile script for persistent environment
  profileScript = profileScript;
  
  # Setup script for initialization
  setupScript = setupScript;
  
  # No apps for Python module
  apps = {};
}
