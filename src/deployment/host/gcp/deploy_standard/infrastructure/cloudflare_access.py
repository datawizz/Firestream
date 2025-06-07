import pulumi
from pulumi_cloudflare import (
    ZeroTrustAccessApplication,
    ZeroTrustAccessPolicy,
    Provider
)
from typing import List, Optional

def create_access_config(
    allowed_domains: List[str],
    session_duration: str = "24h",
    opts: Optional[pulumi.ResourceOptions] = None
) -> ZeroTrustAccessApplication:
    # Get configuration from Pulumi config
    config = pulumi.Config()
    cf_account_id = config.require('cloudflare_account_id')
    cf_domain = config.require('cloudflare_domain')
    cf_api_token = config.require_secret('cloudflare_api_token')
    cf_zone_id = config.require('cloudflare_zone_id')
    customer_name = config.require('customer_name')
    customer_email_domain = config.require('customer_email_domain') # e.g., enjoypleasantrees.com

    # Create provider
    provider = Provider('cloudflare-provider-access',
        api_token=cf_api_token,
    )

    provider_opts = pulumi.ResourceOptions.merge(
        opts or pulumi.ResourceOptions(),
        pulumi.ResourceOptions(provider=provider)
    )

    # Get the team name by taking just the first part of the domain
    # e.g., "centerpoint-consulting.com" -> "centerpoint"
    team_name = cf_domain.split('.')[0].split('-')[0]


    app = ZeroTrustAccessApplication("protected-app",
        account_id=cf_account_id,
        name=team_name,
        domain=f"{customer_name}.{cf_domain}",
        type="self_hosted",
        session_duration=session_duration,
        auto_redirect_to_identity=True,
        opts=provider_opts
    )

    ZeroTrustAccessPolicy("allow-policy",
        account_id=cf_account_id,
        application_id=app.id,
        name="Allow OTP Authenticated Users",
        precedence=1,
        decision="allow",
        # "includes" define WHO can try to access
        includes=[{
            "email_domains": allowed_domains,
        }],
        # "requires" define HOW they must authenticate
        requires=[{
            "auth_method": "email",  # This enforces an email-based OTP challenge
            "email_domains": allowed_domains
        }],
        session_duration="24h",
        opts=provider_opts
    )

    # Create fallback deny policy with lower precedence
    ZeroTrustAccessPolicy("deny-policy",
        account_id=cf_account_id,
        application_id=app.id,
        name="Deny Unauthorized",
        precedence=2,
        decision="deny",
        includes=[{"everyone": True}],
        opts=provider_opts
    )

    return app
