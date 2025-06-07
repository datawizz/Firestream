import pulumi
from pulumi_gcp import serviceaccount, projects, storage
from typing import List, Optional

class ServiceAccountManager:
    def __init__(self, project_id: str, service_account_id: str):
        """
        Initialize ServiceAccountManager with required APIs enabled.

        Args:
            project_id: GCP project ID
            service_account_id: Service account identifier
        """
        self.project_id = project_id
        self.service_account_id = service_account_id
        self.service_account = None
        self.key = None
        self.role_bindings = []

        # Enable Service Usage API first
        self.service_usage_api = self._enable_service_usage_api()

        # Enable other required APIs with dependency on Service Usage API
        self.enabled_apis = self._enable_required_apis()

        # Initialize resources with dependency on API enablement
        self._create_service_account()
        self._create_service_account_key()

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
            "compute.googleapis.com"
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

    def assign_roles(self, additional_roles: Optional[List[str]] = None) -> None:
        """
        Assign roles to the service account.

        Args:
            additional_roles: Optional list of additional roles to assign
        """
        if not self.service_account:
            raise ValueError("Service account must be created before assigning roles")

        default_roles = [
            'roles/bigquery.dataEditor',
            'roles/bigquery.jobUser',
            'roles/storage.objectViewer',
            'roles/storage.objectCreator',
            'roles/secretmanager.secretAccessor'
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
