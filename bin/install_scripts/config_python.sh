#!/bin/bash

# Create the virtual environment directory if it doesn't exist
VENV_PATH="$HOME/opt/python/.venv"
mkdir -p "$(dirname "$VENV_PATH")"

# Create the virtual environment
python -m venv "$VENV_PATH"

# Create the profile additions
PROFILE_CONTENT="
# Python virtual environment configuration
export VIRTUAL_ENV=\"$VENV_PATH\"
export PATH=\"\$VIRTUAL_ENV/bin:\$PATH\"
# Unset PYTHON_HOME to avoid conflicts
unset PYTHON_HOME"

# Add to .profile if not already present
if ! grep -q "VIRTUAL_ENV=\"$VENV_PATH\"" "$HOME/.profile"; then
    echo "$PROFILE_CONTENT" >> "$HOME/.profile"
    echo "Virtual environment configuration added to .profile"
else
    echo "Virtual environment configuration already exists in .profile"
fi

# Activate the virtual environment for the current session
source "$VENV_PATH/bin/activate"

python -m pip install -r /workspace/requirements.txt

echo "Virtual environment setup complete at $VENV_PATH"
echo "Please restart your shell or run 'source ~/.profile' to apply changes"
