

from dataclasses import dataclass
from typing import List, Optional
import re
from pathlib import Path


@dataclass
class EnvironmentVariables:
    """
    These are set as environment variables in the built virtual machine image.
    """
    google_application_credentials: str
    machine_hostname: str
    machine_root_password: str
    machine_app_user: str
    key_rotation_days: str


@dataclass
class GithubRepo:
    """
    Each instance of this class represents a GitHub repository.
    Each
    """
    url: str
    branch: str
    commit: str


@dataclass
class SoftwareDeployment:
    github_repo_1: GithubRepo
    github_repo_2: GithubRepo
    github_repo_3: GithubRepo


@dataclass
class IdentityAdmin:
    public_domain: str
    fully_qualified_domain_name: str
    gcp_project_id: str
    gcp_regions: List[str]
    oauth_client_id: str
    oauth_client_secret: str
    google_project_apis: List[str]
    cloudflare_account_id: str
    cloudflare_access_client_id: str
    cloudflare_access_client_secret: str
    session_duration: str
    token_expiry: str
    refresh_token_expiry: str
    allowed_callback_domains: str


@dataclass
class DeployStandard:
    tenant_id: str
    customer_name: str
    application: str
    customer_email_domain: str
    tenant_callback: List[str]
    auth_callback_url: str
    domain_name: str
    fully_qualified_domain_name: str
    deployment_name: str
    gcp_project_id: str
    geographic_region: str
    gcp_regions: List[str]
    gcp_region_zones: List[str]
    gcp_machine_type: str
    gcp_machine_node_count: str
    google_service_account_roles: List[str]
    google_project_apis: List[str]
    build_environment_production: str
    build_environment_production_target: str
    build_environment_development: str
    build_environment_development_target: str


@dataclass
class PulumiConfig:
    environment_variables: EnvironmentVariables
    software_deployment: SoftwareDeployment
    identity_admin: IdentityAdmin
    deploy_standard: DeployStandard


