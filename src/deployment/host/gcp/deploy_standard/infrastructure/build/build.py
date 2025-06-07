# # # # import os
# # # # import subprocess
# # # # import logging
# # # # import shutil
# # # # from pathlib import Path
# # # # from typing import Optional, Dict
# # # # from dataclasses import dataclass
# # # # import hashlib
# # # # import time
# # # # from enum import Enum
# # # # import json

# # # # # enum for different build types gcp or aws
# # # # class BuildType(Enum):
# # # #     GCP = "gcp" # results in a raw.tar.gz image
# # # #     AWS = "aws" # TODO: implement this
# # # #     QEMU = "qemu" # results in a qcow2 image compatible with KVM, UTM, Proxmox, etc.

# # # # @dataclass
# # # # class BuildConfig:
# # # #     """Contains information about the built image"""
# # # #     image_path: Path
# # # #     image_name: str
# # # #     file_size: int
# # # #     file_hash: str
# # # #     success: bool
# # # #     hostname: str
# # # #     build_type: BuildType


# # # # class NixBuildError(Exception):
# # # #     """Custom exception for Nix build failures."""
# # # #     pass

# # # # class NixImageBuilder:
# # # #     """Handles building Nix images for Google Compute Engine with Pulumi integration."""

# # # #     def __init__(self,
# # # #                  workspace_dir: str = "/workspace",
# # # #                  build_jobs: int = 10,
# # # #                  command_timeout: int = 600,
# # # #                  extra_args: Optional[Dict[str, str]] = None):
# # # #         self.workspace_dir = Path(workspace_dir)
# # # #         self.build_dir = self.workspace_dir / "_build"
# # # #         self.tmp_dir = self.build_dir / "tmp"
# # # #         self.out_dir = self.build_dir / "out"
# # # #         self.build_jobs = build_jobs
# # # #         self.command_timeout = command_timeout
# # # #         self.extra_args = extra_args or {}

# # # #         # Configure logging with detailed format for debugging
# # # #         logging.basicConfig(
# # # #             level=logging.DEBUG,
# # # #             format='%(asctime)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s'
# # # #         )
# # # #         self.logger = logging.getLogger(__name__)

# # # #     def _run_command(self, cmd: list[str], shell: bool = False, timeout: Optional[int] = None) -> subprocess.CompletedProcess:
# # # #         """Execute a command with proper error handling, logging, and timeout."""
# # # #         cmd_str = ' '.join(cmd) if isinstance(cmd, list) else cmd
# # # #         self.logger.debug(f"Running command: {cmd_str}")

# # # #         timeout = timeout or self.command_timeout

# # # #         try:
# # # #             start_time = time.time()
# # # #             result = subprocess.run(
# # # #                 cmd,
# # # #                 check=True,
# # # #                 capture_output=True,
# # # #                 text=True,
# # # #                 shell=shell,
# # # #                 timeout=timeout
# # # #             )
# # # #             end_time = time.time()

# # # #             self.logger.debug(f"Command completed in {end_time - start_time:.2f} seconds")
# # # #             if result.stdout:
# # # #                 self.logger.debug(f"Command output: {result.stdout.strip()}")
# # # #             return result

# # # #         except subprocess.TimeoutExpired as e:
# # # #             self.logger.error(f"Command timed out after {timeout} seconds: {cmd_str}")
# # # #             raise NixBuildError(f"Command timed out: {cmd_str}") from e
# # # #         except subprocess.CalledProcessError as e:
# # # #             self.logger.error(f"Command failed with exit code {e.returncode}: {cmd_str}")
# # # #             self.logger.error(f"Standard error: {e.stderr}")
# # # #             raise NixBuildError(f"Command failed: {e.stderr}") from e

# # # #     def _setup_directories(self) -> None:
# # # #         """Setup required directories with proper permissions."""
# # # #         self.logger.info("Setting up build directories...")
# # # #         for directory in [self.out_dir, self.tmp_dir]:
# # # #             try:
# # # #                 directory.mkdir(parents=True, exist_ok=True)
# # # #                 self._run_command(["sudo", "chown", "-R", f"{os.getuid()}:{os.getgid()}", str(directory)])
# # # #             except Exception as e:
# # # #                 self.logger.error(f"Failed to setup directory {directory}: {e}")
# # # #                 raise NixBuildError(f"Directory setup failed: {str(e)}") from e

# # # #     def _cleanup_tmp(self) -> None:
# # # #         """Clean up temporary build directory."""
# # # #         self.logger.info("Cleaning up temporary directory...")
# # # #         try:
# # # #             if self.tmp_dir.exists():
# # # #                 shutil.rmtree(self.tmp_dir)
# # # #             self.tmp_dir.mkdir(parents=True, exist_ok=True)
# # # #         except Exception as e:
# # # #             self.logger.error(f"Failed to cleanup temporary directory: {e}")
# # # #             raise NixBuildError(f"Cleanup failed: {str(e)}") from e

# # # #     def _find_nix_store_image(self, nix_path: str) -> Path:
# # # #         """Find the image file in the Nix store directory."""
# # # #         try:
# # # #             # Read the symlink to get the actual store path
# # # #             store_path = Path(os.readlink(nix_path))
# # # #             self.logger.debug(f"Resolved store path: {store_path}")

