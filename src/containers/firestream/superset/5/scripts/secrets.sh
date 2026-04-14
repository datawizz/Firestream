# Superset secrets processing
# This file is sourced, not executed directly
# Copyright Firestream. MIT License.

# Docker secrets support is handled automatically by the Firestream
# file-loader module which processes all *_FILE environment variables.
#
# Supported secret variables:
#   SUPERSET_SECRET_KEY_FILE
#   SUPERSET_PASSWORD_FILE
#   SUPERSET_DATABASE_PASSWORD_FILE
#   REDIS_PASSWORD_FILE
#   FLOWER_BASIC_AUTH_FILE
#
# Example Docker Compose usage:
#   services:
#     superset:
#       environment:
#         - SUPERSET_SECRET_KEY_FILE=/run/secrets/superset_secret_key
#         - SUPERSET_DATABASE_PASSWORD_FILE=/run/secrets/db_password
#       secrets:
#         - superset_secret_key
#         - db_password
#
# The file-loader module automatically:
# 1. Reads the content of files specified in *_FILE variables
# 2. Sets the corresponding environment variable (without _FILE suffix)
# 3. Unsets the _FILE variable

debug "Secrets processing delegated to Firestream file-loader module"