class ConfigManager:
    def __init__(self):
        self._config = None

    def _parse_list(self, value: str) -> List[str]:
        """Parse a string representation of a list into a Python list."""
        if not value:
            return []
        # Handle both string and list inputs
        if value.startswith('[') and value.endswith(']'):
            # Remove brackets and split by comma
            cleaned = value.strip('[]')
            # Split by comma but handle quoted strings properly
            items = re.findall(r'\"([^\"]*)\"|\'([^\']*)\'|([^,\s]+)', cleaned)
            # Take the first non-empty group from each match
            return [next(s for s in item if s) for item in items]
        else:
            # Single value
            return [value.strip('"').strip("'")]

    def _parse_bootstrap_line(self, line: str) -> tuple[str, str, str]:
        """Parse a line from bootstrap.sh file."""
        # Match both regular and secret config sets
        pattern = r'pulumi\s+config\s+set\s+(?:--secret\s+)?([^:]+):([^\s]+)\s+[\'"](.*?)[\'"]'
        match = re.match(pattern, line)
        if match:
            namespace, key, value = match.groups()
            return namespace, key, value
        return None, None, None

    def load_from_bootstrap(self, file_path: str) -> "PulumiConfig":
        """Load configuration from a bootstrap.sh file."""
        config_dict = {
            "environment_variable": {},
            "software_deployment": {},
            "identity_admin": {},
            "deploy_standard": {}
        }

        # Read the entire file content
        with open(file_path, 'r') as f:
            content = f.read()

        # Find all pulumi config set commands, including multiline
        pattern = r'pulumi\s+config\s+set\s+(?:--secret\s+)?([^:]+):([^\s]+)\s+[\'"](.*?)[\'"]\s*(?:\s+#.*)?(?:\n|$)'
        matches = re.finditer(pattern, content, re.MULTILINE)

        for match in matches:
            namespace, key, value = match.groups()
            if namespace and key:
                if namespace not in config_dict:
                    config_dict[namespace] = {}
                config_dict[namespace][key] = value
                print(f"Parsed: {namespace}:{key} = {value}")

        return self._create_config_from_dict(config_dict)

    def load_from_pulumi(self) -> "PulumiConfig":
        """Load configuration from Pulumi."""
        try:
            import pulumi
            config_dict = {
                "environment_variable": {},
                "software_deployment": {},
                "identity_admin": {},
                "deploy_standard": {}
            }

            # For each namespace, get all configurations
            for namespace in config_dict.keys():
                config = pulumi.Config(namespace)
                # Note: This is a simplified version. In reality, you'd need to
                # handle getting all keys from Pulumi config which might require
                # different approach depending on Pulumi's API
                for key in self._get_expected_keys(namespace):
                    try:
                        value = config.get(key)
                        if value:
                            config_dict[namespace][key] = value
                    except:
                        pass

            return self._create_config_from_dict(config_dict)
        except ImportError:
            raise ImportError("Pulumi package is required to load from Pulumi")

    def _get_expected_keys(self, namespace: str) -> List[str]:
        """Get expected keys for each namespace based on dataclass fields."""
        mapping = {
            "environment_variable": EnvironmentVariables,
            "software_deployment": SoftwareDeployment,
            "identity_admin": IdentityAdmin,
            "deploy_standard": DeployStandard
        }
        if namespace in mapping:
            return [field.name for field in mapping[namespace].__dataclass_fields__]
        return []

    def _create_config_from_dict(self, config_dict: dict) -> PulumiConfig:
        """Create a PulumiConfig instance from a dictionary."""
        env_vars = EnvironmentVariables(
            google_application_credentials=config_dict["environment_variable"].get("google_application_credentials", ""),
            machine_hostname=config_dict["environment_variable"].get("machine_hostname", ""),
            machine_root_password=config_dict["environment_variable"].get("machine_root_password", ""),
            machine_app_user=config_dict["environment_variable"].get("machine_app_user", ""),
            key_rotation_days=config_dict["environment_variable"].get("key_rotation_days", "")
        )

        software_deployment = SoftwareDeployment(
            github_repo_1=GithubRepo(
                url=config_dict["software_deployment"].get("github_repo_1_url", ""),
                branch=config_dict["software_deployment"].get("github_repo_1_branch", ""),
                commit=config_dict["software_deployment"].get("github_repo_1_commit", "")
            ),
            github_repo_2=GithubRepo(
                url=config_dict["software_deployment"].get("github_repo_2_url", ""),
                branch=config_dict["software_deployment"].get("github_repo_2_branch", ""),
                commit=config_dict["software_deployment"].get("github_repo_2_commit", "")
            ),
            github_repo_3=GithubRepo(
                url=config_dict["software_deployment"].get("github_repo_3_url", ""),
                branch=config_dict["software_deployment"].get("github_repo_3_branch", ""),
                commit=config_dict["software_deployment"].get("github_repo_3_commit", "")
            )
        )

        identity_admin = IdentityAdmin(
            public_domain=config_dict["identity_admin"].get("plublic_domain", ""),  # Note: typo in original
            fully_qualified_domain_name=config_dict["identity_admin"].get("fully_qualified_domain_name", ""),
            gcp_project_id=config_dict["identity_admin"].get("gcp_project_id", ""),
            gcp_regions=self._parse_list(config_dict["identity_admin"].get("gcp_regions", "[]")),
            oauth_client_id=config_dict["identity_admin"].get("oauth_client_id", ""),
            oauth_client_secret=config_dict["identity_admin"].get("oauth_client_secret", ""),
            google_project_apis=self._parse_list(config_dict["identity_admin"].get("google_project_apis", "[]")),
            cloudflare_account_id=config_dict["identity_admin"].get("cloudflare_account_id", ""),
            cloudflare_access_client_id=config_dict["identity_admin"].get("cloudflare_access_client_id", ""),
            cloudflare_access_client_secret=config_dict["identity_admin"].get("cloudflare_access_client_secret", ""),
            session_duration=config_dict["identity_admin"].get("session_duration", ""),
            token_expiry=config_dict["identity_admin"].get("token_expiry", ""),
            refresh_token_expiry=config_dict["identity_admin"].get("refresh_token_expiry", ""),
            allowed_callback_domains=config_dict["identity_admin"].get("allowed_callback_domains", "")
        )

        deploy_standard = DeployStandard(
            tenant_id=config_dict["deploy_standard"].get("tenant_id", ""),
            customer_name=config_dict["deploy_standard"].get("customer_name", ""),
            application=config_dict["deploy_standard"].get("application_", ""),
            customer_email_domain=config_dict["deploy_standard"].get("customer_email_domain", ""),
            tenant_callback=self._parse_list(config_dict["deploy_standard"].get("tenant_callback", "[]")),
            auth_callback_url=config_dict["deploy_standard"].get("auth_callback_url", ""),
            domain_name=config_dict["deploy_standard"].get("domain_name", ""),
            fully_qualified_domain_name=config_dict["deploy_standard"].get("fully_qualified_domain_name", ""),
            deployment_name=config_dict["deploy_standard"].get("deployment_name", ""),
            gcp_project_id=config_dict["deploy_standard"].get("gcp_project_id", ""),
            geographic_region=config_dict["deploy_standard"].get("geographic_region", ""),
            gcp_regions=self._parse_list(config_dict["deploy_standard"].get("gcp_regions", "[]")),
            gcp_region_zones=self._parse_list(config_dict["deploy_standard"].get("gcp_region_zones", "[]")),
            gcp_machine_type=config_dict["deploy_standard"].get("gcp_machine_type", ""),
            gcp_machine_node_count=config_dict["deploy_standard"].get("gcp_machine_node_count", ""),
            google_service_account_roles=self._parse_list(config_dict["deploy_standard"].get("google_service_account_roles", "[]")),
            google_project_apis=self._parse_list(config_dict["deploy_standard"].get("google_project_apis", "[]")),
            build_environment_production=config_dict["deploy_standard"].get("build_environment_production", ""),
            build_environment_production_target=config_dict["deploy_standard"].get("build_environment_production_target", ""),
            build_environment_development=config_dict["deploy_standard"].get("build_environment_development", ""),
            build_environment_development_target=config_dict["deploy_standard"].get("build_environment_development_target", "")
        )

        return PulumiConfig(
            environment_variables=env_vars,
            software_deployment=software_deployment,
            identity_admin=identity_admin,
            deploy_standard=deploy_standard
        )