# # # #             # Look for the image file in the store path
# # # #             image_files = list(store_path.glob("*.raw.tar.gz"))
# # # #             if not image_files:
# # # #                 raise NixBuildError(f"No image files found in Nix store path: {store_path}")

# # # #             self.logger.info(f"Found image file: {image_files[0]}")
# # # #             return image_files[0]
# # # #         except OSError as e:
# # # #             self.logger.error(f"Failed to read store path: {e}")
# # # #             raise NixBuildError(f"Failed to access Nix store path: {e}") from e

# # # #     def _calculate_file_hash(self, file_path: Path) -> str:
# # # #         """Calculate SHA256 hash of a file."""
# # # #         sha256_hash = hashlib.sha256()
# # # #         with open(file_path, "rb") as f:
# # # #             for byte_block in iter(lambda: f.read(4096), b""):
# # # #                 sha256_hash.update(byte_block)
# # # #         return sha256_hash.hexdigest()

# # # #     def _copy_file(self, source: Path, dest: Path) -> None:
# # # #         """Copy file with progress logging and verification."""
# # # #         try:
# # # #             self.logger.info(f"Starting file copy from {source} to {dest}")

# # # #             if not source.exists():
# # # #                 raise NixBuildError(f"Source file not found: {source}")

# # # #             # Get source file details
# # # #             source_size = source.stat().st_size
# # # #             source_hash = self._calculate_file_hash(source)
# # # #             self.logger.info(f"Source file size: {source_size/1024/1024:.2f} MB")

# # # #             # Ensure destination directory exists
# # # #             dest.parent.mkdir(parents=True, exist_ok=True)

# # # #             # Copy the file
# # # #             shutil.copy2(source, dest)

# # # #             # Fix permissions on the destination file
# # # #             os.chmod(dest, 0o644)

# # # #             # Verify the copy
# # # #             if not dest.exists():
# # # #                 raise NixBuildError(f"Destination file not found after copy: {dest}")

# # # #             dest_size = dest.stat().st_size
# # # #             dest_hash = self._calculate_file_hash(dest)

# # # #             if dest_size != source_size:
# # # #                 raise NixBuildError(
# # # #                     f"File size mismatch after copy: source={source_size}, dest={dest_size}"
# # # #                 )

# # # #             if dest_hash != source_hash:
# # # #                 raise NixBuildError(
# # # #                     f"File hash mismatch after copy: source={source_hash}, dest={dest_hash}"
# # # #                 )

# # # #             self.logger.info("File copy completed and verified successfully")

# # # #         except Exception as e:
# # # #             self.logger.error(f"File copy failed: {str(e)}")
# # # #             raise NixBuildError(f"Failed to copy file: {str(e)}") from e


# # # #     def build_image(self, nix_config_path: str) -> BuildConfig:
# # # #         """
# # # #         Build the Nix image for GCE using the new argument set format.
# # # #         """
# # # #         target_file = self.out_dir / "nixos-image-x86_64-linux.raw.tar.gz"

# # # #         try:
# # # #             self._setup_directories()
# # # #             self._cleanup_tmp()

# # # #             if target_file.exists():
# # # #                 self.logger.info("Removing existing target file...")
# # # #                 target_file.unlink()

# # # #             # NOTE build the nixos-image "nixos-image-x86_64-linux.raw.tar.gz"

# # # #             # Image build command
# # # #             build_cmd = [
# # # #                 "nix-build",
# # # #                 "default.nix",
# # # #                 "-A", "gcpImage",
# # # #                 "--arg", "config", json.dumps({
# # # #                     "google_application_credentials": self.extra_args.get("google_application_credentials", ""),
# # # #                     "hostname": self.extra_args.get("hostname", ""),
# # # #                     "build_type": self.build_type
# # # #                 }).replace('"', '\\"'),  # Escape quotes for Nix
# # # #                 "-o", str(self.tmp_dir / "result"),
# # # #                 "-j", str(self.build_jobs),
# # # #                 "--show-trace"
# # # #             ]


# # # #             self.logger.info(f"Building Nix image with command: {' '.join(build_cmd)}")
# # # #             result = self._run_command(build_cmd)

# # # #             # Rest of the method remains the same
# # # #             nix_store_path = str(self.tmp_dir / "result")
# # # #             self.logger.info(f"Build completed, store path: {nix_store_path}")

# # # #             source_image = self._find_nix_store_image(nix_store_path)
# # # #             self.logger.info(f"Found built image at: {source_image}")

# # # #             self._copy_file(source_image, target_file)

# # # #             file_hash = self._calculate_file_hash(target_file)
# # # #             file_size = target_file.stat().st_size

# # # #             return BuildConfig(
# # # #                 image_path=target_file,
# # # #                 image_name=target_file.name,
# # # #                 file_size=file_size,
# # # #                 file_hash=file_hash,
# # # #                 hostname=self.extra_args.get('hostname', ''),
# # # #                 success=True
# # # #             )

# # # #         except Exception as e:
# # # #             self.logger.error(f"Failed to build Nix image: {str(e)}")
# # # #             raise NixBuildError(f"Image build failed: {str(e)}") from e


# # # import os
# # # import subprocess
# # # import logging
# # # import shutil
# # # from pathlib import Path
# # # from typing import Optional, Dict, Any
# # # from dataclasses import dataclass, field
# # # import hashlib
# # # import time
# # # from enum import Enum
# # # import json

