from dataclasses import dataclass, asdict
import json
import yaml
from pathlib import Path
from typing import Optional, Dict, List, Any
import pulumi
from pulumi import Output, ResourceOptions


from infrastructure.cloudflare_access import ClouflareAccessManager, AccessConfig
from infrastructure.cloudflare_tunnel import CloudflareTunnelManager, TunnelConfig
from infrastructure.google_cloud_storage import BucketConfig, BucketManager
from infrastructure.google_service_account import ServiceAccountManager
from infrastructure.google_compute_engine import GoogleComputeManager
from infrastructure.google_identity_platform import IdentityPlatformManager, IdentityConfig
from infrastructure.google_secrets_manager import SecretsManager
from infrastructure.tailscale import TailscaleManager
from infrastructure.nixos_deployment import NixOSDeploymentManager



@dataclass
class MakeResult:
    """Standard return type for all make_* functions"""
    outputs: Dict[str, Output[any]]
    resources: Optional[Dict[str, any]] = None


@dataclass
class AccessOutput:
    domain: str
    id: str

@dataclass
class StorageOutput:
    app_bucket: str
    machine_images: str

@dataclass
class ComputeImageInfo:
    gcs_url: str
    hash: str
    size: str
    path: str

@dataclass
class ComputeImage:
    name: str
    self_link: str
    info: ComputeImageInfo

@dataclass
class ComputeInstance:
    name: str
    external_ip: Optional[str]
    self_link: str

@dataclass
class ComputeOutput:
    image: ComputeImage
    instance: ComputeInstance

@dataclass
class TunnelOutput:
    id: str
    credentials: Dict[str, Any]
    secrets: Dict[str, Any]

@dataclass
class TailscaleOutput:
    auth_key: str
    expiry_date: str
    next_rotation: str
    secrets: Dict[str, Any]

@dataclass
class SecurityOutput:
    service_account: str
    deployment_secret: str
    tunnel: TunnelOutput
    tailscale: TailscaleOutput

@dataclass
class DeploymentOutput:
    access: AccessOutput
    storage: StorageOutput
    compute: ComputeOutput
    security: SecurityOutput




def make_buckets(
    bucket_configs: Optional[List[BucketConfig]] = None
) -> MakeResult:
    """
    Create Google Cloud Storage buckets and export their URLs.

    Args:
        bucket_configs: Optional list of bucket configurations. If None, creates default buckets.

    Returns:
        MakeResult containing bucket URLs and resources
    """
    _manager = BucketManager()

    if bucket_configs is None:
        bucket_configs = [
            BucketConfig(
                bucket_name="machines-images",
                location="us-central1",
                force_destroy=True
            ),
            BucketConfig(
                bucket_name="app-bucket",
                location="us-central1",
                force_destroy=True
            )
        ]

    created_buckets = []
    for bucket in bucket_configs:
        created_bucket = _manager.create_bucket(
            bucket.bucket_name,
            bucket.location,
            bucket.force_destroy
        )
        created_buckets.append(created_bucket)

    return MakeResult(
        outputs=_manager.get_bucket_urls(),
        resources={"buckets": created_buckets}
    )

def make_tailscale(
    config: Optional[pulumi.Config] = None,
    opts: Optional[ResourceOptions] = None
) -> MakeResult:
    """
    Creates a Tailscale instance with dynamic key management.

    Args:
        config: Optional Pulumi config. If None, uses default config.
        opts: Optional resource options.

    Returns:
        MakeResult containing Tailscale outputs and resources
    """
    if config is None:
        config = pulumi.Config()

    api_key = config.require_secret("tailscale_api_key")
    customer_name = config.require("customer_name")
    tailscale_tailnet_name = config.require("tailscale_tailnet_name")

    manager = TailscaleManager(
        api_key=api_key,
        customer_name=customer_name,
        rotation_days=60,
        overlap_days=7,
        tailnet=tailscale_tailnet_name
    )

    auth_key_outputs = manager.create_auth_key()

    return MakeResult(
        outputs=auth_key_outputs,
        resources={"manager": manager}
    )

