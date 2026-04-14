# Superset 5.x Python Package Overrides
# Copyright Firestream. MIT License.
#
# This file contains wheel and source build overrides for Superset 5.x dependencies.
# These are necessary because some Python packages:
# - Need system libraries at runtime (wheelOverrides)
# - Don't have prebuilt Linux wheels and need build-time deps (sourceOverrides)
#
# Note: Superset 5.x uses Python 3.11

{ pkgs, lib }:

let
  # Build Python with common build tools (Python 3.11 for Superset 5.x)
  buildPython = pkgs.python311.withPackages (ps: [ ps.setuptools ps.cython ps.wheel ps.hatchling ps.flit-core ]);
  setupPythonPath = ''
    export PATH="${buildPython}/bin:$PATH"
    export PYTHONPATH="${buildPython}/lib/python3.11/site-packages:''${PYTHONPATH:-}"
  '';

  # Helper to add setuptools to a package
  addSetuptools = pkg: pkg.overrideAttrs (old: {
    nativeBuildInputs = (old.nativeBuildInputs or [ ]) ++ [ buildPython ];
    preBuild = (old.preBuild or "") + setupPythonPath;
  });
in
{
  # Wheel runtime overrides for C extensions needing system libs
  wheelOverrides = final: prev: {
    # Cryptography needs openssl and libffi
    cryptography = prev.cryptography.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.openssl
        pkgs.libffi
      ];
    });

    # psycopg2-binary needs postgresql
    psycopg2-binary = prev.psycopg2-binary.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.postgresql.lib
      ];
    });

    # lxml needs libxml2/libxslt
    lxml = prev.lxml.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.libxml2
        pkgs.libxslt
      ];
    });

    # gevent needs libevent
    gevent = prev.gevent.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.libevent
      ];
    });

    # Pillow needs image libraries
    pillow = prev.pillow.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.libjpeg
        pkgs.zlib
        pkgs.libpng
        pkgs.freetype
      ];
    });

    # pyarrow needs arrow-cpp
    pyarrow = prev.pyarrow.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.arrow-cpp
      ];
    });

    # grpcio needs zlib and openssl
    grpcio = prev.grpcio.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.zlib
        pkgs.openssl
      ];
    });

    # markupsafe needs gcc runtime
    markupsafe = prev.markupsafe.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.stdenv.cc.cc.lib
      ];
    });

    # numba needs Intel TBB for tbbpool threading backend
    numba = prev.numba.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.tbb
        pkgs.stdenv.cc.cc.lib
      ];
    });

    # llvmlite (numba dependency) needs LLVM
    llvmlite = prev.llvmlite.overrideAttrs (old: {
      buildInputs = (old.buildInputs or [ ]) ++ [
        pkgs.llvm
        pkgs.stdenv.cc.cc.lib
      ];
    });
  };

  # Source build overrides for packages without Linux wheels
  sourceOverrides = final: prev: {
    # python-ldap needs additional system libraries
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
        export PYTHONPATH="${buildPython}/lib/python3.11/site-packages:''${PYTHONPATH:-}"
        export SETUPTOOLS_USE_DISTUTILS=local
      '';
    });

    # mysqlclient needs libmysqlclient
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

    # All sdist-only packages that need setuptools for building
    apache-superset = addSetuptools prev.apache-superset;
    bottleneck = addSetuptools prev.bottleneck;
    func-timeout = addSetuptools prev.func-timeout;
    python-geohash = addSetuptools prev.python-geohash;
    shortid = addSetuptools prev.shortid;
    alembic = addSetuptools prev.alembic;
    apispec = addSetuptools prev.apispec;
    babel = addSetuptools prev.babel;
    backoff = addSetuptools prev.backoff;
    blinker = addSetuptools prev.blinker;
    cachelib = addSetuptools prev.cachelib;
    cachetools = addSetuptools prev.cachetools;
    celery = addSetuptools prev.celery;
    click = addSetuptools prev.click;
    click-didyoumean = addSetuptools prev.click-didyoumean;
    click-option-group = addSetuptools prev.click-option-group;
    click-plugins = addSetuptools prev.click-plugins;
    click-repl = addSetuptools prev.click-repl;
    colorama = addSetuptools prev.colorama;
    cron-descriptor = addSetuptools prev.cron-descriptor;
    croniter = addSetuptools prev.croniter;
    deprecated = addSetuptools prev.deprecated;
    deprecation = addSetuptools prev.deprecation;
    dnspython = addSetuptools prev.dnspython;
    email-validator = addSetuptools prev.email-validator;
    et-xmlfile = addSetuptools prev.et-xmlfile;
    filelock = addSetuptools prev.filelock;
    flask = addSetuptools prev.flask;
    flask-appbuilder = addSetuptools prev.flask-appbuilder;
    flask-babel = addSetuptools prev.flask-babel;
    flask-caching = addSetuptools prev.flask-caching;
    flask-jwt-extended = addSetuptools prev.flask-jwt-extended;
    flask-limiter = addSetuptools prev.flask-limiter;
    flask-login = addSetuptools prev.flask-login;
    flask-migrate = addSetuptools prev.flask-migrate;
    flask-session = addSetuptools prev.flask-session;
    flask-sqlalchemy = addSetuptools prev.flask-sqlalchemy;
    flask-talisman = addSetuptools prev.flask-talisman;
    flask-wtf = addSetuptools prev.flask-wtf;
    flower = addSetuptools prev.flower;
    future = addSetuptools prev.future;
    geographiclib = addSetuptools prev.geographiclib;
    geopy = addSetuptools prev.geopy;
    google-api-core = addSetuptools prev.google-api-core;
    google-auth = addSetuptools prev.google-auth;
    google-cloud-bigquery = addSetuptools prev.google-cloud-bigquery;
    google-cloud-core = addSetuptools prev.google-cloud-core;
    google-resumable-media = addSetuptools prev.google-resumable-media;
    gunicorn = addSetuptools prev.gunicorn;
    holidays = addSetuptools prev.holidays;
    humanize = addSetuptools prev.humanize;
    idna = addSetuptools prev.idna;
    isodate = addSetuptools prev.isodate;
    itsdangerous = addSetuptools prev.itsdangerous;
    jinja2 = addSetuptools prev.jinja2;
    jmespath = addSetuptools prev.jmespath;
    jsonschema = addSetuptools prev.jsonschema;
    kombu = addSetuptools prev.kombu;
    limits = addSetuptools prev.limits;
    mako = addSetuptools prev.mako;
    markdown = addSetuptools prev.markdown;
    marshmallow = addSetuptools prev.marshmallow;
    marshmallow-sqlalchemy = addSetuptools prev.marshmallow-sqlalchemy;
    odfpy = addSetuptools prev.odfpy;
    ordered-set = addSetuptools prev.ordered-set;
    packaging = addSetuptools prev.packaging;
    prison = addSetuptools prev.prison;
    prompt-toolkit = addSetuptools prev.prompt-toolkit;
    proto-plus = addSetuptools prev.proto-plus;
    protobuf = addSetuptools prev.protobuf;
    pyasn1 = addSetuptools prev.pyasn1;
    pyasn1-modules = addSetuptools prev.pyasn1-modules;
    pyjwt = addSetuptools prev.pyjwt;
    python-dateutil = addSetuptools prev.python-dateutil;
    python-dotenv = addSetuptools prev.python-dotenv;
    pytz = addSetuptools prev.pytz;
    pyyaml = addSetuptools prev.pyyaml;
    redis = addSetuptools prev.redis;
    rich = addSetuptools prev.rich;
    rsa = addSetuptools prev.rsa;
    s3transfer = addSetuptools prev.s3transfer;
    selenium = addSetuptools prev.selenium;
    shortuuid = addSetuptools prev.shortuuid;
    simplejson = addSetuptools prev.simplejson;
    six = addSetuptools prev.six;
    slack-sdk = addSetuptools prev.slack-sdk;
    sqlalchemy = addSetuptools prev.sqlalchemy;
    sqlalchemy-utils = addSetuptools prev.sqlalchemy-utils;
    sqlparse = addSetuptools prev.sqlparse;
    tabulate = addSetuptools prev.tabulate;
    tenacity = addSetuptools prev.tenacity;
    toml = addSetuptools prev.toml;
    tornado = addSetuptools prev.tornado;
    trino = addSetuptools prev.trino;
    typing-extensions = addSetuptools prev.typing-extensions;
    tzdata = addSetuptools prev.tzdata;
    urllib3 = addSetuptools prev.urllib3;
    vine = addSetuptools prev.vine;
    wcwidth = addSetuptools prev.wcwidth;
    werkzeug = addSetuptools prev.werkzeug;
    wrapt = addSetuptools prev.wrapt;
    wtforms = addSetuptools prev.wtforms;
    wtforms-json = addSetuptools prev.wtforms-json;
    xlsxwriter = addSetuptools prev.xlsxwriter;
    zipp = addSetuptools prev.zipp;
  };
}
