
from pathlib import Path
import subprocess
import sys
import os
from typing import Dict, List
from InquirerPy import prompt
from InquirerPy.base.control import Choice
import platform
import psutil

class FirestreamSetup:
    def __init__(self):
        self.workspace = Path("/workspace")
        self.valid_modes = ["development", "test", "clean", "cicd", "production", "resume", "build"]
        self.config: Dict = {}
        self.required_scripts = [
            "src/deployment/host/k3d/bootstrap.sh",
        ]

    def check_required_files(self) -> bool:
        """Check if all required script files exist."""
        missing_files = []
        for script in self.required_scripts:
            if not (self.workspace / script).exists():
                missing_files.append(script)

        if missing_files:
            print("\nError: Required files are missing:")
            for file in missing_files:
                print(f"  - {self.workspace / file}")
            return False
        return True

    def _check_kvm(self) -> bool:
        """Check if KVM is available and properly configured."""
        try:
            # Check if KVM module is loaded
            kvm_loaded = subprocess.run(
                ['lsmod | grep kvm'],
                shell=True,
                capture_output=True
            ).returncode == 0

            # Check if user has access to KVM device
            kvm_access = os.access('/dev/kvm', os.R_OK | os.W_OK)

            if not kvm_loaded:
                print("Warning: KVM module is not loaded")
            if not kvm_access:
                print("Warning: Current user doesn't have access to KVM device")

            return kvm_loaded and kvm_access
        except Exception as e:
            print(f"Error checking KVM: {e}")
            return False

    def get_system_info(self) -> Dict:
        """Gather system information."""
        has_kvm = self._check_kvm()
        return {
            "cpu_architecture": platform.machine(),
            "total_cpu": psutil.cpu_count(),
            "total_memory": psutil.virtual_memory().total // (1024 * 1024),  # Convert to MB
            "has_nvidia_gpu": self._check_nvidia_gpu(),
            "has_kvm": has_kvm
        }

    def _check_nvidia_gpu(self) -> bool:
        """Check for NVIDIA GPU presence."""
        try:
            result = subprocess.run(['lspci'], capture_output=True, text=True)
            return 'nvidia' in result.stdout.lower()
        except:
            return False

    def _check_docker(self) -> bool:
        """Verify Docker installation and permissions."""
        try:
            subprocess.run(['docker', '--version'], check=True, capture_output=True)
            socket_path = Path('/var/run/docker.sock')
            return socket_path.exists() and os.access(socket_path, os.R_OK | os.W_OK)
        except:
            return False

    def get_infrastructure_questions(self) -> List[Dict]:
        """Generate infrastructure-related questions."""
        return [
            {
                "type": "list",
                "message": "Select infrastructure provider:",
                "name": "infra_provider",
                "choices": [
                    Choice("local", "Local K3D (Default)"),
                    Choice("gcp", "Google Cloud Platform"),
                    Choice("aws", "AWS (Coming Soon)")
                ],
                "default": "local"
            },
            {
                "type": "list",
                "message": "Select ingress type:",
                "name": "ingress_type",
                "choices": lambda answers: (
                    [Choice("local", "Local Ingress"), Choice("cloudflare", "Cloudflare")]
                    if answers["infra_provider"] == "local"
                    else [Choice("cloudflare", "Cloudflare")]  # GCP/AWS must use Cloudflare
                ),
                "default": "local",
                "when": lambda answers: answers["infra_provider"] != "aws"  # AWS not implemented yet
            },
            {
                "type": "confirm",
                "message": "Enable HTTPS? (Self-signed certificates will be used)",
                "name": "enable_https",
                "default": True,
                "when": lambda answers: (
                    answers["infra_provider"] == "local"
                    and answers["ingress_type"] == "local"
                )  # Only ask for HTTPS if using local K3D with local ingress
            },
        ]

    def get_cloudflare_questions(self) -> List[Dict]:
        """Generate Cloudflare-specific questions."""
        return [
            {
                "type": "input",
                "message": "Cloudflare Account ID:",
                "name": "cloudflare_account_id",
                "validate": lambda x: len(x) > 0
            },
            {
                "type": "input",
                "message": "Cloudflare Zone ID:",
                "name": "cloudflare_zone_id",
                "validate": lambda x: len(x) > 0
            },
            {
                "type": "input",
                "message": "Cloudflare API Token:",
                "name": "cloudflare_api_token",
                "validate": lambda x: len(x) > 0
            }
        ]

    def get_tailscale_questions(self) -> List[Dict]:
        """Generate Tailscale-specific questions."""
        return [
            {
                "type": "input",
                "message": "Tailscale API Key:",
                "name": "tailscale_api_key",
                "validate": lambda x: len(x) > 0
            },
            {
                "type": "input",
                "message": "Tailscale Tailnet Name:",
                "name": "tailscale_tailnet_name",
                "validate": lambda x: len(x) > 0
            }
        ]

    def get_gcp_questions(self) -> List[Dict]:
        """Generate GCP-specific configuration questions."""
        return [
            {
                "type": "input",
                "message": "GCP Project ID:",
                "name": "gcp_project",
                "validate": lambda x: len(x) > 0
            },
            {
                "type": "input",
                "message": "GCP Region:",
                "name": "gcp_region",
                "default": "us-central1"
            },
            {
                "type": "input",
                "message": "GCP Zone:",
                "name": "gcp_region_zone",
                "default": "us-central1-a"
            },
            {
                "type": "input",
                "message": "GCE Machine Type:",
                "name": "gce_machine_type",
                "default": "e2-standard-2"
            },
            {
                "type": "input",
                "message": "Deployment Name:",
                "name": "deployment_name",
                "default": "nodeone"
            },
            {
                "type": "input",
                "message": "Customer Name:",
                "name": "customer_name",
                "validate": lambda x: len(x) > 0
            },
            {
                "type": "input",
                "message": "Customer Email Domain:",
                "name": "customer_email_domain",
                "validate": lambda x: len(x) > 0
            },
            {
                "type": "input",
                "message": "Environment (prod/dev):",
                "name": "environment",
                "default": "prod"
            },
            {
                "type": "number",
                "message": "Rotation Days:",
                "name": "rotation_days",
                "default": 30
            },
            {
                "type": "input",
                "message": "Machine Prefix:",
                "name": "machine_prefix",
                "default": "prodbox"
            },
            {
                "type": "input",
                "message": "Region Code:",
                "name": "region",
                "default": "us1"
            }
        ]

    def set_pulumi_config(self, config: Dict):
        """Set Pulumi configuration values."""
        for key, value in config.items():
            self.run_command(f"pulumi config set deploy_standard:{key} {value}")

    def deploy(self):
        """Main deployment orchestration."""
        # Display banner and system info
        self.display_banner()
        system_info = self.get_system_info()
        print("\nSystem Information:")
        print(f"CPU Architecture: {system_info['cpu_architecture']}")
        print(f"Total CPU cores: {system_info['total_cpu']}")
        print(f"Total Memory (MB): {system_info['total_memory']}")
        print(f"NVIDIA GPU: {'Detected' if system_info['has_nvidia_gpu'] else 'Not detected'}")
        print(f"KVM Status: {'Available' if system_info['has_kvm'] else 'Not available (required for local NixOS VM builds)'}")

        if not system_info['has_kvm']:
            proceed = prompt([{
                "type": "confirm",
                "message": "KVM is required for local NixOS VM builds. Continue anyway?",
                "name": "continue",
                "default": False
            }])
            if not proceed["continue"]:
                return False

        # Check Docker
        if not self._check_docker():
            print("Error: Docker is required for setup")
            return False

        # Check required files
        if not self.check_required_files():
            return False

        # Get infrastructure and ingress configuration
        infra_config = prompt(self.get_infrastructure_questions())
        self.config.update(infra_config)

        # Configure Cloudflare if needed
        if self.config.get("ingress_type") == "cloudflare" or self.config["infra_provider"] in ["gcp", "aws"]:
            print("\nCloudflare configuration required for this setup:")
            cloudflare_config = prompt(self.get_cloudflare_questions())
            self.config.update(cloudflare_config)

        # Get provider-specific configuration
        if self.config["infra_provider"] == "gcp":
            gcp_config = prompt(self.get_gcp_questions())
            self.config.update(gcp_config)
            self.set_pulumi_config(gcp_config)

        # Configure Tailscale (optional, asked at the end)
        tailscale_config = prompt([{
            "type": "confirm",
            "message": "Would you like to enable Tailscale access?",
            "name": "enable_tailscale",
            "default": False
        }])

        if tailscale_config["enable_tailscale"]:
            self.config.update(prompt(self.get_tailscale_questions()))

        # Set up K3D
        if not self.run_command(f"bash {self.workspace}/src/deployment/host/k3d/bootstrap.sh",
                              "K3D cluster setup failed"):
            return False

        # Save configuration
        config_path = self.workspace / "config" / "setup_config.json"
        config_path.parent.mkdir(exist_ok=True)
        with open(config_path, 'w') as f:
            json.dump(self.config, f, indent=2)

        print("✨ Configuration saved successfully!")
        return True

    def run_command(self, command: str, error_msg: str = "Command failed") -> bool:
        """Execute a shell command and handle errors."""
        try:
            subprocess.run(command, shell=True, check=True)
            return True
        except subprocess.CalledProcessError:
            print(f"Error: {error_msg}")
            return False

    def display_banner(self):
        """Display the Firestream ASCII art banner."""
        banner = """
   ███████ ██ ██████  ███████ ███████ ████████ ██████  ███████  █████  ███    ███
   ██      ██ ██   ██ ██      ██         ██    ██   ██ ██      ██   ██ ████  ████
   █████   ██ ██████  █████   ███████    ██    ██████  █████   ███████ ██ ████ ██
   ██      ██ ██   ██ ██           ██    ██    ██   ██ ██      ██   ██ ██  ██  ██
   ██      ██ ██   ██ ███████ ███████    ██    ██   ██ ███████ ██   ██ ██      ██
        """
        print(banner)
        print(f"\nStarting Firestream Setup...")

def main():
    setup = FirestreamSetup()
    if not setup.deploy():
        sys.exit(1)
    print("✨ Firestream setup completed successfully!")

if __name__ == "__main__":
    main()
