# ---------------------------------------------------------------------------
# YOUR local Airflow deployment configuration. Single source of truth for the
# chart override (firestream/airflow-overrides.nix) and the deploy script.
# Smaller than the GKE example — no cloud project, registry, or Secret Manager.
#
# The custom DAG is NOT configured here; it's the ./dags folder, baked into the
# image via firestream/airflow-image-overrides.nix.
# ---------------------------------------------------------------------------
{
  namespace = "airflow";

  # k3s/k3d default provisioner.
  storageClass = "local-path";

  # --- Credentials (inline) ------------------------------------------------
  # Local example: secrets live here, not in Secret Manager. Explicit values
  # give a known login AND keep `helm upgrade` idempotent (Bitnami's postgres
  # refuses upgrades with an empty password).
  airflowPassword = "admin1234"; # Airflow web login (user: admin)
  dbPassword = "airflow"; # PostgreSQL application user
  dbPostgresPassword = "airflow-admin"; # PostgreSQL superuser
  redisPassword = "airflow-redis"; # Redis (Celery broker) password

  # Airflow crypto keys. fernetKey MUST be a 32-byte url-safe base64 value
  # (generate: `python3 -c "from cryptography.fernet import Fernet;
  # print(Fernet.generate_key().decode())"`). The secret/jwt keys are arbitrary
  # strings. Replace these for anything beyond local dev.
  fernetKey = "sV_wWs7mF7FHputWypHG6IgvkpAtWGgh-VSAVAEHuxE=";
  secretKey = "firestream-local-webserver-secret";
  jwtSecretKey = "firestream-local-jwt-secret";
}