# # # class BuildType(Enum):
# # #     GCP = "gcp"
# # #     AWS = "aws"
# # #     QEMU = "qemu"

# # # @dataclass
# # # class BuildArguments:
# # #     """Contains the build arguments for the Nix build"""
# # #     hostname: str
# # #     build_type: BuildType
# # #     extra_args: Dict[str, Any] = field(default_factory=dict)

# # #     def to_nix_args(self) -> Dict[str, Any]:
# # #         """Convert the build arguments to a Nix-compatible format"""
# # #         args = {
# # #             "hostname": self.hostname,
# # #             "build_type": self.build_type.value,
# # #             **self.extra_args
# # #         }
# # #         return args

# # # @dataclass
# # # class BuildConfig:
# # #     """Contains information about the built image"""
# # #     image_path: Path
# # #     image_name: str
# # #     file_size: int
# # #     file_hash: str
# # #     success: bool
# # #     build_args: BuildArguments
# # #     build_time: float = 0.0

# # # class NixBuildError(Exception):
# # #     """Custom exception for Nix build failures."""
# # #     pass

# # # class NixImageBuilder:
# # #     """Handles building Nix images with flexible build arguments."""

# # #     def __init__(self,
# # #                  workspace_dir: str = "/workspace",
# # #                  build_jobs: int = 10,
# # #                  command_timeout: int = 600):
# # #         self.workspace_dir = Path(workspace_dir)
# # #         self.build_dir = self.workspace_dir / "_build"
# # #         self.tmp_dir = self.build_dir / "tmp"
# # #         self.out_dir = self.build_dir / "out"
# # #         self.build_jobs = build_jobs
# # #         self.command_timeout = command_timeout

# # #         # Configure logging
# # #         logging.basicConfig(
# # #             level=logging.DEBUG,
# # #             format='%(asctime)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s'
# # #         )
# # #         self.logger = logging.getLogger(__name__)

# # #     def _run_command(self, cmd: list[str], shell: bool = False, timeout: Optional[int] = None) -> subprocess.CompletedProcess:
# # #         """Execute a command with proper error handling and logging."""
# # #         cmd_str = ' '.join(cmd) if isinstance(cmd, list) else cmd
# # #         self.logger.debug(f"Running command: {cmd_str}")

# # #         timeout = timeout or self.command_timeout

# # #         try:
# # #             start_time = time.time()
# # #             result = subprocess.run(
# # #                 cmd,
# # #                 check=True,
# # #                 capture_output=True,
# # #                 text=True,
# # #                 shell=shell,
# # #                 timeout=timeout
# # #             )
# # #             end_time = time.time()

# # #             self.logger.debug(f"Command completed in {end_time - start_time:.2f} seconds")
# # #             if result.stdout:
# # #                 self.logger.debug(f"Command output: {result.stdout.strip()}")
# # #             return result

# # #         except subprocess.TimeoutExpired as e:
# # #             self.logger.error(f"Command timed out after {timeout} seconds: {cmd_str}")
# # #             raise NixBuildError(f"Command timed out: {cmd_str}") from e
# # #         except subprocess.CalledProcessError as e:
# # #             self.logger.error(f"Command failed with exit code {e.returncode}: {cmd_str}")
# # #             self.logger.error(f"Standard error: {e.stderr}")
# # #             raise NixBuildError(f"Command failed: {e.stderr}") from e

# # #     def _construct_build_command(self, nix_file: str, attr: str, build_args: BuildArguments) -> list[str]:
# # #         """Construct the nix-build command with the provided arguments."""
# # #         nix_args = build_args.to_nix_args()

# # #         # Escape special characters in the JSON string for nix
# # #         escaped_args = json.dumps(nix_args).replace('"', '\\"')

# # #         cmd = [
# # #             "nix-build",
# # #             nix_file,
# # #             "-A", attr,
# # #             "--arg", "config", escaped_args,
# # #             "-o", str(self.tmp_dir / "result"),
# # #             "-j", str(self.build_jobs),
# # #             "--show-trace"
# # #         ]

# # #         return cmd

# # #     def build_image(self, nix_file: str, attr: str, build_args: BuildArguments) -> BuildConfig:
# # #         """
# # #         Build a Nix image with the provided build arguments.

# # #         Args:
# # #             nix_file: Path to the Nix expression file
# # #             attr: The attribute to build from the Nix expression
# # #             build_args: BuildArguments instance containing the build configuration
# # #         """
# # #         target_file = self.out_dir / f"nixos-{build_args.build_type.value}-image.raw.tar.gz"

# # #         try:
# # #             self._setup_directories()
# # #             self._cleanup_tmp()

# # #             if target_file.exists():
# # #                 self.logger.info("Removing existing target file...")
# # #                 target_file.unlink()

# # #             start_time = time.time()

# # #             # Construct and run the build command
# # #             build_cmd = self._construct_build_command(nix_file, attr, build_args)
# # #             self.logger.info(f"Building Nix image with command: {' '.join(build_cmd)}")
# # #             self._run_command(build_cmd)

# # #             # Process the build result
# # #             nix_store_path = str(self.tmp_dir / "result")
# # #             self.logger.info(f"Build completed, store path: {nix_store_path}")

# # #             source_image = self._find_nix_store_image(nix_store_path)
# # #             self.logger.info(f"Found built image at: {source_image}")

# # #             self._copy_file(source_image, target_file)

