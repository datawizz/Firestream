

import pulumi
from pulumi_tailscale import Provider, TailnetKey
from datetime import datetime, timedelta
import json
from typing import Optional, Dict, Any

import pulumi
from pulumi_tailscale import Provider, TailnetKey
from datetime import datetime, timedelta
import json
import requests
from typing import Optional, Dict, Any
class TailscaleManager:
    """
    Manages Tailscale resources including auth key creation, rotation, and device cleanup.

    Attributes:
        provider: Tailscale provider instance
        customer_name: Name of the customer
        rotation_days: Number of days before key rotation
        overlap_days: Number of days to keep old key active during rotation
        api_base_url: Base URL for Tailscale API
    """

    API_BASE_URL = "https://api.tailscale.com/api/v2"

    MAX_KEY_LIFETIME = 90  # Maximum key lifetime in days

    def __init__(self,
                 api_key: pulumi.Output[str],
                 customer_name: str,
                 rotation_days: int = 60,
                 overlap_days: int = 7,
                 tailnet: Optional[str] = None):
        """
        Initialize TailscaleManager.

        Args:
            api_key: Scoped Tailscale API key (account level)
            customer_name: Name of the customer
            rotation_days: Days before key rotation (default: 60)
            overlap_days: Days to keep old key during rotation (default: 7)
        """
        if rotation_days > self.MAX_KEY_LIFETIME:
            raise ValueError(f"rotation_days cannot exceed {self.MAX_KEY_LIFETIME}")

        self.provider = Provider("tailscale-provider",
                               api_key=api_key)
        self.customer_name = customer_name
        self.rotation_days = rotation_days
        self.overlap_days = overlap_days
        self.tailnet = tailnet or f"{customer_name}.github"
        self.device_name = self._generate_device_name()
        self.api_key = api_key

    def _get_headers(self, api_key: str) -> Dict[str, str]:
        """
        Generate headers for Tailscale API requests.
        """
        return {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json",
        }

    def _cleanup_old_devices(self, api_key: str) -> None:
        """
        Clean up old devices with the same name pattern using Tailscale API.
        """
        try:
            # List all devices
            response = requests.get(
                f"{self.API_BASE_URL}/tailnet/{self.tailnet}/devices",
                headers=self._get_headers(api_key),
                timeout=30
            )
            response.raise_for_status()
            devices = response.json().get("devices", [])

            # Find and delete devices matching our name pattern
            for device in devices:
                if device.get("hostname") == self.device_name and not device.get("authorized", False):
                    # Only delete unauthorized/expired devices
                    device_id = device.get("id")
                    if device_id:
                        delete_response = requests.delete(
                            f"{self.API_BASE_URL}/device/{device_id}",
                            headers=self._get_headers(api_key),
                            timeout=30
                        )
                        delete_response.raise_for_status()
                        pulumi.log.info(f"Cleaned up old device: {self.device_name} (ID: {device_id})")

        except requests.exceptions.RequestException as e:
            pulumi.log.warn(f"Warning: Error cleaning up old devices: {str(e)}")
            # Don't raise the error - we want to continue with key creation even if cleanup fails

    def _get_previous_key_state(self) -> Optional[Dict[str, Any]]:
        """
        Retrieve the previous key state from stack outputs.

        Returns:
            Dictionary containing previous key state or None if not found
        """
        try:
            previous_state = pulumi.get_stack().outputs.get("key_state")
            if previous_state:
                return json.loads(previous_state)
            return None
        except Exception:
            return None

    def _is_key_valid(self, key_state: Dict[str, Any]) -> bool:
        """
        Check if a key is still valid based on its expiry.

        Args:
            key_state: Dictionary containing key state information

        Returns:
            bool: True if key is still valid, False otherwise
        """
        try:
            expiry = datetime.fromtimestamp(int(key_state["expiry"]))
            return expiry > datetime.now()
        except (KeyError, ValueError, TypeError):
            return False

    def _generate_device_name(self) -> str:
        """
        Generate a consistent device name without relying on auto-incrementing.
        Format: {prefix}-{customer}-{environment}-{region}
        Example: prodbox-acme-prod-us1
        """
        config = pulumi.Config()
        environment = config.get("environment") or "prod"
        region = config.get("region") or "us1"
        prefix = config.get("machine_prefix") or "prodbox"

        return f"{prefix}-{self.customer_name}-{environment}-{region}"

    def create_auth_key(self) -> Dict[str, pulumi.Output]:
        """
        Create a new auth key and handle rotation if needed.

        Returns:
            Dictionary containing new key outputs and exports
        """
        # Calculate expiration time in seconds
        expiration = self.rotation_days * 24 * 60 * 60

        # Clean up any old devices first
        api_key_str = self.api_key.apply(lambda k: self._cleanup_old_devices(k))

        # Create new key
        new_key = TailnetKey("tailscale-auth-key",
            opts=pulumi.ResourceOptions(provider=self.provider),
            ephemeral=False,
            expiry=expiration,
            preauthorized=True,
            reusable=True,
            tags=[f"tag:{self.customer_name}"]
            # hostname=self._generate_device_name()
        )

        # Handle key rotation if there's a valid previous key
        previous_state = self._get_previous_key_state()
        if previous_state and self._is_key_valid(previous_state):
            # Calculate remaining validity for old key
            old_expiry = min(
                int(previous_state["expiry"]),
                int((datetime.now() + timedelta(days=self.overlap_days)).timestamp())
            )

            # Keep old key active during overlap period
            old_key = TailnetKey("tailscale-auth-key-old",
                opts=pulumi.ResourceOptions(provider=self.provider),
                key_id=previous_state["key_id"],
                ephemeral=False,
                expiry=old_expiry,
                preauthorized=True,
                reusable=True,
                tags=[f"tag:{self.customer_name}"]
            )

        # Prepare key state for export
        key_state = pulumi.Output.all(
            key=new_key.key,
            key_id=new_key.id,
            expiry=new_key.expiry,
            tags=new_key.tags
        ).apply(
            lambda args: json.dumps({
                "key": args["key"],
                "key_id": args["key_id"],
                "expiry": args["expiry"],
                "tags": args["tags"]
            })
        )

        # Calculate human-readable dates
        human_readable_expiry = new_key.expiry.apply(
            lambda seconds: (
                datetime.now() + timedelta(seconds=int(seconds))
            ).strftime('%Y-%m-%d')
        )

        next_rotation = (datetime.now() + timedelta(days=self.rotation_days)).strftime('%Y-%m-%d')

        # Export values
        exports = {
            "key_state": key_state,
            "current_key": new_key.key,
            "expiry_date": human_readable_expiry,
            "next_rotation": next_rotation
        }

        return exports

def create_tailscale_manager() -> TailscaleManager:
    """
    Factory function to create a TailscaleManager instance from Pulumi config.

    Returns:
        TailscaleManager instance
    """
    config = pulumi.Config()
    api_key = config.require_secret("tailscale_api_key")
    customer_name = config.require("customer_name")
    rotation_days = config.get_int("rotation_days") or 60
    overlap_days = config.get_int("overlap_days") or 7

    return TailscaleManager(
        api_key=api_key,
        customer_name=customer_name,
        rotation_days=rotation_days,
        overlap_days=overlap_days
    )

# Example usage
if __name__ == "__main__":
    manager = create_tailscale_manager()
    manager.create_auth_key()