def make_cloudflare_access(
    config: Optional[pulumi.Config] = None,
    opts: Optional[ResourceOptions] = None
) -> MakeResult:
    """
    Creates Cloudflare Access configuration.

    Args:
        config: Optional Pulumi config. If None, uses default config.
        opts: Optional resource options.

    Returns:
        MakeResult containing access configuration
    """
    if config is None:
        config = pulumi.Config()

    customer_email_domain = config.require('customer_email_domain')

    allowed_domains = [
        "centerpoint.consulting",
        customer_email_domain
    ]

    if opts is None:
        opts = ResourceOptions(protect=False)

    access_config = access.create_access_config(
        allowed_domains=allowed_domains,
        opts=opts
    )

    return MakeResult(
        outputs={"access_config": access_config}
    )

def make_cloudflare_tunnel(
    depends_on: Optional[pulumi.Resource] = None,
    opts: Optional[ResourceOptions] = None
) -> MakeResult:
    """
    Creates Cloudflare tunnel with proper configuration.

    Args:
        depends_on: Optional resource dependency.
        opts: Optional resource options.

    Returns:
        MakeResult containing tunnel configuration
    """
    if opts is None:
        opts = ResourceOptions(depends_on=[depends_on] if depends_on else None)

    tunnel_config = tunnel.create_tunnel(opts=opts)

    outputs = {
        "tunnel_info": {
            "tunnel_id": tunnel_config.tunnel_id,
            "credentials": tunnel_config.credentials
        }
    }

    return MakeResult(
        outputs=outputs,
        resources={"tunnel_config": tunnel_config}
    )

def make_gcp_project(
    config: Optional[pulumi.Config] = None,
    opts: Optional[ResourceOptions] = None
) -> MakeResult:
    pass


def make_service_account(
    config: Optional[pulumi.Config] = None,
    opts: Optional[ResourceOptions] = None
) -> MakeResult:
    """
    Creates a Google Cloud service account with necessary permissions.

    Args:
        config: Optional Pulumi config. If None, uses default config.
        opts: Optional resource options.

    Returns:
        MakeResult containing service account information
    """
    if config is None:
        config = pulumi.Config()

    gcp_project = config.require('gcp_project')
    deployment_name = config.require('deployment_name')

    sa_manager = ServiceAccountManager(
        project_id=gcp_project,
        service_account_id=deployment_name
    )

    sa_manager.assign_roles()

    outputs = {
        'service_account_email': sa_manager.get_service_account().email,
        'service_account_key': sa_manager.get_key().private_key
    }

    return MakeResult(
        outputs=outputs,
        resources={"sa_manager": sa_manager}
    )


