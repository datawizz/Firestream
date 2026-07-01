# Sparse CHART override for the local k3s deployment.
#
# Deep-merged onto the Firestream nextjs chart defaults by
# `charts.nextjs.eval`. We do NOT repoint images — the injected
# firestream-nextjs + firestream-postgresql refs are kept, and
# scripts/deploy-local.sh side-loads them into the cluster.
{ ... }:

let
  cfg = import ../config.nix;
in
{
  config.nextjs = {
    # Bundled PostgreSQL subchart (the firestream-postgresql image is injected).
    postgresql = {
      enabled = true;
      architecture = "standalone";
      auth = {
        username = "firestream";
        database = "firestream_nextjs";
        password = cfg.dbPassword;
        postgresPassword = cfg.dbPostgresPassword;
      };
    };

    # Bind every PVC to the local-path provisioner.
    global.defaultStorageClass = cfg.storageClass;
  };
}
