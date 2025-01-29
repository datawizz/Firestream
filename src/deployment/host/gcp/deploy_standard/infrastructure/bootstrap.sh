
# Bootstrap

# Bootstraps everything.


# Assumptions

# You have created a Cloudflare account and have access to the API key and zone ID of a domain you own.
# The domain doesn't need to be registered by Cloudflare but it does need cloudflare DNS records for the main ingress to work.

# You have a Tailscale account and tailnet. This is used for administrative access to the host as user or root
# (depending on the configuration in the tailscale admin panel ACLs).

# Any field left blank will be assumed null and will produce an error only if required.
# The goal is to build the project end to end and to fail fast.

# 1. Create a new Project in Google Cloud Platform
    # 1.1 Create a Service account named "pulumi" and give it owner access
    # 1.2 Download the JSON key and save it in the repo root. Name it "service-account.json"

    # Enable the cloud manager API (entrypoint for all programmatic access to GCP)
    # This is not enabled by default or can be enabled remotely (confirm this)
    # https://console.cloud.google.com/apis/library/cloudresourcemanager.googleapis.com?project=project_id

    # Enable the idenity toolkit
    # This is not enabled by default or can be enabled remotely (confirm this)
    # https://console.developers.google.com/apis/api/identitytoolkit.googleapis.com/overview?project=project_id
    # This uses firebase auth in the background and cannot be enabled via the CLI as of Dev 2024

    # Enable the Identity Platform API
    # This is not enabled by default or can be enabled remotely (confirm this)
    # https://console.cloud.google.com/marketplace/details/google-cloud-platform/customer-identity?project=project_id

    # Go to settings and enable Tenants
    # This is not enabled by default or can be enabled remotely (confirm this)
    # https://console.cloud.google.com/customer-identity/tenants?project=project_id




export GOOGLE_APPLICATION_CREDENTIALS="/workspace/service-account.json"
# export CLOUDSDK_CORE_PROJECT="pleasantrees-dev"
# export GOOGLE_CLOUD_QUOTA_PROJECT="pleasantrees-dev"


### Pulumi Config ###
#
#
export PULUMI_CONFIG_PASSPHRASE="trustme"

### Real Secrets ###

pulumi config set --secret super_admin_key:cloudflare_api_token
pulumi config set --secret super_admin_key:cloudflare_zone_id
pulumi config set --secret super_admin_key:cloudflare_account_id

# Tailscale allows administrative access to the host as user or root (depending on the configuration in the tailscale admin panel ACLs)
# ACLs are not edited here but the API key is used to attach each machine to the tailnet
# #TODO improve security posture to 9000 by removing the tailscale tailnet.
pulumi config set --secret super_admin_key:tailscale_api_key
pulumi config set --secret super_admin_key:tailscale_tailnet_name

# OAuth Configuration
# This OAuth Config is used by the root domain and all tenants
# Cloudflare is also configured to accept Google Identity Platform tokens using this client id and secret
# For local development authentication is still used via Cloudflare Access and Google Identity Platform
# This allow authentic authorication even in development environments and especially in production environments and staging environments
pulumi config set --secret super_admin_key:oauth_client_id '<client-id>'
pulumi config set --secret super_admin_key:oauth_client_secret '<client-secret>'


gcloud auth activate-service-account --key-file=$GOOGLE_APPLICATION_CREDENTIALS
gcloud auth

cd /workspace/src/scripts/pulumi/deploy_standard
source venv/bin/activate
pip install -r requirements.txt

pulumi login file://$PWD/.pulumi


# Environment Variables

# These are used on all machines in all projects.
# These are translated to caps and mapped to the environment variables in the NixOS configuration
pulumi config set environment_variable:google_application_credentials '/etc/gcp/service-account.json'
pulumi config set environment_variable:machine_hostname 'prodbox'
pulumi config set environment_variable:machine_root_password 'trustme'
pulumi config set environment_variable:machine_app_user 'app'
pulumi config set environment_variable:key_rotation_days '90'


# Deployment Resources
# Each of the github repos are assumned to
# 1. Be accessible at build time
# 2. Track a branch or tag that is deployable
# 2. Supply a makefile available at the root of the project
# 2.1 Makefile should have a `build` target which reads the environment variables and builds the project pushing to the configured registry
# 2.2 Makefile should have a `deploy` target which reads the environment variables and deploys the project to the configured host
# 3.1 Makefile should run entirely within the devcontainer of this repo (assuming access to kvm and docker on the host at build time)
#
# The build script will clone the repo, checkout the branch or tag, and run `make build` and `make deploy` in the build directory
# then will copy the files and docker images in the build process of the image NixOS image.
# Wrapping in NixOS allows for a consistent build environment and a consistent deployment environment while staging the build artifacts
# in a way that is easy to deploy to the target host.



