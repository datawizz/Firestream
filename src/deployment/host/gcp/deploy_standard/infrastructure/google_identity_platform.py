import pulumi
import pulumi_gcp as gcp
from typing import List, Optional
from dataclasses import dataclass

@dataclass
class IdentityConfig:
    """Configuration for Identity Platform settings"""
    project_id: str
    allowed_domains: List[str]
    google_client_id: str
    google_client_secret: str
    application_title: str = "Secure Application"

class IdentityPlatformManager:
    """Manages Google Identity Platform resources"""
    def __init__(self, config: IdentityConfig, opts: Optional[pulumi.ResourceOptions] = None):
        self.config = config
        self.project_id = config.project_id
        self.opts = opts

        # Create a GCP provider
        self.gcp_provider = gcp.Provider(
            'gcp-provider',
            project=self.project_id
        )

        # Enable Identity Toolkit API
        self.identity_toolkit_api = gcp.projects.Service(
            "enable-identitytoolkit-api",
            project=self.project_id,
            service="identitytoolkit.googleapis.com",
            disable_dependent_services=True,
            opts=pulumi.ResourceOptions(provider=self.gcp_provider)
        )

        # Set up the core resources
        # self.platform_config = self._create_platform_config()
        self.tenant = self._create_tenant()
        self.oauth_config = self._configure_google_oauth()
        self.default_idp_config = self._configure_default_idp()

    # def _create_platform_config(self) -> gcp.identityplatform.Config:
    #     """Create the base Identity Platform configuration"""
    #     return gcp.identityplatform.Config(
    #         "identity-platform-config",
    #         project=self.project_id,
    #         authorized_domains=self.config.allowed_domains,
    #         sign_in=gcp.identityplatform.ConfigSignInArgs(
    #             allow_duplicate_emails=False,
    #             email=gcp.identityplatform.ConfigSignInEmailArgs(
    #                 enabled=True,
    #                 password_required=True
    #             ),
    #             anonymous=gcp.identityplatform.ConfigSignInAnonymousArgs(
    #                 enabled=False
    #             )
    #         ),
    #         opts=pulumi.ResourceOptions(
    #             provider=self.gcp_provider,
    #             depends_on=[self.identity_toolkit_api]
    #         )
    #     )

    def _create_tenant(self) -> gcp.identityplatform.Tenant:
        """Create an Identity Platform tenant"""
        return gcp.identityplatform.Tenant(
            "identity-tenant",
            project=self.project_id,
            display_name=self.config.application_title,
            allow_password_signup=True,
            enable_email_link_signin=False,
            disable_auth=False,
            opts=pulumi.ResourceOptions(
                provider=self.gcp_provider,
                # depends_on=[self.platform_config]
            )
        )

    def _configure_google_oauth(self) -> gcp.identityplatform.OauthIdpConfig:
        """Configure Google OAuth Identity Provider"""
        oauth_id = "oidc.oauth-idp-config"  # Added compliant OAuth config ID
        return gcp.identityplatform.OauthIdpConfig(
            resource_name=oauth_id,
            project=self.project_id,
            display_name="Google",
            client_id=self.config.google_client_id,
            client_secret=self.config.google_client_secret,
            issuer="https://accounts.google.com",
            enabled=True,
            opts=pulumi.ResourceOptions(
                provider=self.gcp_provider,
                depends_on=[self.tenant]
            )
        )

    def _configure_default_idp(self) -> gcp.identityplatform.DefaultSupportedIdpConfig:
        """Configure default supported identity provider"""
        return gcp.identityplatform.DefaultSupportedIdpConfig(
            "google",  # Changed to match expected provider ID
            project=self.project_id,
            client_id=self.config.google_client_id,
            client_secret=self.config.google_client_secret,
            idp_id="google.com",  # Changed to correct provider ID
            enabled=True,
            opts=pulumi.ResourceOptions(
                provider=self.gcp_provider,
                # depends_on=[self.platform_config]
            )
        )

    def export_values(self) -> None:
        """Export relevant values as Pulumi outputs"""
        pulumi.export("identity_platform_tenant_name", self.tenant.name)
        pulumi.export("identity_platform_project_id", self.project_id)
        pulumi.export("identity_platform_oauth_idp", self.oauth_config.name)
        pulumi.export("identity_platform_default_idp", self.default_idp_config.name)
        pulumi.export("identity_authorized_domains", self.config.allowed_domains)
