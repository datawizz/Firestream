# JupyterHub Python Package Overrides
# Copyright Firestream. MIT License.
#
# This file contains wheel and source build overrides for JupyterHub dependencies.
# These are necessary because some Python packages:
# - Need system libraries at runtime (wheelOverrides)
# - Don't have prebuilt Linux wheels and need build-time deps (sourceOverrides)

{ pkgs, lib }:

let
  # Build Python with common build tools
  buildPython = pkgs.python312.withPackages (ps: [ ps.setuptools ps.cython ps.wheel ]);
  setupPythonPath = ''
    export PATH="${buildPython}/bin:$PATH"
    export PYTHONPATH="${buildPython}/lib/python3.12/site-packages:''${PYTHONPATH:-}"
  '';
in
{
  # Wheel runtime overrides for C extensions needing system libs
  wheelOverrides = final: prev: {
    # cryptography needs openssl and libffi
    cryptography = prev.cryptography.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.openssl
        pkgs.libffi
      ];
    });

    # psycopg2-binary may need PostgreSQL libs at runtime
    psycopg2-binary = prev.psycopg2-binary.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.postgresql.lib
      ];
    });

    # pamela needs PAM library
    pamela = prev.pamela.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.linux-pam
      ];
    });
  };

  # Source build overrides for packages without Linux wheels
  sourceOverrides = final: prev: {
    # tornado may need source build on some platforms
    tornado = prev.tornado.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
        buildPython
      ];
      preBuild = setupPythonPath;
    });

    # greenlet for async support
    greenlet = prev.greenlet.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
        buildPython
      ];
      preBuild = setupPythonPath;
    });
  };
}
