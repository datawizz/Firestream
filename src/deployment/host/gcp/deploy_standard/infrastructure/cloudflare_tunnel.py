import pulumi
from pulumi_cloudflare import (
    ZeroTrustTunnelCloudflared,
    ZeroTrustTunnelCloudflaredConfig,
    ZeroTrustAccessApplication,
    Record,
    Provider
)
import pulumi_random as random
import json
import yaml
from typing import Optional, Dict, NamedTuple
from pathlib import Path

class TunnelConfig(NamedTuple):
    """Configuration result from tunnel creation"""
    tunnel_id: str
    credentials: str
    tunnel_config: Dict[str, any]
    yaml_config: str

def generate_tunnel_yaml(tunnel_id: str, domain: str) -> str:
    """Generates the tunnel YAML configuration in the required format."""
    config = {
        'tunnel': tunnel_id,
        'credentials-file': '/home/app/credentials.json',
        'ingress': [
            {
                'hostname': domain,
                'service': 'http://localhost:8080',
                'access': {
                    'required': True
                }
            },
            {
                'service': 'http_status:404'
            }
        ]
    }
    return yaml.dump(config, sort_keys=False)

def create_tunnel(
    access_app: ZeroTrustAccessApplication,
    opts: Optional[pulumi.ResourceOptions] = None
) -> TunnelConfig:
    """Creates a Cloudflare tunnel with the specified configuration.

    Returns:
        TunnelConfig: Named tuple containing tunnel_id, credentials, tunnel_config, and yaml_config
    """
    config = pulumi.Config()
    cf_api_token = config.require_secret('cloudflare_api_token')
    cf_zone_id = config.require('cloudflare_zone_id')
    cf_domain = config.require('cloudflare_domain')
    cf_account_id = config.require('cloudflare_account_id')
    customer_name = config.require('customer_name')

    provider = Provider('cloudflare-provider-tunnel',
        api_token=cf_api_token
    )

    resource_opts = pulumi.ResourceOptions.merge(
        opts or pulumi.ResourceOptions(),
        pulumi.ResourceOptions(provider=provider)
    )

    tunnel_secret = random.RandomString("tunnel-secret",
        length=32,
        special=False,
        upper=True,
        numeric=True
    )

    tunnel = ZeroTrustTunnelCloudflared("main-tunnel",
        account_id=cf_account_id,
        name=f"tunnel-{customer_name}-{cf_domain}",
        secret=tunnel_secret.result,
        opts=resource_opts
    )

    # Get the team name by taking just the first part of the domain
    team_name = cf_domain.split('.')[0].split('-')[0]

    # Create the tunnel configuration
    tunnel_config_data = {
        'ingress_rule': [{
            'hostname': f"{customer_name}.{cf_domain}",
            'path': "/*",
            'service': "http://localhost:8080",
            'origin_request': {
                'access': {
                    'required': True,
                    'team_name': team_name
                }
            }
        }, {
            'service': "http_status:404"
        }]
    }

    config = ZeroTrustTunnelCloudflaredConfig("tunnel-config",
        account_id=cf_account_id,
        tunnel_id=tunnel.id,
        config=tunnel_config_data,
        opts=pulumi.ResourceOptions.merge(
            resource_opts,
            pulumi.ResourceOptions(depends_on=[tunnel, access_app], delete_before_replace=True)
        )
    )

    Record("tunnel-dns",
        zone_id=cf_zone_id,
        name=customer_name,
        type="CNAME",
        content=tunnel.id.apply(lambda id: f"{id}.cfargotunnel.com"),
        proxied=True,
        opts=resource_opts
    )

    # Generate the credentials JSON
    credentials = pulumi.Output.all(tunnel.id, tunnel_secret.result).apply(
        lambda args: json.dumps({
            "AccountTag": cf_account_id,
            "TunnelID": args[0],
            "TunnelSecret": args[1]
        }, indent=2)
    )

    # Generate the YAML configuration
    full_domain = f"{customer_name}.{cf_domain}"
    yaml_config = tunnel.id.apply(lambda id: generate_tunnel_yaml(id, full_domain))

    # Return all configurations
    return TunnelConfig(
        tunnel_id=tunnel.id,
        credentials=credentials,
        tunnel_config=tunnel_config_data,
        yaml_config=yaml_config
    )

def export_tunnel_configs(tunnel_config: TunnelConfig, output_dir: str = "tunnel_config"):
    """
    Exports the tunnel configurations to files.

    Args:
        tunnel_config: TunnelConfig object containing all configurations
        output_dir: Directory to store the configuration files
    """
    output_path = Path(output_dir)
    output_path.mkdir(parents=True, exist_ok=True)

    # # Export as Pulumi outputs
    # pulumi.export('tunnel_id', tunnel_config.tunnel_id)
    # pulumi.export('credentials', tunnel_config.credentials)
    # pulumi.export('yaml_config', tunnel_config.yaml_config)

    # The actual file writing should be done in a separate Pulumi resource
    # or during the deployment phase, as Pulumi outputs are computed values

__all__ = ['create_tunnel', 'TunnelConfig', 'export_tunnel_configs']
