# Next.js Container Module — Firestream Node factory
# Copyright Firestream. MIT License.
#
# Builds a PRODUCTION Next.js container: `mkNodePackage` compiles the app with
# Next.js standalone output (`output: "standalone"` in next.config.js), and
# `mkNodeContainerModule` wraps the traced server into a Firestream image.
#
# This is a NET-NEW supported app (not a Bitnami fork): the application source
# is Firestream's own (./app), overridable per-deployment via the `appSource`
# seam forwarded from options.nix (config.nextjs.vendoredApp →
# extraModuleArgs.appSource).
#
# Invoked by eval-container.nix's `runtimeType = "system"` branch, which passes:
#   { pkgs, lib, firestream, version, envVars, envVarsWithSecrets, paths,
#     exposedPorts, imageName, imageTag } // extraModuleArgs
# `firestream` is firestreamLib (exposes mkNodePackage + mkNodeContainerModule).

{ pkgs
, lib
, firestream
, version ? "1.0.0"
, envVars ? { }
, envVarsWithSecrets ? [ ]
, paths ? {
    base = "/opt/firestream/nextjs";
    conf = "/opt/firestream/nextjs/config";
    data = "/firestream/nextjs/data";
    logs = "/opt/firestream/nextjs/logs";
  }
, exposedPorts ? [ 3000 ]
  # imageName/imageTag are supplied by eval-container; mkNodeContainerModule
  # derives them from `name`/`version` itself, so we accept-and-ignore to keep
  # the firestream-nextjs:<version> default (the chart injects registry/repo/tag
  # into values.yaml independently).
, imageName ? "firestream-nextjs"
, imageTag ? version
  # Vendored-app override seam (config.nextjs.vendoredApp).
, appSource ? { src = null; npmDepsHash = null; }
}:

let
  nodejs = pkgs.nodejs_22;

  # Install location inside the package derivation; also referenced by runCmd.
  appDir = "firestream-nextjs";

  # ./app is the canonical Firestream Next.js source; a downstream example
  # overrides it with its own tree via config.nextjs.vendoredApp.src.
  appSrc = if appSource.src != null then appSource.src else ./app;

  # buildNpmPackage fixed-output hash for the lockfile in `appSrc`. The default
  # below is the hash of the canonical ./app lockfile (pin after first build
  # with `nix build .#nextjs`); a vendored app supplies its own.
  depHash =
    if appSource.npmDepsHash != null
    then appSource.npmDepsHash
    else "sha256-1BUzCAuAUshgvkL8Hxr3z9WYBhBh59nTO9XrImBYvZQ=";

  # Build the Next.js app to standalone output.
  nextApp = firestream.mkNodePackage {
    pname = appDir;
    inherit version nodejs;
    src = appSrc;
    npmDepsHash = depHash;

    # We run the app via runCmd (node server.js); no bin wrapper needed.
    skipBinWrapper = true;

    # Next.js needs its platform-specific @next/swc-* (optional deps) at build
    # time; mkNodePackage defaults npm_config_optional to "false", so re-enable.
    env = {
      NEXT_TELEMETRY_DISABLED = "1";
      npm_config_optional = "true";
    };

    # mkNodePackage's plain-JS default buildPhase is a no-op; force a real
    # production build (runs `next build` → emits .next/standalone).
    buildPhase = ''
      runHook preBuild
      npm run build
      runHook postBuild
    '';

    # Assemble the runnable standalone tree the server expects: server.js +
    # pruned node_modules at the root, static assets under .next/static, and
    # public/ if present.
    installPhase = ''
      runHook preInstall
      mkdir -p "$out/lib/${appDir}"
      cp -r .next/standalone/. "$out/lib/${appDir}/"
      mkdir -p "$out/lib/${appDir}/.next"
      cp -r .next/static "$out/lib/${appDir}/.next/static"
      if [ -d public ]; then
        cp -r public "$out/lib/${appDir}/public"
      fi
      runHook postInstall
    '';
  };

in
firestream.mkNodeContainerModule {
  name = "nextjs";
  inherit version nodejs;
  nodePackage = nextApp;
  nodeEnv = "production";

  inherit paths exposedPorts envVars envVarsWithSecrets;

  # Docker HEALTHCHECK + a target for the chart's HTTP probes.
  healthCheckPort = 3000;
  healthCheckPath = "/api/health";

  volumes = [ "/firestream/nextjs/data" ];

  # The standalone server resolves .next/static and public/ relative to its cwd.
  # NOTE: the run wrapper emits `exec ${runCmd}`, so the body MUST start with a
  # comment (turning that prefix into a harmless no-op `exec`) and end with its
  # own `exec` — the same convention redis/spark use.
  runCmd = ''
    # Launch the Next.js standalone server.
    cd "${nextApp}/lib/${appDir}"
    exec ${nodejs}/bin/node "${nextApp}/lib/${appDir}/server.js"
  '';
}
