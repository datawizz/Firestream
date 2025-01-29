#!/usr/bin/env python3
"""
Test script for NixOS Image Builder
Tests both GCP and QEMU image builds
"""

import logging
from pathlib import Path
from datetime import datetime

from build import NixImageBuilder, BuildArguments, BuildType, NixBuildError

# Configure logging
logging.basicConfig(
    level=logging.INFO,
    format='%(asctime)s - %(levelname)s - %(message)s'
)
logger = logging.getLogger(__name__)

def test_builds():
    """Run test builds for both GCP and QEMU images"""

    # Initialize builder
    builder = NixImageBuilder(workspace_dir="/workspace")

    # Generate unique hostnames using timestamp
    timestamp = datetime.now().strftime("%Y%m%d-%H%M")

    # Create build arguments for GCP
    gcp_args = BuildArguments(
        hostname=f"test-gcp-{timestamp}",
        build_type=BuildType.GCP,
        extra_args={
            "google_application_credentials": "/path/to/credentials.json"
        }
    )

    # Create build arguments for QEMU
    qemu_args = BuildArguments(
        hostname=f"test-qemu-{timestamp}",
        build_type=BuildType.QEMU,
        extra_args={}
    )

    try:
        # Build GCP image
        logger.info("Building GCP image...")
        gcp_result = builder.build_image(
            nix_file="default.nix",
            build_args=gcp_args
        )
        logger.info(f"GCP build successful: {gcp_result.image_path}")

        # Build QEMU image
        logger.info("Building QEMU image...")
        qemu_result = builder.build_image(
            nix_file="default.nix",
            build_args=qemu_args
        )
        logger.info(f"QEMU build successful: {qemu_result.image_path}")

        return True

    except NixBuildError as e:
        logger.error(f"Build failed: {e}")
        return False
    except Exception as e:
        logger.error(f"Unexpected error: {e}")
        return False

if __name__ == "__main__":
    success = test_builds()
    exit(0 if success else 1)