# # #             build_time = time.time() - start_time
# # #             file_hash = self._calculate_file_hash(target_file)
# # #             file_size = target_file.stat().st_size

# # #             return BuildConfig(
# # #                 image_path=target_file,
# # #                 image_name=target_file.name,
# # #                 file_size=file_size,
# # #                 file_hash=file_hash,
# # #                 success=True,
# # #                 build_args=build_args,
# # #                 build_time=build_time
# # #             )

# # #         except Exception as e:
# # #             self.logger.error(f"Failed to build Nix image: {str(e)}")
# # #             raise NixBuildError(f"Image build failed: {str(e)}") from e

# # #     # ... (rest of the helper methods remain the same)



# # import os
# # import subprocess
# # import logging
# # import shutil
# # from pathlib import Path
# # from typing import Optional, Dict, Any, List
# # from dataclasses import dataclass, field
# # import hashlib
# # import time
# # from enum import Enum
# # import json

# # class BuildType(Enum):
# #     """Supported build types for NixOS images"""
# #     GCP = "gcp"    # Google Cloud Platform image
# #     AWS = "aws"    # Amazon Web Services image (placeholder)
# #     QEMU = "qemu"  # QEMU/KVM compatible image

# # @dataclass
# # class BuildArguments:
# #     """Contains all arguments needed for the Nix build process"""
# #     hostname: str
# #     build_type: BuildType
# #     google_credentials: Optional[str] = None
# #     extra_modules: List[str] = field(default_factory=list)
# #     extra_args: Dict[str, Any] = field(default_factory=dict)

# #     def to_nix_args(self) -> Dict[str, Any]:
# #         """Convert build arguments to Nix-compatible format"""
# #         args = {
# #             "hostname": self.hostname,
# #             "build_type": self.build_type.value,
# #         }

# #         if self.google_credentials:
# #             args["google_application_credentials"] = self.google_credentials

# #         if self.extra_args:
# #             args.update(self.extra_args)

# #         return args

# # @dataclass
# # class BuildConfig:
# #     """Results and metadata from a successful build"""
# #     image_path: Path
# #     image_name: str
# #     file_size: int
# #     file_hash: str
# #     success: bool
# #     build_args: BuildArguments
# #     build_time: float = 0.0

# # class NixBuildError(Exception):
# #     """Custom exception for Nix build failures"""
# #     pass

# # class NixImageBuilder:
# #     """Builds NixOS images using nix-build with proper argument handling"""

# #     def __init__(
# #         self,
# #         workspace_dir: str = "/workspace",
# #         build_jobs: int = 8,
# #         command_timeout: int = 1800  # 30 minutes default timeout
# #     ):
# #         self.workspace_dir = Path(workspace_dir)
# #         self.build_dir = self.workspace_dir / "_build"
# #         self.tmp_dir = self.build_dir / "tmp"
# #         self.out_dir = self.build_dir / "out"
# #         self.build_jobs = build_jobs
# #         self.command_timeout = command_timeout

# #         # Configure logging
# #         logging.basicConfig(
# #             level=logging.INFO,
# #             format='%(asctime)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s'
# #         )
# #         self.logger = logging.getLogger(__name__)

# #     def _setup_directories(self) -> None:
# #         """Create and set permissions for build directories"""
# #         for directory in [self.build_dir, self.tmp_dir, self.out_dir]:
# #             directory.mkdir(parents=True, exist_ok=True)
# #             # Ensure proper permissions
# #             os.chmod(directory, 0o755)

# #     def _cleanup_tmp(self) -> None:
# #         """Clean temporary build artifacts"""
# #         if self.tmp_dir.exists():
# #             shutil.rmtree(self.tmp_dir)
# #         self.tmp_dir.mkdir(parents=True)

# #     def _run_command(
# #         self,
# #         cmd: list[str],
# #         shell: bool = False,
# #         timeout: Optional[int] = None
# #     ) -> subprocess.CompletedProcess:
# #         """Execute a command with proper error handling and logging"""
# #         cmd_str = ' '.join(cmd) if isinstance(cmd, list) else cmd
# #         self.logger.debug(f"Running command: {cmd_str}")

# #         timeout = timeout or self.command_timeout

# #         try:
# #             start_time = time.time()
# #             result = subprocess.run(
# #                 cmd,
# #                 check=True,
# #                 capture_output=True,
# #                 text=True,
# #                 shell=shell,
# #                 timeout=timeout
# #             )
# #             duration = time.time() - start_time

# #             self.logger.debug(f"Command completed in {duration:.2f} seconds")
# #             if result.stdout:
# #                 self.logger.debug(f"Command output: {result.stdout.strip()}")
# #             return result

# #         except subprocess.TimeoutExpired as e:
# #             self.logger.error(f"Command timed out after {timeout} seconds: {cmd_str}")
# #             raise NixBuildError(f"Command timed out: {cmd_str}") from e
# #         except subprocess.CalledProcessError as e:
# #             self.logger.error(f"Command failed with exit code {e.returncode}: {cmd_str}")
# #             self.logger.error(f"Standard error: {e.stderr}")
# #             raise NixBuildError(f"Command failed: {e.stderr}") from e