class PulumiOutputManager:
    def __init__(self, config_manager=None):
        self.config_manager = config_manager
        self._outputs = None

    def create_outputs(self,
                      access_app,
                      app_bucket,
                      machine_images_bucket,
                      nixos_image,
                      deployment,
                      gcp_manager,
                      sa_manager,
                      secrets_manager,
                      tunnel_config,
                      tailscale_outputs,
                      tunnel_secrets,
                      tailscale_secrets) -> DeploymentOutput:
        """Create structured outputs from resources"""

        outputs = DeploymentOutput(
            access=AccessOutput(
                domain=access_app.domain,
                id=access_app.id
            ),
            storage=StorageOutput(
                app_bucket=app_bucket.url,
                machine_images=machine_images_bucket.url
            ),
            compute=ComputeOutput(
                image=ComputeImage(
                    name=nixos_image.name,
                    self_link=nixos_image.self_link,
                    info=ComputeImageInfo(
                        gcs_url=deployment.image_info["gcs_url"],
                        hash=deployment.image_info["image_hash"],
                        size=deployment.image_info["image_size"],
                        path=deployment.image_info["image_path"]
                    )
                ),
                instance=ComputeInstance(
                    name="nixos-vm",
                    external_ip=gcp_manager.get_vm_instance().network_interfaces.apply(
                        lambda interfaces: interfaces[0].access_configs[0].nat_ip if interfaces else None
                    ),
                    self_link=gcp_manager.get_vm_instance().self_link
                )
            ),
            security=SecurityOutput(
                service_account=sa_manager.get_service_account().email,
                deployment_secret=secrets_manager.deployment_secret["secret"].secret_id,
                tunnel=TunnelOutput(
                    id=tunnel_config.tunnel_id,
                    credentials=tunnel_config.credentials,
                    secrets=tunnel_secrets
                ),
                tailscale=TailscaleOutput(
                    auth_key=tailscale_outputs["current_key"],
                    expiry_date=tailscale_outputs["expiry_date"],
                    next_rotation=tailscale_outputs["next_rotation"],
                    secrets=tailscale_secrets
                )
            )
        )

        self._outputs = outputs
        return outputs

    def export_to_pulumi(self, outputs: Optional[DeploymentOutput] = None):
        """Export the outputs to Pulumi"""
        if outputs is None:
            outputs = self._outputs
        if outputs is None:
            raise ValueError("No outputs available to export")

        # Export access configuration
        pulumi.export("access", {
            "domain": outputs.access.domain,
            "id": pulumi.Output.secret(outputs.access.id)
        })

        # Export storage information
        pulumi.export("storage", {
            "app_bucket": outputs.storage.app_bucket,
            "machine_images": outputs.storage.machine_images
        })

        # Export compute information
        pulumi.export("compute", {
            "image": {
                "name": outputs.compute.image.name,
                "self_link": outputs.compute.image.self_link,
                "info": asdict(outputs.compute.image.info)
            },
            "instance": asdict(outputs.compute.instance)
        })

        # Export security information
        pulumi.export("security", {
            "service_account": pulumi.Output.secret(outputs.security.service_account),
            "deployment_secret": pulumi.Output.secret(outputs.security.deployment_secret),
            "tunnel": {
                "id": pulumi.Output.secret(outputs.security.tunnel.id),
                "credentials": pulumi.Output.secret(outputs.security.tunnel.credentials),
                "secrets": pulumi.Output.secret(outputs.security.tunnel.secrets)
            },
            "tailscale": {
                "auth_key": pulumi.Output.secret(outputs.security.tailscale.auth_key),
                "expiry_date": outputs.security.tailscale.expiry_date,
                "next_rotation": outputs.security.tailscale.next_rotation,
                "secrets": pulumi.Output.secret(outputs.security.tailscale.secrets)
            }
        })

    def save_to_file(self, file_path: str, format: str = 'json'):
        """Save the outputs to a file"""
        if self._outputs is None:
            raise ValueError("No outputs available to save")

        data = asdict(self._outputs)

        # Convert to appropriate format
        if format.lower() == 'json':
            content = json.dumps(data, indent=2)
        elif format.lower() == 'yaml':
            content = yaml.dump(data, default_flow_style=False)
        else:
            raise ValueError(f"Unsupported format: {format}")

        # Save to file
        with open(file_path, 'w') as f:
            f.write(content)

    def load_from_file(self, file_path: str) -> DeploymentOutput:
        """Load outputs from a file"""
        file_path = Path(file_path)

        with open(file_path, 'r') as f:
            if file_path.suffix == '.json':
                data = json.load(f)
            elif file_path.suffix in ['.yml', '.yaml']:
                data = yaml.safe_load(f)
            else:
                raise ValueError(f"Unsupported file format: {file_path.suffix}")

        # Reconstruct the DeploymentOutput from the loaded data
        outputs = DeploymentOutput(
            access=AccessOutput(**data['access']),
            storage=StorageOutput(**data['storage']),
            compute=ComputeOutput(
                image=ComputeImage(
                    **{**data['compute']['image'],
                       'info': ComputeImageInfo(**data['compute']['image']['info'])}
                ),
                instance=ComputeInstance(**data['compute']['instance'])
            ),
            security=SecurityOutput(
                service_account=data['security']['service_account'],
                deployment_secret=data['security']['deployment_secret'],
                tunnel=TunnelOutput(**data['security']['tunnel']),
                tailscale=TailscaleOutput(**data['security']['tailscale'])
            )
        )

        self._outputs = outputs
        return outputs

# Example usage in your Pulumi program:
def pulumi_main():
    config = pulumi.Config()

    # Your existing resource creation code here



    # Create output manager
    output_manager = PulumiOutputManager()

    # Create outputs structure
    outputs = output_manager.create_outputs(
        access_app=access_app,
        app_bucket=app_bucket,
        machine_images_bucket=machine_images_bucket,
        nixos_image=nixos_image,
        deployment=deployment,
        gcp_manager=gcp_manager,
        sa_manager=sa_manager,
        secrets_manager=secrets_manager,
        tunnel_config=tunnel_config,
        tailscale_outputs=tailscale_outputs,
        tunnel_secrets=tunnel_secrets,
        tailscale_secrets=tailscale_secrets
    )

    # Export to Pulumi
    output_manager.export_to_pulumi(outputs)

    # Optionally save to file
    output_manager.save_to_file('deployment_outputs.json')

    return output_manager
