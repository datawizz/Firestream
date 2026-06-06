

import pulumi
import pulumi_gcp as gcp
from typing import List, Optional
from dataclasses import dataclass

@dataclass
class BucketConfig:
    """Configuration for Google Cloud Storage"""
    bucket_name: str
    location: str
    force_destroy: bool

class BucketManager:

    def __init__(self):
        pass


    def create_bucket(self, bucket_name: str, location: str, force_destroy: bool):
        """Create a GCP storage bucket"""
        return gcp.storage.Bucket(bucket_name,
            location=location,
            force_destroy=force_destroy
        )