# #     def _get_build_attr(self, build_type: BuildType) -> str:
# #         """Get the appropriate Nix attribute based on build type"""
# #         attr_map = {
# #             BuildType.GCP: "gcpImage",
# #             BuildType.QEMU: "qemuImage",
# #             BuildType.AWS: "awsImage"  # Placeholder for future implementation
# #         }
# #         return attr_map.get(build_type, "gcpImage")

# #     def _construct_build_command(
# #         self,
# #         nix_file: str,
# #         build_args: BuildArguments
# #     ) -> list[str]:
# #         """Construct the nix-build command with proper arguments"""
# #         attr = self._get_build_attr(build_args.build_type)
# #         nix_args = build_args.to_nix_args()

# #         # Escape special characters in JSON for Nix
# #         escaped_args = json.dumps(nix_args).replace('"', '\\"')

# #         cmd = [
# #             "nix-build",
# #             nix_file,
# #             "-A", attr,
# #             "--arg", "config", escaped_args,
# #             "-o", str(self.tmp_dir / "result"),
# #             "-j", str(self.build_jobs),
# #             "--show-trace"
# #         ]

# #         return cmd

# #     def _find_build_output(self, build_type: BuildType, result_path: str) -> Path:
# #         """Find the built image file based on build type"""
# #         store_path = Path(os.readlink(result_path))

# #         patterns = {
# #             BuildType.GCP: "*.raw.tar.gz",
# #             BuildType.QEMU: "*.qcow2",
# #             BuildType.AWS: "*.vhd"  # Placeholder
# #         }

# #         pattern = patterns.get(build_type, "*.raw.tar.gz")
# #         matches = list(store_path.glob(pattern))

# #         if not matches:
# #             raise NixBuildError(f"No image file matching {pattern} found in {store_path}")

# #         return matches[0]

# #     def _calculate_file_hash(self, file_path: Path) -> str:
# #         """Calculate SHA256 hash of a file"""
# #         sha256_hash = hashlib.sha256()
# #         with open(file_path, "rb") as f:
# #             for byte_block in iter(lambda: f.read(4096), b""):
# #                 sha256_hash.update(byte_block)
# #         return sha256_hash.hexdigest()

# #     def _copy_file(self, source: Path, dest: Path) -> None:
# #         """Copy file with verification"""
# #         if not source.exists():
# #             raise NixBuildError(f"Source file not found: {source}")

# #         source_size = source.stat().st_size
# #         source_hash = self._calculate_file_hash(source)

# #         dest.parent.mkdir(parents=True, exist_ok=True)
# #         shutil.copy2(source, dest)
# #         os.chmod(dest, 0o644)

# #         dest_size = dest.stat().st_size
# #         dest_hash = self._calculate_file_hash(dest)

# #         if dest_size != source_size or dest_hash != source_hash:
# #             raise NixBuildError("File verification failed after copy")

# #     def build_image(
# #         self,
# #         nix_file: str,
# #         build_args: BuildArguments
# #     ) -> BuildConfig:
# #         """
# #         Build a NixOS image using the provided configuration

# #         Args:
# #             nix_file: Path to the Nix expression file (default.nix)
# #             build_args: BuildArguments instance with build configuration

# #         Returns:
# #             BuildConfig object containing build results and metadata
# #         """
# #         target_name = f"nixos-{build_args.build_type.value}-{build_args.hostname}"
# #         extension = ".qcow2" if build_args.build_type == BuildType.QEMU else ".raw.tar.gz"
# #         target_file = self.out_dir / f"{target_name}{extension}"

# #         try:
# #             self._setup_directories()
# #             self._cleanup_tmp()

# #             if target_file.exists():
# #                 target_file.unlink()

# #             start_time = time.time()

# #             # Build the image
# #             build_cmd = self._construct_build_command(nix_file, build_args)
# #             self.logger.info(f"Building NixOS image: {target_name}")
# #             self._run_command(build_cmd)

# #             # Process build output
# #             result_path = str(self.tmp_dir / "result")
# #             source_image = self._find_build_output(build_args.build_type, result_path)
# #             self._copy_file(source_image, target_file)

# #             # Calculate metadata
# #             build_time = time.time() - start_time
# #             file_hash = self._calculate_file_hash(target_file)
# #             file_size = target_file.stat().st_size

# #             return BuildConfig(
# #                 image_path=target_file,
# #                 image_name=target_file.name,
# #                 file_size=file_size,
# #                 file_hash=file_hash,
# #                 success=True,
# #                 build_args=build_args,
# #                 build_time=build_time
# #             )

# #         except Exception as e:
# #             self.logger.error(f"Failed to build NixOS image: {str(e)}")
# #             raise NixBuildError(f"Image build failed: {str(e)}") from e

# import os
# import subprocess
# import logging
# import shutil
# from pathlib import Path
# from typing import Optional, Dict, Any, List
# from dataclasses import dataclass, field
# import hashlib
# import time
# from enum import Enum
# import json

# class BuildType(Enum):
#     """Supported build types for NixOS images"""
#     GCP = "gcp"    # Google Cloud Platform image
#     AWS = "aws"    # Amazon Web Services image (placeholder)
#     QEMU = "qemu"  # QEMU/KVM compatible image

# @dataclass
# class BuildArguments:
#     """Contains all arguments needed for the Nix build process"""
#     hostname: str
#     build_type: BuildType
#     google_credentials: Optional[str] = None
#     extra_modules: List[str] = field(default_factory=list)
#     extra_args: Dict[str, Any] = field(default_factory=dict)

