# Airflow Python Package Overrides
# Copyright Firestream. MIT License.
#
# This file contains wheel and source build overrides for Airflow dependencies.
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
    # Remove docs directory to avoid file collisions
    aiomysql = prev.aiomysql.overrideAttrs (old: {
      postInstall = (old.postInstall or "") + ''
        rm -rf $out/lib/python*/site-packages/docs
      '';
    });

    google-cloud-audit-log = prev.google-cloud-audit-log.overrideAttrs (old: {
      postInstall = (old.postInstall or "") + ''
        rm -rf $out/lib/python*/site-packages/docs
      '';
    });

    grpcio = prev.grpcio.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.zlib
        pkgs.openssl
      ];
    });

    cryptography = prev.cryptography.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.openssl
        pkgs.libffi
      ];
    });

    psycopg2-binary = prev.psycopg2-binary.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.postgresql.lib
      ];
    });

    lxml = prev.lxml.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.libxml2
        pkgs.libxslt
      ];
    });

    mysql-connector-python = prev.mysql-connector-python.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.keyutils.lib
        pkgs.libxcrypt-legacy
        pkgs.krb5
      ];
      autoPatchelfIgnoreMissingDeps = [
        "libudev.so.1"
      ];
    });
  };

  # Source build overrides for packages without Linux wheels
  sourceOverrides = final: prev: {
    gssapi = prev.gssapi.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
        buildPython
        pkgs.krb5.dev
        pkgs.pkg-config
      ];
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.krb5.lib
      ];
      preBuild = ''
        ${setupPythonPath}
        export PATH="${pkgs.krb5.dev}/bin:$PATH"
        export LD_LIBRARY_PATH="${pkgs.krb5.lib}/lib:''${LD_LIBRARY_PATH:-}"
        export C_INCLUDE_PATH="${pkgs.krb5.dev}/include:''${C_INCLUDE_PATH:-}"
        export CPLUS_INCLUDE_PATH="${pkgs.krb5.dev}/include:''${CPLUS_INCLUDE_PATH:-}"
      '';
      GSSAPI_MAIN_LIB = "${pkgs.krb5.lib}/lib/libgssapi_krb5.so";
      GSSAPI_LINKER_ARGS = "-L${pkgs.krb5.lib}/lib";
      GSSAPI_COMPILER_ARGS = "-DHAS_GSSAPI_EXT_H -I${pkgs.krb5.dev}/include -I${pkgs.krb5.dev}/include/gssapi";
    });

    krb5 = prev.krb5.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
        buildPython
        pkgs.krb5.dev
        pkgs.pkg-config
      ];
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.krb5.lib
      ];
      preBuild = ''
        ${setupPythonPath}
        export PATH="${pkgs.krb5.dev}/bin:$PATH"
        export LD_LIBRARY_PATH="${pkgs.krb5.lib}/lib:''${LD_LIBRARY_PATH:-}"
      '';
    });

    pykerberos = prev.pykerberos.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
        buildPython
        pkgs.krb5.dev
        pkgs.pkg-config
      ];
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.krb5.lib
      ];
      preBuild = ''
        ${setupPythonPath}
        export PATH="${pkgs.krb5.dev}/bin:$PATH"
        export LD_LIBRARY_PATH="${pkgs.krb5.lib}/lib:''${LD_LIBRARY_PATH:-}"
      '';
    });

    python-ldap = prev.python-ldap.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
        buildPython
        pkgs.openldap.dev
        pkgs.cyrus_sasl.dev
        pkgs.pkg-config
      ];
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.openldap
        pkgs.cyrus_sasl
      ];
      postPatch = ''
        ${pkgs.gnused}/bin/sed -i 's/compile = 1/compile = 0/g' setup.cfg
        ${pkgs.gnused}/bin/sed -i 's/optimize = 1/optimize = 0/g' setup.cfg
      '';
      preBuild = ''
        export PYTHONPATH="${buildPython}/lib/python3.12/site-packages:''${PYTHONPATH:-}"
        export SETUPTOOLS_USE_DISTUTILS=local
      '';
    });

    mysqlclient = prev.mysqlclient.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
        buildPython
        pkgs.pkg-config
      ];
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.libmysqlclient
        pkgs.openssl
        pkgs.zlib
      ];
      preBuild = setupPythonPath;
    });

    python-nvd3 = prev.python-nvd3.overrideAttrs (old: {
      nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [
        buildPython
      ];
      preBuild = setupPythonPath;
    });
  };
}
