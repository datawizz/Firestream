# ---------------------------------------------------------------------------
# The "make it your own" surface for GKE.
#
# Passed as the LAST module to `firestream.lib.<sys>.charts.nextjs.eval`, so
# every value deep-merges on top of the Firestream defaults and the
# image-injection overlay. Intentionally sparse.
#
# Contrast with ../nextjs-k3s: here we REPOINT the image refs at Artifact
# Registry (Cloud Build pushes them there) and read the database credentials
# from a K8s Secret that Cloud Build materialises from Secret Manager
# (`existingSecret`), so no passwords land in git or values.yaml.
#
# Option names mirror src/charts/firestream/nextjs/values.yaml.
# ---------------------------------------------------------------------------
{ ... }:

let
  cfg = import ../config.nix;
in
{
  config.nextjs = {
    # --- Next.js image from Artifact Registry (Cloud Build pushes it). This
    #     overlaps the slot Firestream injects by default; an explicit user
    #     value wins. ----------------------------------------------------------
    image = {
      registry = cfg.arHost;
      repository = "${cfg.arRepo}/firestream-nextjs";
      tag = cfg.imageTag;
    };

    # --- Bundled PostgreSQL subchart ---------------------------------------
    postgresql = {
      enabled = true;
      architecture = "standalone";
      auth = {
        username = "firestream";
        database = "firestream_nextjs";
        # Bitnami pg reads `password` (user) + `postgres-password` (admin).
        existingSecret = cfg.dbSecretName;
      };
      image = {
        registry = cfg.arHost;
        repository = "${cfg.arRepo}/firestream-postgresql";
        tag = cfg.imageTag;
      };
      primary.persistence = {
        enabled = true;
        storageClass = cfg.storageClass;
        size = "8Gi";
      };
    };

    # --- Storage default for any component PVCs ----------------------------
    global.defaultStorageClass = cfg.storageClass;

    # --- Expose via a GKE L7 ingress (GCE). On Autopilot/VPC-native clusters
    #     a ClusterIP service is fronted by container-native (NEG) LB. ---------
    service.type = "ClusterIP";
    ingress = {
      enabled = true;
      ingressClassName = "gce";
      hostname = cfg.domain;
      path = "/";
      pathType = "Prefix";
    };
  };
}