#     def to_nix_args(self) -> Dict[str, Any]:
#         """Convert build arguments to Nix-compatible format"""
#         args = {
#             "hostname": self.hostname,
#             "build_type": self.build_type.value,
#         }

#         if self.google_credentials:
#             args["google_application_credentials"] = self.google_credentials

#         if self.extra_args:
#             args.update(self.extra_args)

#         return args

# @dataclass
# class BuildConfig:
#     """Results and metadata from a successful build"""
#     image_path: Path
#     image_name: str
#     file_size: int
#     file_hash: str
#     success: bool
#     build_args: BuildArguments
#     build_time: float = 0.0

# class NixBuildError(Exception):
#     """Custom exception for Nix build failures"""
#     pass

# class NixImageBuilder:
#     """Builds NixOS images using nix-build with proper argument handling"""

#     def __init__(
#         self,
#         workspace_dir: str = "/workspace",
#         build_jobs: int = 8,
#         command_timeout: int = 1800  # 30 minutes default timeout
#     ):
#         self.workspace_dir = Path(workspace_dir)
#         self.build_dir = self.workspace_dir / "_build"
#         self.tmp_dir = self.build_dir / "tmp"
#         self.out_dir = self.build_dir / "out"
#         self.build_jobs = build_jobs
#         self.command_timeout = command_timeout

#         # Configure logging
#         logging.basicConfig(
#             level=logging.INFO,
#             format='%(asctime)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s'
#         )
#         self.logger = logging.getLogger(__name__)

#     def _setup_directories(self) -> None:
#         """Create and set permissions for build directories"""
#         for directory in [self.build_dir, self.tmp_dir, self.out_dir]:
#             directory.mkdir(parents=True, exist_ok=True)
#             # Ensure proper permissions
#             os.chmod(directory, 0o755)

#     def _cleanup_tmp(self) -> None:
#         """Clean temporary build artifacts"""
#         if self.tmp_dir.exists():
#             shutil.rmtree(self.tmp_dir)
#         self.tmp_dir.mkdir(parents=True)

#     def _run_command(
#         self,
#         cmd: list[str],
#         shell: bool = False,
#         timeout: Optional[int] = None
#     ) -> subprocess.CompletedProcess:
#         """Execute a command with proper error handling and logging"""
#         cmd_str = ' '.join(cmd) if isinstance(cmd, list) else cmd
#         self.logger.debug(f"Running command: {cmd_str}")

#         timeout = timeout or self.command_timeout

#         try:
#             start_time = time.time()
#             result = subprocess.run(
#                 cmd,
#                 check=True,
#                 capture_output=True,
#                 text=True,
#                 shell=shell,
#                 timeout=timeout
#             )
#             duration = time.time() - start_time

#             self.logger.debug(f"Command completed in {duration:.2f} seconds")
#             if result.stdout:
#                 self.logger.debug(f"Command output: {result.stdout.strip()}")
#             return result

#         except subprocess.TimeoutExpired as e:
#             self.logger.error(f"Command timed out after {timeout} seconds: {cmd_str}")
#             raise NixBuildError(f"Command timed out: {cmd_str}") from e
#         except subprocess.CalledProcessError as e:
#             self.logger.error(f"Command failed with exit code {e.returncode}: {cmd_str}")
#             self.logger.error(f"Standard error: {e.stderr}")
#             raise NixBuildError(f"Command failed: {e.stderr}") from e

#     def _get_build_attr(self, build_type: BuildType) -> str:
#         """Get the appropriate Nix attribute based on build type"""
#         attr_map = {
#             BuildType.GCP: "gcpImage",
#             BuildType.QEMU: "qemuImage",
#             BuildType.AWS: "awsImage"  # Placeholder for future implementation
#         }
#         return attr_map.get(build_type, "gcpImage")

#     def _construct_build_command(
#         self,
#         nix_file: str,
#         build_args: BuildArguments
#     ) -> list[str]:
#         """Construct the nix-build command with proper arguments"""
#         attr = self._get_build_attr(build_args.build_type)
#         nix_args = build_args.to_nix_args()

#         # Convert to Nix expression format
#         nix_str = "{"
#         for key, value in nix_args.items():
#             if isinstance(value, str):
#                 nix_str += f'{key} = "{value}"; '
#             else:
#                 nix_str += f'{key} = {value}; '
#         nix_str = nix_str.rstrip("; ") + "}"

#         cmd = [
#             "nix-build",
#             nix_file,
#             "-A", attr,
#             "--arg", "config", nix_str,
#             "-o", str(self.tmp_dir / "result"),
#             "-j", str(self.build_jobs),
#             "--show-trace"
#         ]

#         return cmd

#     def _find_build_output(self, build_type: BuildType, result_path: str) -> Path:
#         """Find the built image file based on build type"""
#         store_path = Path(os.readlink(result_path))

#         patterns = {
#             BuildType.GCP: "*.raw.tar.gz",
#             BuildType.QEMU: "*.qcow2",
#             BuildType.AWS: "*.vhd"  # Placeholder
#         }

#         pattern = patterns.get(build_type, "*.raw.tar.gz")
#         matches = list(store_path.glob(pattern))

#         if not matches:
#             raise NixBuildError(f"No image file matching {pattern} found in {store_path}")

#         return matches[0]