### New ###


# Base Identity Platform Configuration
pulumi config set identity_admin:plublic_domain 'example.com'
pulumi config set identity_admin:fully_qualified_domain_name 'example.com'
pulumi config set --secret identity_admin:gcp_project_id '<project-id>'
pulumi config set identity_admin:gcp_regions '["us-central1"]'
pulumi config set --secret identity_admin:oauth_client_id '<client-id>'
pulumi config set --secret identity_admin:oauth_client_secret '<client-secret>'
pulumi config set identity_admin:google_project_apis '["iam.googleapis.com", "cloudresourcemanager.googleapis.com", "storage.googleapis.com", "secretmanager.googleapis.com", "bigquery.googleapis.com", "compute.googleapis.com"]'


# Cloudflare Access Configuration
pulumi config set --secret identity_admin:cloudflare_account_id '<account-id>'
pulumi config set --secret identity_admin:cloudflare_access_client_id '<access-client-id>'
pulumi config set --secret identity_admin:cloudflare_access_client_secret '<access-client-secret>'

# Identity Platform Settings
pulumi config set identity_admin:session_duration '24h'
pulumi config set identity_admin:token_expiry '1h'
pulumi config set identity_admin:refresh_token_expiry '30d'
pulumi config set identity_admin:allowed_callback_domains '*.example.com'


### Tenant ###

# Tenant Identity Configuration
pulumi config set deploy_standard:tenant_id 'name'
pulumi config set deploy_standard:customer_name 'name'
pulumi config set deploy_standard:application_ 'name'
pulumi config set deploy_standard:customer_email_domain 'example.com'
pulumi config set deploy_standard:tenant_callback '["example.com"]'
pulumi config set deploy_standard:auth_callback_url 'https://customer.example.com/auth/callback'

# Domain Configuration
pulumi config set deploy_standard:domain_name 'example.com'
pulumi config set deploy_standard:fully_qualified_domain_name 'customer.example.com'
pulumi config set deploy_standard:deployment_name 'deploy_standard'

# GCP Infrastructure Configuration
pulumi config set deploy_standard:geographic_region 'us1'
pulumi config set deploy_standard:gcp_project_id '<project-id>'
pulumi config set deploy_standard:gcp_regions '["us-central1"]'
pulumi config set deploy_standard:gcp_region_zones '["us-central1-a"]'
pulumi config set deploy_standard:gcp_machine_type 'e2-standard-2'
pulumi config set deploy_standard:gcp_machine_node_count '1' # number of nodes

# price per node per month as of 2024-DEC
# | **Machine Type** | **Virtual CPUs** | **Memory** | **Price (USD) **    |
# |-------------------|------------------|------------|--------------------|
# | e2-standard-2     | 2                | 8 GB       | $63.99             |
# | e2-standard-4     | 4                | 16 GB      | $127.97            |
# | e2-standard-8     | 8                | 32 GB      | $255.94            |
# | e2-standard-16    | 16               | 64 GB      | $511.88            |


# Cloud Depdenencies

# Default GCP Service Account Roles
# Enables effective access to BigQuery, Cloud Storage, and Secret Manager in a ready only for critical data
pulumi config set deploy_standard:google_service_account_roles '["roles/bigquery.dataEditor", "roles/bigquery.jobUser", "roles/storage.objectViewer", "roles/storage.objectCreator", "roles/secretmanager.secretAccessor"]'
pulumi config set deploy_standard:google_project_apis '["iam.googleapis.com", "cloudresourcemanager.googleapis.com", "storage.googleapis.com", "secretmanager.googleapis.com", "bigquery.googleapis.com", "compute.googleapis.com"]'


# Build Configuration
pulumi config set deploy_standard:build_environment_production 'true' # builds the production environment
pulumi config set deploy_standard:build_environment_production_target 'google-cloud-platform' # build for google cloud platform host
# TODO create a config for AWS


## Staging Environment ##

# builds the staging environment with a copy of the production data
# NOTE this is disabled by default for cost and security reasons
pulumi config set deploy_standard:build_environment_staging 'false'
# build for google cloud platform host
pulumi config set deploy_standard:build_environment_staging_target 'google-cloud-platform'

## Development Environment ##

# builds the development environment
pulumi config set deploy_standard:build_environment_development 'true'
pulumi config set deploy_standard:build_environment_development_target 'qemu' # build for qemu virtualization host
