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

  # pnpm workspace path is selected by supplying pnpmDepsHash. Then `src` is the
  # workspace root, the app lives at `subDir`, and the standalone server is at
  # .next/standalone/<subDir>/server.js (Next writes the app path relative to
  # outputFileTracingRoot = workspace root). npm path stays flat (server.js at
  # the standalone root).
  isPnpm = (appSource.pnpmDepsHash or null) != null;
  subDir = appSource.appDir or ".";
  isNested = isPnpm && subDir != "." && subDir != "";
  # Where server.js lives inside $out/lib/${appDir} at runtime.
  runSubDir = if isNested then subDir else ".";

  pkgName =
    if (appSource.packageName or null) != null then appSource.packageName
    else baseNameOf subDir;
  buildBefore = appSource.buildBefore or [ ];
  # Install these workspace packages (with devDeps) so libs can be built.
  pnpmWorkspaces = lib.optionals isNested ([ pkgName ] ++ buildBefore);

  # buildNpmPackage fixed-output hash for the npm lockfile in `appSrc` (npm path
  # only). The default is the canonical ./app lockfile hash.
  depHash =
    if appSource.npmDepsHash != null
    then appSource.npmDepsHash
    else "sha256-1BUzCAuAUshgvkL8Hxr3z9WYBhBh59nTO9XrImBYvZQ=";

  # Build the Vite libraries (buildBefore) then the app, from the workspace root.
  pnpmBuildPhase = ''
    runHook preBuild
    ${lib.concatMapStringsSep "\n" (p: "pnpm --filter ${p} run build") buildBefore}
    pnpm --filter ${pkgName} run build
    runHook postBuild
  '';

  # Nested standalone: server.js + hoisted node_modules land at the standalone
  # root; the app's static/public sit under <subDir>. cwd during install is the
  # workspace root, so the standalone tree is at <subDir>/.next/standalone.
  pnpmInstallPhase = ''
    runHook preInstall
    mkdir -p "$out/lib/${appDir}"
    cp -r ${subDir}/.next/standalone/. "$out/lib/${appDir}/"
    mkdir -p "$out/lib/${appDir}/${subDir}/.next"
    cp -r ${subDir}/.next/static "$out/lib/${appDir}/${subDir}/.next/static"
    if [ -d ${subDir}/public ]; then
      cp -r ${subDir}/public "$out/lib/${appDir}/${subDir}/public"
    fi
    runHook postInstall
  '';

  npmBuildPhase = ''
    runHook preBuild
    npm run build
    runHook postBuild
  '';

  npmInstallPhase = ''
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

  # Build the Next.js app to standalone output (pnpm-workspace or npm).
  nextApp = firestream.mkNodePackage ({
    pname = appDir;
    inherit version nodejs;
    src = appSrc;

    # We run the app via runCmd (node server.js); no bin wrapper needed.
    skipBinWrapper = true;

    # Next.js needs its platform-specific @next/swc-* (optional deps) at build
    # time; keep them enabled (npm defaults them off).
    env = {
      NEXT_TELEMETRY_DISABLED = "1";
      npm_config_optional = "true";
    };

    buildPhase = if isPnpm then pnpmBuildPhase else npmBuildPhase;
    installPhase = if isPnpm then pnpmInstallPhase else npmInstallPhase;
  } // (if isPnpm
        then { pnpmDepsHash = appSource.pnpmDepsHash; inherit pnpmWorkspaces; }
        else { npmDepsHash = depHash; }));

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
    # Launch the Next.js standalone server. For a pnpm-workspace app the server
    # lives under <subDir> (hoisted node_modules sits one level up at the
    # standalone root; Node resolves upward).
    cd "${nextApp}/lib/${appDir}/${runSubDir}"
    exec ${nodejs}/bin/node "${nextApp}/lib/${appDir}/${runSubDir}/server.js"
  '';
}