#     def _calculate_file_hash(self, file_path: Path) -> str:
#         """Calculate SHA256 hash of a file"""
#         sha256_hash = hashlib.sha256()
#         with open(file_path, "rb") as f:
#             for byte_block in iter(lambda: f.read(4096), b""):
#                 sha256_hash.update(byte_block)
#         return sha256_hash.hexdigest()

#     def _copy_file(self, source: Path, dest: Path) -> None:
#         """Copy file with verification"""
#         if not source.exists():
#             raise NixBuildError(f"Source file not found: {source}")

#         source_size = source.stat().st_size
#         source_hash = self._calculate_file_hash(source)

#         dest.parent.mkdir(parents=True, exist_ok=True)
#         shutil.copy2(source, dest)
#         os.chmod(dest, 0o644)

#         dest_size = dest.stat().st_size
#         dest_hash = self._calculate_file_hash(dest)

#         if dest_size != source_size or dest_hash != source_hash:
#             raise NixBuildError("File verification failed after copy")

#     def build_image(
#         self,
#         nix_file: str,
#         build_args: BuildArguments
#     ) -> BuildConfig:
#         """
#         Build a NixOS image using the provided configuration

#         Args:
#             nix_file: Path to the Nix expression file (default.nix)
#             build_args: BuildArguments instance with build configuration

#         Returns:
#             BuildConfig object containing build results and metadata
#         """
#         target_name = f"nixos-{build_args.build_type.value}-{build_args.hostname}"
#         extension = ".qcow2" if build_args.build_type == BuildType.QEMU else ".raw.tar.gz"
#         target_file = self.out_dir / f"{target_name}{extension}"

#         try:
#             self._setup_directories()
#             self._cleanup_tmp()

#             if target_file.exists():
#                 target_file.unlink()

#             start_time = time.time()

#             # Build the image
#             build_cmd = self._construct_build_command(nix_file, build_args)
#             self.logger.info(f"Building NixOS image: {target_name}")
#             self._run_command(build_cmd)

#             # Process build output
#             result_path = str(self.tmp_dir / "result")
#             source_image = self._find_build_output(build_args.build_type, result_path)
#             self._copy_file(source_image, target_file)

#             # Calculate metadata
#             build_time = time.time() - start_time
#             file_hash = self._calculate_file_hash(target_file)
#             file_size = target_file.stat().st_size

#             return BuildConfig(
#                 image_path=target_file,
#                 image_name=target_file.name,
#                 file_size=file_size,
#                 file_hash=file_hash,
#                 success=True,
#                 build_args=build_args,
#                 build_time=build_time
#             )

#         except Exception as e:
#             self.logger.error(f"Failed to build NixOS image: {str(e)}")
#             raise NixBuildError(f"Image build failed: {str(e)}") from e

import os
import subprocess
import logging
import shutil
from pathlib import Path
from typing import Optional, Dict, Any, List
from dataclasses import dataclass, field
import hashlib
import time
from enum import Enum
import json

class BuildType(Enum):
    """Supported build types for NixOS images"""
    GCP = "gcp"    # Google Cloud Platform image
    AWS = "aws"    # Amazon Web Services image (placeholder)
    QEMU = "qemu"  # QEMU/KVM compatible image

@dataclass
class BuildArguments:
    """Contains all arguments needed for the Nix build process"""
    hostname: str
    build_type: BuildType
    google_credentials: Optional[str] = None
    extra_modules: List[str] = field(default_factory=list)
    extra_args: Dict[str, Any] = field(default_factory=dict)

    def to_nix_args(self) -> Dict[str, Any]:
        """Convert build arguments to Nix-compatible format"""
        args = {
            "hostname": self.hostname,
            "build_type": self.build_type.value,
        }

        if self.google_credentials:
            args["google_application_credentials"] = self.google_credentials

        if self.extra_args:
            args.update(self.extra_args)

        return args

@dataclass
class BuildConfig:
    """Results and metadata from a successful build"""
    image_path: Path
    image_name: str
    file_size: int
    file_hash: str
    success: bool
    build_args: BuildArguments
    build_time: float = 0.0

class NixBuildError(Exception):
    """Custom exception for Nix build failures"""
    pass

