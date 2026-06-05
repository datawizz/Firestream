import pulumi
from pulumi import ResourceOptions
from pulumi_gcp import compute, storage
from typing import Optional, Dict
import json

class GoogleComputeManager:
    """
    Manages GCE resources:
    - Creates a GCE custom image from a GCS-stored NixOS image
    - Launches a VM instance based on that image
    """

    def __init__(
        self,
        nixos_image: storage.BucketObject,
        image_info: pulumi.Output[dict],
        dependencies: Optional[list] = None
    ):
        """
        Initialize the manager with the given NixOS image object and related info.

        Args:
            nixos_image: The GCS BucketObject representing the uploaded NixOS image
            image_info: Pulumi Output containing image metadata
            dependencies: Optional list of resources this should depend on
        """
        self.nixos_image = nixos_image
        self.image_info = image_info
        self.dependencies = dependencies or []

        # Create the GCE image from the uploaded NixOS image
        self.compute_image = self._create_gce_image()

        # Create the GCE VM instance using the image
        self.vm_instance = self._create_gce_instance()

    def _get_gcs_url(self) -> pulumi.Output[str]:
        """
        Constructs the proper GCS URL format required by GCE.
        Returns a URL in the format: storage.googleapis.com/BUCKET_NAME/PATH
        """
        return pulumi.Output.all(
            bucket=self.nixos_image.bucket,
            name=self.nixos_image.name
        ).apply(
            lambda args: f"https://storage.googleapis.com/{args['bucket']}/{args['name']}"
        )

    def _create_gce_image(self) -> compute.Image:
        """
        Create a GCE custom image from the uploaded NixOS image in GCS.
        Ensures proper URL formatting for the source.
        """
        # Get the properly formatted GCS URL
        source_url = self._get_gcs_url()

        # Create the GCE image with the correct source URL format
        gce_image = compute.Image(
            "nixos-gce-image",
            name="nixos-image",  # Add a specific name for the image
            raw_disk=compute.ImageRawDiskArgs(
                source=source_url
            ),
            family="nixos",  # Add a family for better organization
            description="NixOS custom image",
            labels={
                "environment": "production",
                "managed-by": "pulumi"
            },
            opts=ResourceOptions(
                depends_on=self.dependencies + [self.nixos_image],
                parent=self.nixos_image  # Set parent for better resource organization
            )
        )

        # Export the image details
        pulumi.export("gce_image_name", gce_image.name)
        pulumi.export("gce_image_self_link", gce_image.self_link)

        return gce_image

    def _create_gce_instance(self) -> compute.Instance:
        """
        Create a GCE instance using the custom image.
        """
        # Machine type for the instance
        config = pulumi.Config()
        machine_type = config.require("gce_machine_type")
        gcp_region_zone = config.require("gcp_region_zone")

        # Configure the network interface
        network_interface = compute.InstanceNetworkInterfaceArgs(
            network="default",
            access_configs=[compute.InstanceNetworkInterfaceAccessConfigArgs()],
        )

        # Configure the boot disk with the custom image
        boot_disk = compute.InstanceBootDiskArgs(
            initialize_params=compute.InstanceBootDiskInitializeParamsArgs(
                image=self.compute_image.self_link,
                size=100,  # Specify disk size in GB
                type="pd-ssd"  # or "pd-ssd" for better performance
            )
        )

        # Create the instance with proper configuration
        instance = compute.Instance(
            "nixos-instance",
            name="nixos-vm",  # Specific instance name
            machine_type=machine_type,
            zone=gcp_region_zone,  # Specify your desired zone
            boot_disk=boot_disk,
            network_interfaces=[network_interface],
            labels={
                "environment": "production",
                "managed-by": "pulumi"
            },
            metadata={
                "startup-script": """#!/bin/bash
                echo "Instance started successfully" > /var/log/startup.log
                """
            },
            opts=ResourceOptions(
                depends_on=[self.compute_image],
                parent=self.compute_image
            )
        )

        # # Export instance details
        # pulumi.export("vm_instance_name", instance.name)
        # pulumi.export("vm_instance_self_link", instance.self_link)
        # pulumi.export("vm_instance_external_ip",
        #     instance.network_interfaces.apply(
        #         lambda interfaces: interfaces[0].access_configs[0].nat_ip if interfaces else None
        #     )
        # )

        return instance

    def get_compute_image(self) -> compute.Image:
        """Get the created GCE image."""
        return self.compute_image

    def get_vm_instance(self) -> compute.Instance:
        """Get the created VM instance."""
        return self.vm_instance