# Example usage:
if __name__ == "__main__":
    import os
    import sys

    config_manager = ConfigManager()

    # Get the directory containing the current script
    current_dir = os.path.dirname(os.path.abspath(__file__))

    # Look for paste.txt in the current directory
    bootstrap_file = os.path.join(current_dir, "bootstrap.sh")
    if not os.path.exists(bootstrap_file):
        print(f"Error: Could not find {bootstrap_file}")
        sys.exit(1)

    print(f"Loading configuration from {bootstrap_file}")
    config = config_manager.load_from_bootstrap(bootstrap_file)
    print("\nFinal configuration:")
    print("Environment Variables:")
    for field in config.environment_variables.__dataclass_fields__:
        value = getattr(config.environment_variables, field)
        print(f"  {field}: {value}")

    print("\nSoftware Deployment:")
    for i, repo in enumerate([config.software_deployment.github_repo_1,
                            config.software_deployment.github_repo_2,
                            config.software_deployment.github_repo_3], 1):
        print(f"  Repo {i}:")
        print(f"    URL: {repo.url}")
        print(f"    Branch: {repo.branch}")
        print(f"    Commit: {repo.commit}")

    # Load from Pulumi (when running in Pulumi context)
    # config = config_manager.load_from_pulumi()
    # print("Loaded from Pulumi:", config)
