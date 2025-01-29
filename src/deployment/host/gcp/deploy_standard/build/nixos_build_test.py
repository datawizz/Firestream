#!/usr/bin/env python3

import os
import json
import tempfile
from pathlib import Path
from typing import NamedTuple, Optional
from contextlib import contextmanager

from nixos_build import NixImageBuilder, NixBuildError, BuildResult, HostType

class TestCredentials(NamedTuple):
    """Test credential files with cleanup handling"""
    service_account_path: Optional[Path]
    temp_dir: Path
    hostname: str

@contextmanager
def create_test_credentials(host_type: HostType) -> TestCredentials:
    """
    Creates temporary credential files for testing.
    Uses context manager for automatic cleanup.

    Args:
        host_type: The type of host being built ("qemu" or "gcp")
    """
    temp_dir = None
    try:
        # Create temporary directory
        temp_dir = Path(tempfile.mkdtemp(prefix='nixos-test-'))
        hostname = f"test-{host_type}-host"

        # Always create service account credentials for testing
        service_account = {
            "type": "service_account",
            "project_id": "test-project",
            "private_key_id": "test-key-id",
            "private_key": "-----BEGIN PRIVATE KEY-----\ntest-key\n-----END PRIVATE KEY-----\n",
            "client_email": "test@test-project.iam.gserviceaccount.com",
            "client_id": "test-client-id",
            "auth_uri": "https://accounts.google.com/o/oauth2/auth",
            "token_uri": "https://oauth2.googleapis.com/token",
            "auth_provider_x509_cert_url": "https://www.googleapis.com/oauth2/v1/certs",
            "client_x509_cert_url": "https://www.googleapis.com/robot/v1/metadata/x509/test"
        }

        # Create and write service account file
        sa_path = temp_dir / 'service-account.json'
        sa_path.write_text(json.dumps(service_account, indent=2))
        os.chmod(sa_path, 0o600)

        yield TestCredentials(
            service_account_path=sa_path,
            temp_dir=temp_dir,
            hostname=hostname
        )

    finally:
        # Cleanup temporary directory and files
        if temp_dir and temp_dir.exists():
            for file in temp_dir.glob('*'):
                if file.is_file():
                    os.chmod(str(file), 0o600)  # Ensure we can delete the file
                    file.unlink()
            temp_dir.rmdir()

def run_test_build(config_path: Optional[str] = None, host_type: HostType = "qemu") -> None:
    """
    Run a test build of the NixOS image.

    Args:
        config_path: Optional path to configuration.nix. If not provided,
                    uses default test path.
        host_type: Type of host to build for ("qemu" or "gcp")
    """
    default_config = "/workspace/src/scripts/pulumi/deploy_standard/build/configuration.nix"
    nix_config = config_path or default_config

    print(f"Starting test build for {host_type} with configuration: {nix_config}")

    with create_test_credentials(host_type) as creds:
        cred_info = ["Test credentials created:"]
        if creds.service_account_path:
            cred_info.append(f"- Service Account: {creds.service_account_path}")
        cred_info.append(f"- Hostname: {creds.hostname}")
        print("\n".join(cred_info))

        try:
            # Initialize builder with test credentials
            extra_args = {'hostname': creds.hostname}
            if creds.service_account_path:
                extra_args['serviceAccountFile'] = str(creds.service_account_path)

            builder = NixImageBuilder(
                workspace_dir="/workspace",
                build_jobs=10,
                extra_args=extra_args
            )

            # Build the image
            result = builder.build_image(nix_config, host_type=host_type)

            if result.success:
                print(f"""
Image built successfully:
- Type: {result.host_type}
- Path: {result.image_path}
- Name: {result.image_name}
- Size: {result.file_size/1024/1024:.2f} MB
- Hash: {result.file_hash}
        """)
            else:
                print("Image build failed")
                exit(1)

        except NixBuildError as e:
            print(f"Failed to build image: {e}")
            exit(1)
        except Exception as e:
            print(f"Unexpected error during build: {e}")
            exit(1)

if __name__ == "__main__":
    import argparse

    parser = argparse.ArgumentParser(description='Test NixOS image builder')
    parser.add_argument('--config', type=str, help='Path to configuration.nix file')
    parser.add_argument('--type', type=str, choices=['qemu', 'gcp'], default='qemu',
                        help='Type of image to build (qemu or gcp)')

    args = parser.parse_args()

    run_test_build(args.config, args.type)
