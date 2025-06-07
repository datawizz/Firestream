

import pulumi
import pulumi_gcp as gcp
from typing import Optional, Dict, Any, Union
import json
import base64
from dataclasses import dataclass
from datetime import datetime
import uuid

@dataclass
class DeploymentSecrets:
    """Container for all deployment-related secrets"""
    cloudflare_domain: str
    deployment_name: str
    customer_name: str
    gcp_project: str
    gcp_region: str
    tunnel_credentials: Optional[Dict[str, Any]] = None
    tailscale_key: Optional[str] = None
    database_credentials: Optional[Dict[str, Any]] = None
    service_account_key: Optional[Dict[str, Any]] = None

    def to_dict(self) -> Dict[str, Any]:
        """Convert secrets to dictionary format"""
        return {
            "cloudflare": {
                "domain": self.cloudflare_domain,
            },
            "deployment": {
                "name": self.deployment_name,
                "customer": self.customer_name,
                "created_at": datetime.now().isoformat(),
            },
            "gcp": {
                "project": self.gcp_project,
                "region": self.gcp_region,
                "service_account": self.service_account_key
            },
            "tunnel": self.tunnel_credentials if self.tunnel_credentials else None,
            "tailscale": {
                "key": self.tailscale_key
            } if self.tailscale_key else None,
            "database": self.database_credentials if self.database_credentials else None
        }

class SecretsManager:
    """Manages consolidated deployment secrets in Google Secret Manager."""

    def __init__(self, service_account: gcp.serviceaccount.Account, opts: Optional[pulumi.ResourceOptions] = None):
        """Initialize SecretsManager with service account."""
        self.config = pulumi.Config("deploy_standard")
        self.service_account = service_account
        self.deployment_secret = None
        self._initialize_deployment_secrets()

    def _initialize_deployment_secrets(self) -> None:
        """Initialize base deployment secrets."""
        secrets = DeploymentSecrets(
            cloudflare_domain=self.config.require("cloudflare_domain"),
            deployment_name=self.config.require("deployment_name"),
            customer_name=self.config.require("customer_name"),
            gcp_project=self.config.require("gcp_project"),
            gcp_region=self.config.require("gcp_region")
        )

        # Create the consolidated secret
        self.deployment_secret = self._create_secret(
            name="deployment-secrets",
            data=secrets.to_dict()
        )

    def update_tunnel_secrets(self, tunnel_config) -> None:
        """Updates the deployment secrets with tunnel configuration."""
        combined_data = pulumi.Output.all(
            tunnel_id=tunnel_config.tunnel_id,
            credentials=tunnel_config.credentials,
            tunnel_config=tunnel_config.tunnel_config,
            yaml_config=tunnel_config.yaml_config
        )

        tunnel_data = combined_data.apply(lambda args: {
            "tunnel_id": args["tunnel_id"],
            "credentials_b64": base64.b64encode(args["credentials"].encode()).decode(),
            "tunnel_config": args["tunnel_config"],
            "yaml_config_b64": base64.b64encode(args["yaml_config"].encode()).decode()
        })

        def update_with_tunnel(existing_data: Dict[str, Any], tunnel: Dict[str, Any]) -> Dict[str, Any]:
            return {
                **existing_data,
                "tunnel": tunnel
            }

        self._update_secret_version(update_type="tunnel", new_data=tunnel_data, update_func=update_with_tunnel)

    def update_database_secrets(self, db_credentials: Dict[str, Any]) -> None:
        """Updates the deployment secrets with database credentials."""
        def update_with_db(existing_data: Dict[str, Any], db: Dict[str, Any]) -> Dict[str, Any]:
            return {
                **existing_data,
                "database": db
            }

        self._update_secret_version(update_type="database", new_data=db_credentials, update_func=update_with_db)

    def update_tailscale_key(self, tailscale_key: str) -> None:
        """Updates the deployment secrets with Tailscale authentication key."""
        def update_with_tailscale(existing_data: Dict[str, Any], key: str) -> Dict[str, Any]:
            return {
                **existing_data,
                "tailscale": {"key": key}
            }

        self._update_secret_version(update_type="tailscale", new_data=tailscale_key, update_func=update_with_tailscale)

    def update_service_account_key(self, sa_key: Dict[str, Any]) -> None:
        """Updates the deployment secrets with service account key."""
        def update_with_sa(existing_data: Dict[str, Any], key: Dict[str, Any]) -> Dict[str, Any]:
            return {
                **existing_data,
                "gcp": {
                    **existing_data.get("gcp", {}),
                    "service_account": key
                }
            }

        self._update_secret_version(update_type="service-account", new_data=sa_key, update_func=update_with_sa)

    def _create_secret(
        self,
        name: str,
        data: Union[Dict[str, Any], pulumi.Output[Dict[str, Any]]],
        opts: Optional[pulumi.ResourceOptions] = None
    ) -> Dict[str, Any]:
        """Creates a single secret with provided data."""
        secret_id = name

        # Create the secret
        secret = gcp.secretmanager.Secret(
            secret_id,
            secret_id=secret_id,
            replication=gcp.secretmanager.SecretReplicationArgs(
                user_managed=gcp.secretmanager.SecretReplicationUserManagedArgs(
                    replicas=[
                        gcp.secretmanager.SecretReplicationUserManagedReplicaArgs(
                            location=self.config.require("gcp_region"),
                        )
                    ]
                )
            ),
            opts=opts
        )

        # Handle both Output and regular dict types
        secret_data = data if isinstance(data, pulumi.Output) else pulumi.Output.from_input(data)

        # Create the initial secret version
        secret_version = gcp.secretmanager.SecretVersion(
            f"{secret_id}-version-initial",
            secret=secret.id,
            secret_data=secret_data.apply(lambda d: json.dumps(d, indent=2)),
            opts=pulumi.ResourceOptions(parent=secret)
        )

        # Grant access to the service account
        secret_binding = gcp.secretmanager.SecretIamBinding(
            f"{secret_id}-binding",
            secret_id=secret.secret_id,
            role="roles/secretmanager.secretAccessor",
            members=[pulumi.Output.concat("serviceAccount:", self.service_account.email)],
            opts=pulumi.ResourceOptions(parent=secret)
        )

        return {
            "secret": secret,
            "version": secret_version,
            "binding": secret_binding
        }

    def _update_secret_version(
        self,
        update_type: str,
        new_data: Any,
        update_func: callable,
        opts: Optional[pulumi.ResourceOptions] = None
    ) -> None:
        """Updates the secret version with modified data."""
        if not self.deployment_secret:
            raise ValueError("Deployment secrets have not been initialized")

        # Get the current secret version's data
        secret = self.deployment_secret["secret"]
        latest_version = self.deployment_secret["version"]

        # Generate a unique identifier for this version update
        version_id = f"deployment-secrets-version-{update_type}-{str(uuid.uuid4())[:8]}"

        # Combine the existing data with the new data using pulumi.Output.all()
        combined = pulumi.Output.all(
            existing_data=latest_version.secret_data.apply(json.loads),
            new_data=new_data
        )

        # Apply the update function to merge the data
        updated_data = combined.apply(
            lambda args: update_func(args["existing_data"], args["new_data"])
        )

        # Create a new version with the updated data and unique name
        new_version = gcp.secretmanager.SecretVersion(
            version_id,
            secret=secret.id,
            secret_data=updated_data.apply(lambda d: json.dumps(d, indent=2)),
            opts=opts or pulumi.ResourceOptions(parent=secret)
        )

        # Update the stored version reference
        self.deployment_secret["version"] = new_version
