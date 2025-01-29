#!/usr/bin/env python3
import os
import sys
import json
import logging
from pathlib import Path
from typing import Optional
from nixos_build import NixImageBuilder, NixBuildError, BuildResult

if __name__ == "__main__":
    # Basic argument parsing
    # We assume the script can be called as:
    # python3 build_image.py --sa-key-content "$SERVICE_ACCOUNT_JSON" --hostname "$HOSTNAME" --config /path/to/configuration.nix
    # Or we can pass them as environment variables.
    import argparse

    parser = argparse.ArgumentParser(description="Build NixOS Image CLI")
    parser.add_argument("--sa-key-content", type=str, required=True, help="Service account key JSON content")
    parser.add_argument("--hostname", type=str, required=True, help="The hostname of the machine")
    parser.add_argument("--config", type=str, default="/workspace/src/scripts/pulumi/deploy_standard/build/configuration.nix", help="Path to configuration.nix")
    parser.add_argument("--workspace-dir", type=str, default="/workspace", help="Workspace directory")
    args = parser.parse_args()

    workspace_dir = Path(args.workspace_dir)
    build_dir = workspace_dir / "_build"
    creds_dir = build_dir / "creds"
    creds_dir.mkdir(parents=True, exist_ok=True)

    # Write credentials directly to the build directory
    sa_path = creds_dir / "service-account.json"

    sa_path.write_text(args.sa_key_content)
    os.chmod(sa_path, 0o600)

    # Configure logging
    logging.basicConfig(level=logging.DEBUG, format='%(asctime)s - %(levelname)s - %(message)s')
    logger = logging.getLogger(__name__)

    # Initialize builder
    builder = NixImageBuilder(
        workspace_dir=str(workspace_dir),
        build_jobs=10,
        extra_args={
            'serviceAccountFile': str(sa_path),
            'hostname': args.hostname
        }
    )

    try:
        # Build the image
        result = builder.build_image(args.config)

        # Output JSON result
        if result.success:
            output = {
                "image_path": str(result.image_path),
                "image_name": result.image_name,
                "file_size": result.file_size,
                "file_hash": result.file_hash,
                "hostname": result.hostname
            }
            print(json.dumps(output))
        else:
            logger.error("Build result reported as not successful")
            sys.exit(1)

    except NixBuildError as e:
        logger.error(f"Failed to build image: {e}")
        sys.exit(1)
    except Exception as e:
        logger.error(f"Unexpected error during build: {e}")
        sys.exit(1)
