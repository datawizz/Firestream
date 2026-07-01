import pulumi
from pulumi_gcp import serviceaccount, projects, storage, identityplatform
from typing import List, Optional

class GCPProjectManager:
    def __init__(self, project_id: str, service_account_id: str):
        """
        Initialize GCP Project Manager with required APIs and Identity Platform setup.

        Args:
            project_id: GCP project ID
            service_account_id: Service account identifier
        """
        self.project_id = project_id
        self.service_account_id = service_account_id
        self.service_account = None
        self.key = None
        self.role_bindings = []
        self.identity_platform_config = None
        self.tenant_config = None

        # Enable Service Usage API first
        self.service_usage_api = self._enable_service_usage_api()

        # Enable other required APIs with dependency on Service Usage API
        self.enabled_apis = self._enable_required_apis()

        # Initialize resources with dependency on API enablement
        self._create_service_account()
        self._create_service_account_key()
        self._setup_identity_platform()

    def _enable_service_usage_api(self) -> projects.Service:
        """Enable the Service Usage API first."""
        return projects.Service(
            "enable-serviceusage.googleapis.com",
            project=self.project_id,
            service="serviceusage.googleapis.com",
            disable_dependent_services=False,
            disable_on_destroy=False
        )

    def _enable_required_apis(self) -> List[projects.Service]:
        """Enable all required GCP APIs with dependency on Service Usage API."""
        required_apis = [
            "iam.googleapis.com",
            "cloudresourcemanager.googleapis.com",
            "storage.googleapis.com",
            "secretmanager.googleapis.com",
            "bigquery.googleapis.com",
            "compute.googleapis.com",
            "identitytoolkit.googleapis.com",  # Required for Firebase Auth
            "cloudidentity.googleapis.com",    # Required for Identity Platform
        ]

        enabled_apis = []
        for api in required_apis:
            service = projects.Service(
                f"enable-{api}",
                project=self.project_id,
                service=api,
                disable_dependent_services=False,
                disable_on_destroy=False,
                opts=pulumi.ResourceOptions(depends_on=[self.service_usage_api])
            )
            enabled_apis.append(service)

        return enabled_apis

    def _create_service_account(self) -> None:
        """Create the service account with dependency on API enablement."""
        self.service_account = serviceaccount.Account(
            "service-account",
            account_id=self.service_account_id,
            display_name=f"{self.service_account_id} Service Account",
            project=self.project_id,
            opts=pulumi.ResourceOptions(depends_on=[
                self.service_usage_api,
                *self.enabled_apis
            ])
        )

    def _create_service_account_key(self) -> None:
        """Create service account key with proper dependencies."""
        if not self.service_account:
            raise ValueError("Service account must be created before creating a key")

        self.key = serviceaccount.Key(
            "service-account-key",
            service_account_id=self.service_account.name,
            public_key_type="TYPE_X509_PEM_FILE",
            private_key_type="TYPE_GOOGLE_CREDENTIALS_FILE",
            opts=pulumi.ResourceOptions(depends_on=[self.service_account])
        )

    def _setup_identity_platform(self) -> None:
        """Setup Identity Platform configuration and enable multi-tenancy."""
        # Note: Some Identity Platform features may require manual setup in the console
        # as they are not fully supported via API/Pulumi as of Dec 2024

        # Create Identity Platform Config
        self.identity_platform_config = identityplatform.Config(
            "identity-platform-config",
            project=self.project_id,
            opts=pulumi.ResourceOptions(depends_on=[
                *self.enabled_apis
            ])
        )

        # Note: Tenant creation might need to be done manually through the console
        # We can add a reminder in the outputs
        pulumi.export("identity_platform_setup_required",
                     "Please complete Identity Platform setup in the console:\n" +
                     f"1. Visit https://console.cloud.google.com/customer-identity/tenants?project={self.project_id}\n" +
                     "2. Enable multi-tenancy in the settings\n" +
                     "3. Create required tenants")

    def assign_roles(self, additional_roles: Optional[List[str]] = None) -> None:
        """
        Assign roles to the service account.

        Args:
            additional_roles: Optional list of additional roles to assign
        """
        if not self.service_account:
            raise ValueError("Service account must be created before assigning roles")

        default_roles = [
            'roles/owner',  # As requested for the pulumi service account
            'roles/bigquery.dataEditor',
            'roles/bigquery.jobUser',
            'roles/storage.objectViewer',
            'roles/storage.objectCreator',
            'roles/secretmanager.secretAccessor',
            'roles/identityplatform.admin'  # Added for Identity Platform management
        ]

        roles_to_assign = default_roles + (additional_roles or [])

        for role in roles_to_assign:
            binding = projects.IAMBinding(
                f"iam-binding-{role.split('/')[-1]}",
                project=self.project_id,
                role=role,
                members=[pulumi.Output.concat("serviceAccount:", self.service_account.email)],
                opts=pulumi.ResourceOptions(depends_on=[
                    self.service_account,
                    self.service_usage_api,
                    *self.enabled_apis
                ])
            )
            self.role_bindings.append(binding)

    def get_service_account(self) -> serviceaccount.Account:
        """Get the created service account."""
        if not self.service_account:
            raise ValueError("Service account has not been created yet")
        return self.service_account

    def get_key(self) -> serviceaccount.Key:
        """Get the created service account key."""
        if not self.key:
            raise ValueError("Service account key has not been created yet")
        return self.key

    def get_role_bindings(self) -> List[projects.IAMBinding]:
        """Get the list of role bindings."""
        return self.role_bindings

# Example usage:
"""
import pulumi
from gcp_project_manager import GCPProjectManager

def main():
    # Initialize the project manager
    project_manager = GCPProjectManager(
        project_id="your-project-id",
        service_account_id="pulumi"
    )

    # Assign roles (including owner access)
    project_manager.assign_roles()

    # Export key for future use
    pulumi.export('service_account_key', project_manager.get_key().private_key)

if __name__ == "__main__":
    main()
"""
