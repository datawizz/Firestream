# docker-compose generator flake-module
# Copyright Firestream. MIT License.
#
# Renders a deployable docker-compose.yml for every container from its evaluated
# `compose` config (eval-container.nix), so the image tags in the compose file
# always match what the flake builds. Contributes (on EVERY system):
#   - packages.<name>-compose   derivation holding docker-compose.yml (+ README)
#                               .passthru: { composeFile; projectName;
#                                            hostPorts; buildList; }
#   - apps.<name>-up            build+load required images, then `docker compose up -d`
#   - apps.<name>-down          `docker compose down`
#
# Iterating config.firestreamImages (NOT the isLinux-gated firestreamContainers)
# keeps this working on Darwin. Reading `.imageRef` / `.compose` / `.exposedPorts`
# only forces option evaluation — never the Linux-only image build.
{ ... }: {
  perSystem = { pkgs, lib, config, firestreamBuildImage, ... }:
    let
      yamlFormat = pkgs.formats.yaml { };

      # Re-evaluate a registry image with no overrides to read its config.
      evalImage = name: config.firestreamImages.${name}.eval (_: { });
      depImageRef = key: (evalImage key).imageRef;

      # null ⇒ own image; "@key" ⇒ dependency image ref (tag stays in sync); else literal.
      resolveImage = ownRef: img:
        if img == null then ownRef
        else if lib.hasPrefix "@" img then depImageRef (lib.removePrefix "@" img)
        else img;

      # Parse a service `ports` entry of the form expected by docker-compose short
      # syntax. Returns the host port as an integer when it can be determined
      # (numeric host side, possibly with /protocol suffix), else null.
      # Recognised forms (subset of compose short syntax):
      #   "8090"            -> 8090       (target only; host == container)
      #   "8090:8080"       -> 8090
      #   "8090:8080/tcp"   -> 8090
      #   "127.0.0.1:8090:8080" -> null   (IP-prefixed; preserved verbatim)
      #   "8090-8095:..."   -> null       (port ranges; preserved verbatim)
      parseHostPort = entry:
        let
          # Strip trailing "/proto" if present.
          stripped = lib.head (lib.splitString "/" entry);
          parts = lib.splitString ":" stripped;
          # Sole part = both host and container.
          soleHostStr = if lib.length parts == 1 then lib.head parts else null;
          # 2 parts: host:container.
          twoHostStr = if lib.length parts == 2 then lib.head parts else null;
          candidate = if soleHostStr != null then soleHostStr
                      else if twoHostStr != null then twoHostStr
                      else null;
          # Numeric? (no '-' so we skip ranges; only digits)
          isNumeric = s: s != null
            && s != ""
            && (builtins.match "[0-9]+" s) != null;
        in
          if isNumeric candidate then lib.toInt candidate else null;

      # Apply the offset to a single service-declared `ports` entry. Numeric host
      # sides get `+ offset`; non-numeric / IP-prefixed entries are returned
      # unchanged (compose accepts them and they were the caller's intent).
      applyOffsetToEntry = offset: entry:
        let
          stripped = lib.head (lib.splitString "/" entry);
          proto = lib.removePrefix stripped entry; # "" or "/tcp"
          parts = lib.splitString ":" stripped;
          n = lib.length parts;
          # parseHostPort handles the numeric extraction; if it returns null we
          # do not rewrite.
          host = parseHostPort entry;
        in
          if host == null then entry
          else if n == 1 then
            # "8080" -> "${8080+offset}:8080" preserves container port.
            "${toString (host + offset)}:${toString host}${proto}"
          else if n == 2 then
            # "8090:8080" -> "${8090+offset}:8080"
            "${toString (host + offset)}:${lib.elemAt parts 1}${proto}"
          else
            # 3+ parts (IP:host:cont): leave unchanged.
            entry;

      mkService = { ownRef, sharedEnv, offset }: svc:
        let
          isOwn = svc.image == null;
          envMerged = (if isOwn then sharedEnv else { }) // svc.env;
          portsOffset = map (applyOffsetToEntry offset) svc.ports;
        in
        { image = resolveImage ownRef svc.image; }
        // lib.optionalAttrs (envMerged != { }) { environment = envMerged; }
        // lib.optionalAttrs (portsOffset != [ ]) { ports = portsOffset; }
        // lib.optionalAttrs (svc.volumes != [ ]) { volumes = svc.volumes; }
        // lib.optionalAttrs (svc.dependsOn != [ ]) {
          depends_on = lib.listToAttrs
            (map (d: { name = d; value = { condition = "service_healthy"; }; }) svc.dependsOn);
        }
        // lib.optionalAttrs (svc.healthcheck != null) { healthcheck = svc.healthcheck; }
        // lib.optionalAttrs (svc.restart != "") { restart = svc.restart; };

      # Per-container compose model + rendered YAML.
      mkCompose = name:
        let
          c = evalImage name;
          cmp = c.compose;
          ownRef = c.imageRef;
          offset = cmp.hostPortOffset;

          # Phase 4: synthesise a compose healthcheck for the default
          # single-service branch when the container opts into the in-image
          # firestream-healthd (`health.enable = true`). One contract,
          # three consumers (Rust e2e harness, compose `service_healthy`,
          # future k8s probes). The bash /dev/tcp pattern is universal:
          # every firestream image bundles bashInteractive, so we never need
          # wget/curl/etc. in PATH — just bash.
          defaultHealthcheck = lib.optionalAttrs (c.health.enable or false) {
            healthcheck = {
              test = [
                "CMD"
                "bash"
                "-c"
                "exec 3<>/dev/tcp/127.0.0.1/${toString c.health.port} && printf 'GET /readyz HTTP/1.0\\r\\n\\r\\n' >&3 && head -n 1 <&3 | grep -q ' 200'"
              ];
              interval = "10s";
              timeout = "5s";
              retries = 5;
              start_period = "30s";
            };
          };

          services =
            if cmp.services == { } then
            # Default: a single service from imageRef + exposedPorts.
            # Host port = container port + offset (so multiple stacks can coexist).
              {
                "${name}" = { image = ownRef; }
                  // lib.optionalAttrs (c.exposedPorts != [ ]) {
                  ports = map (p: "${toString (p + offset)}:${toString p}") c.exposedPorts;
                }
                  // defaultHealthcheck;
              }
            else
              lib.mapAttrs (_: mkService { inherit ownRef offset; sharedEnv = cmp.sharedEnv; }) cmp.services;

          # Aggregated host ports actually published by this stack (post-offset).
          # Used by the e2e harness via `passthru.hostPorts` so Rust never has to
          # parse rendered YAML. Includes:
          #   - default branch: exposedPorts + offset
          #   - declared services: numeric host sides of every `ports` entry
          #     (after offset), skipping IP-prefixed / non-numeric / range forms.
          hostPorts =
            if cmp.services == { } then
              map (p: p + offset) c.exposedPorts
            else
              let
                serviceEntries = lib.concatLists
                  (lib.mapAttrsToList (_: svc: svc.ports) cmp.services);
                # Re-run parser on the *offset-applied* entries so the published
                # host side is what shows up.
                applied = map (applyOffsetToEntry offset) serviceEntries;
                parsed = map parseHostPort applied;
              in
                lib.filter (p: p != null) parsed;

          composeData = { name = cmp.projectName; inherit services; }
            // lib.optionalAttrs (cmp.volumes != { }) { volumes = cmp.volumes; };
        in
        {
          inherit cmp ownRef hostPorts;
          composeYaml = yamlFormat.generate "${name}-docker-compose.yml" composeData;
          buildList = [ name ] ++ cmp.dependencies;
        };

      mkComposePkg = name:
        let
          g = mkCompose name;
          c = evalImage name;
          # Phase 3: when the container opts into the in-image health service
          # (`health.enable = true`), publish the post-offset host port for it
          # so the Rust harness can read it directly via
          # `passthru.healthHostPort` without recomputing the offset itself.
          # `null` when health is disabled (the byte-identical case).
          healthHostPort =
            if c.health.enable or false
            then c.health.port + g.cmp.hostPortOffset
            else null;
        in
        if g.cmp.enable then
          pkgs.runCommand "${name}-compose"
            {
              passthru = {
                composeFile = g.composeYaml;
                projectName = g.cmp.projectName;
                hostPorts = g.hostPorts;
                buildList = g.buildList;
                inherit healthHostPort;
              };
            }
            ''
              mkdir -p "$out"
              cp ${g.composeYaml} "$out/docker-compose.yml"
              cat > "$out/README.md" <<EOF
              # ${name} — Firestream docker-compose

              Project: ${g.cmp.projectName}
              Images:  ${toString g.buildList}

              Build + load the images and start the stack:

                  nix run .#${name}-up

              Or manually:

                  ${lib.concatMapStringsSep "\n    " (p: "nix run .#${p}-image -- --load") g.buildList}
                  docker compose -f \$out/docker-compose.yml -p ${g.cmp.projectName} up -d

              Tear down:

                  nix run .#${name}-down
              EOF
            ''
        else
          pkgs.runCommand "${name}-compose-disabled" { } ''
            echo "compose generation is disabled for ${name}" > "$out"
          '';

      mkUpApp = name:
        let
          g = mkCompose name;
          composePkg = mkComposePkg name;
          buildSteps = lib.concatMapStringsSep "\n"
            (p: ''"${firestreamBuildImage}/bin/firestream-build-image" ${lib.escapeShellArg p} --load'')
            g.buildList;
        in
        {
          type = "app";
          program = "${pkgs.writeShellScriptBin "firestream-${name}-up" ''
            set -euo pipefail
            echo ">>> Building + loading images: ${toString g.buildList}" >&2
            ${buildSteps}
            exec docker compose -f ${composePkg}/docker-compose.yml -p ${g.cmp.projectName} up -d "$@"
          ''}/bin/firestream-${name}-up";
        };

      mkDownApp = name:
        let
          g = mkCompose name;
          composePkg = mkComposePkg name;
        in
        {
          type = "app";
          program = "${pkgs.writeShellScriptBin "firestream-${name}-down" ''
            set -euo pipefail
            exec docker compose -f ${composePkg}/docker-compose.yml -p ${g.cmp.projectName} down "$@"
          ''}/bin/firestream-${name}-down";
        };

      names = builtins.attrNames config.firestreamImages;
    in
    {
      packages = lib.listToAttrs
        (map (n: { name = "${n}-compose"; value = mkComposePkg n; }) names);

      apps =
        lib.listToAttrs (map (n: { name = "${n}-up"; value = mkUpApp n; }) names)
        // lib.listToAttrs (map (n: { name = "${n}-down"; value = mkDownApp n; }) names);
    };
}