class NixImageBuilder:
    """Builds NixOS images using nix-build with proper argument handling"""

    def __init__(
        self,
        workspace_dir: str = "/workspace",
        build_jobs: int = 8,
        command_timeout: int = 1800  # 30 minutes default timeout
    ):
        self.workspace_dir = Path(workspace_dir)
        self.build_dir = self.workspace_dir / "_build"
        self.tmp_dir = self.build_dir / "tmp"
        self.out_dir = self.build_dir / "out"
        self.build_jobs = build_jobs
        self.command_timeout = command_timeout

        # Configure logging
        logging.basicConfig(
            level=logging.INFO,
            format='%(asctime)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s'
        )
        self.logger = logging.getLogger(__name__)

    def _setup_directories(self) -> None:
        """Create and set permissions for build directories"""
        for directory in [self.build_dir, self.tmp_dir, self.out_dir]:
            directory.mkdir(parents=True, exist_ok=True)
            # Ensure proper permissions
            os.chmod(directory, 0o755)

    def _cleanup_tmp(self) -> None:
        """Clean temporary build artifacts"""
        if self.tmp_dir.exists():
            shutil.rmtree(self.tmp_dir)
        self.tmp_dir.mkdir(parents=True)

    def _run_command(
        self,
        cmd: list[str],
        shell: bool = False,
        timeout: Optional[int] = None
    ) -> subprocess.CompletedProcess:
        """Execute a command with proper error handling and logging"""
        cmd_str = ' '.join(cmd) if isinstance(cmd, list) else cmd
        self.logger.debug(f"Running command: {cmd_str}")

        timeout = timeout or self.command_timeout

        try:
            start_time = time.time()
            result = subprocess.run(
                cmd,
                check=True,
                capture_output=True,
                text=True,
                shell=shell,
                timeout=timeout
            )
            duration = time.time() - start_time

            self.logger.debug(f"Command completed in {duration:.2f} seconds")
            if result.stdout:
                self.logger.debug(f"Command output: {result.stdout.strip()}")
            return result

        except subprocess.TimeoutExpired as e:
            self.logger.error(f"Command timed out after {timeout} seconds: {cmd_str}")
            raise NixBuildError(f"Command timed out: {cmd_str}") from e
        except subprocess.CalledProcessError as e:
            self.logger.error(f"Command failed with exit code {e.returncode}: {cmd_str}")
            self.logger.error(f"Standard error: {e.stderr}")
            raise NixBuildError(f"Command failed: {e.stderr}") from e

    def _get_build_attr(self, build_type: BuildType) -> str:
        """Get the appropriate Nix attribute based on build type"""
        attr_map = {
            BuildType.GCP: "gcpImage",
            BuildType.QEMU: "qemuImage",
            BuildType.AWS: "awsImage"  # Placeholder for future implementation
        }
        return attr_map.get(build_type, "gcpImage")

    def _construct_build_command(
        self,
        nix_file: str,
        build_args: BuildArguments
    ) -> list[str]:
        """Construct the nix-build command with proper arguments"""
        attr = self._get_build_attr(build_args.build_type)
        nix_args = build_args.to_nix_args()

        # Convert to Nix expression format and properly quote it
        pairs = []
        for key, value in nix_args.items():
            if isinstance(value, str):
                pairs.append(f'{key} = "{value}"')
            else:
                pairs.append(f'{key} = {value}')

        nix_expr = "{ " + "; ".join(pairs) + "; }"

        cmd = ["nix-build",
               nix_file,
               "-A", attr,
               "--arg", "config", nix_expr,
               "-o", str(self.tmp_dir / "result"),
               "-j", str(self.build_jobs),
               "--show-trace"]

        return cmd

    def _find_build_output(self, build_type: BuildType, result_path: str) -> Path:
        """Find the built image file based on build type"""
        store_path = Path(os.readlink(result_path))

        patterns = {
            BuildType.GCP: "*.raw.tar.gz",
            BuildType.QEMU: "*.qcow2",
            BuildType.AWS: "*.vhd"  # Placeholder
        }

        pattern = patterns.get(build_type, "*.raw.tar.gz")
        matches = list(store_path.glob(pattern))

        if not matches:
            raise NixBuildError(f"No image file matching {pattern} found in {store_path}")

        return matches[0]

    def _calculate_file_hash(self, file_path: Path) -> str:
        """Calculate SHA256 hash of a file"""
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()

    def _copy_file(self, source: Path, dest: Path) -> None:
        """Copy file with verification"""
        if not source.exists():
            raise NixBuildError(f"Source file not found: {source}")

        source_size = source.stat().st_size
        source_hash = self._calculate_file_hash(source)

        dest.parent.mkdir(parents=True, exist_ok=True)
        shutil.copy2(source, dest)
        os.chmod(dest, 0o644)

        dest_size = dest.stat().st_size
        dest_hash = self._calculate_file_hash(dest)

        if dest_size != source_size or dest_hash != source_hash:
            raise NixBuildError("File verification failed after copy")

    def build_image(
        self,
        nix_file: str,
        build_args: BuildArguments
    ) -> BuildConfig:
        """
        Build a NixOS image using the provided configuration

        Args:
            nix_file: Path to the Nix expression file (default.nix)
            build_args: BuildArguments instance with build configuration

        Returns:
            BuildConfig object containing build results and metadata
        """
        target_name = f"nixos-{build_args.build_type.value}-{build_args.hostname}"
        extension = ".qcow2" if build_args.build_type == BuildType.QEMU else ".raw.tar.gz"
        target_file = self.out_dir / f"{target_name}{extension}"

        try:
            self._setup_directories()
            self._cleanup_tmp()

            if target_file.exists():
                target_file.unlink()

            start_time = time.time()

            # Build the image
            build_cmd = self._construct_build_command(nix_file, build_args)
            self.logger.info(f"Building NixOS image: {target_name}")
            self._run_command(build_cmd)

            # Process build output
            result_path = str(self.tmp_dir / "result")
            source_image = self._find_build_output(build_args.build_type, result_path)
            self._copy_file(source_image, target_file)

            # Calculate metadata
            build_time = time.time() - start_time
            file_hash = self._calculate_file_hash(target_file)
            file_size = target_file.stat().st_size

            return BuildConfig(
                image_path=target_file,
                image_name=target_file.name,
                file_size=file_size,
                file_hash=file_hash,
                success=True,
                build_args=build_args,
                build_time=build_time
            )

        except Exception as e:
            self.logger.error(f"Failed to build NixOS image: {str(e)}")
            raise NixBuildError(f"Image build failed: {str(e)}") from e
