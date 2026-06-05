
import pulumi
from pulumi import ResourceOptions, Output
from typing import List, Optional
import pulumi_command as command
from pulumi_gcp import storage
import json
import uuid
import infrastructure.google_cloud_storage as gcs

class NixOSDeployment:
    """Handles NixOS image building and deployment to GCP via pulumi_command"""

    def __init__(self):
        self.nixos_image: storage.BucketObject = None
        self.machine_images_bucket = gcs.bucket1
        self.app_bucket = gcs.bucket2
        # Store hostname as a class property so it can be accessed later
        self.hostname = None

    def create_image_on_up(
        self,
        service_account_key: pulumi.Output[str],
        dependencies: Optional[List[pulumi.Resource]] = None
    ) -> storage.BucketObject:
        """
        This method runs a local Python script (e.g., build_image.py) that:
        - Takes service account as input.
        - Builds the NixOS image.
        - Prints JSON output with image metadata.
        Then we use that metadata to create a GCS BucketObject.
        """

        # Generate Hostname
        # Format: prodbox-{CUSTOMER_NAME}-{NODE_ID}

        config = pulumi.Config()
        customer_name = config.require("customer_name")
        node_id = 1 # For now, we only have one node #TODO add more nodes dynamically

        self.hostname = f"prodbox-{customer_name}-{node_id}"

        # Export the hostname
        pulumi.export('nixos_hostname', self.hostname)

        # We use the `local.Command` resource to run the build script
        build_command = command.local.Command(
            "nixosImageBuildCommand",
            create=pulumi.Output.all(service_account_key, self.hostname).apply(
                lambda args: f"python /workspace/src/scripts/pulumi/deploy_standard/build/nixos_build_image.py "
                f"--sa-key-content '{args[0]}' "
                f"--hostname {self.hostname} "
                f"--config /workspace/src/scripts/pulumi/deploy_standard/build/configuration.nix"
            ),
            environment={
                "PYTHONUNBUFFERED": "1"  # ensure output is not buffered
            },
            opts=ResourceOptions(
                depends_on=[self.machine_images_bucket] + (dependencies or []),
                delete_before_replace=True,
                custom_timeouts=pulumi.CustomTimeouts(create='30m', update='30m')
            )
        )

        # Parse the JSON output from the build script
        build_result = build_command.stdout.apply(lambda stdout: json.loads(stdout))

        # Extract properties as separate Outputs
        image_path = build_result.apply(lambda r: r["image_path"])
        image_name = build_result.apply(lambda r: r["image_name"])
        file_size = build_result.apply(lambda r: r["file_size"])
        file_hash = build_result.apply(lambda r: r["file_hash"])

        # Create the bucket object using image_path
        self.nixos_image = storage.BucketObject(
            "nixos-image",
            bucket=self.machine_images_bucket.name,
            name=image_name.apply(lambda n: f"nixos-images/{n}"),
            source=image_path.apply(lambda p: pulumi.FileAsset(p)),
            opts=ResourceOptions(
                depends_on=[build_command, self.machine_images_bucket] + (dependencies or []),
                additional_secret_outputs=["md5hash"],
                protect=False
            )
        )

        # Store image_info as a dictionary of Outputs
        self.image_info = {
            "image_path": image_path,
            "image_name": image_name,
            "image_size": file_size,
            "image_hash": file_hash,
            "gcs_url": self.nixos_image.self_link,  # self_link is also an Output
            "hostname": self.hostname  # Include hostname in image_info
        }
        # pulumi.export("image_info", self.image_info)

        return self.nixos_image

    def get_machine_images_bucket(self) -> storage.Bucket:
        return self.machine_images_bucket

    def get_app_bucket(self) -> storage.Bucket:
        return self.app_bucket
