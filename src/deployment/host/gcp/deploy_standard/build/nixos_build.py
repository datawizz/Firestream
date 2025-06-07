import os
import subprocess
import logging
import shutil
from pathlib import Path
from typing import Optional, Dict
from dataclasses import dataclass
import hashlib
import time

@dataclass
class BuildResult:
    """Contains information about the built image"""
    image_path: Path
    image_name: str
    file_size: int
    file_hash: str
    success: bool
    hostname: str

class NixBuildError(Exception):
    """Custom exception for Nix build failures."""
    pass

class NixImageBuilder:
    """Handles building Nix images for Google Compute Engine with Pulumi integration."""

    def __init__(self,
                 workspace_dir: str = "/workspace",
                 build_jobs: int = 10,
                 command_timeout: int = 600,
                 extra_args: Optional[Dict[str, str]] = None):
        self.workspace_dir = Path(workspace_dir)
        self.build_dir = self.workspace_dir / "_build"
        self.tmp_dir = self.build_dir / "tmp"
        self.out_dir = self.build_dir / "out"
        self.build_jobs = build_jobs
        self.command_timeout = command_timeout
        self.extra_args = extra_args or {}

        # Configure logging with detailed format for debugging
        logging.basicConfig(
            level=logging.DEBUG,
            format='%(asctime)s - %(levelname)s - [%(filename)s:%(lineno)d] - %(message)s'
        )
        self.logger = logging.getLogger(__name__)

    def _run_command(self, cmd: list[str], shell: bool = False, timeout: Optional[int] = None) -> subprocess.CompletedProcess:
        """Execute a command with proper error handling, logging, and timeout."""
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
            end_time = time.time()

            self.logger.debug(f"Command completed in {end_time - start_time:.2f} seconds")
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

    def _setup_directories(self) -> None:
        """Setup required directories with proper permissions."""
        self.logger.info("Setting up build directories...")
        for directory in [self.out_dir, self.tmp_dir]:
            try:
                directory.mkdir(parents=True, exist_ok=True)
                self._run_command(["sudo", "chown", "-R", f"{os.getuid()}:{os.getgid()}", str(directory)])
            except Exception as e:
                self.logger.error(f"Failed to setup directory {directory}: {e}")
                raise NixBuildError(f"Directory setup failed: {str(e)}") from e

    def _cleanup_tmp(self) -> None:
        """Clean up temporary build directory."""
        self.logger.info("Cleaning up temporary directory...")
        try:
            if self.tmp_dir.exists():
                shutil.rmtree(self.tmp_dir)
            self.tmp_dir.mkdir(parents=True, exist_ok=True)
        except Exception as e:
            self.logger.error(f"Failed to cleanup temporary directory: {e}")
            raise NixBuildError(f"Cleanup failed: {str(e)}") from e

    def _find_nix_store_image(self, nix_path: str) -> Path:
        """Find the image file in the Nix store directory."""
        try:
            # Read the symlink to get the actual store path
            store_path = Path(os.readlink(nix_path))
            self.logger.debug(f"Resolved store path: {store_path}")

            # Look for the image file in the store path
            image_files = list(store_path.glob("*.raw.tar.gz"))
            if not image_files:
                raise NixBuildError(f"No image files found in Nix store path: {store_path}")

            self.logger.info(f"Found image file: {image_files[0]}")
            return image_files[0]
        except OSError as e:
            self.logger.error(f"Failed to read store path: {e}")
            raise NixBuildError(f"Failed to access Nix store path: {e}") from e

    def _calculate_file_hash(self, file_path: Path) -> str:
        """Calculate SHA256 hash of a file."""
        sha256_hash = hashlib.sha256()
        with open(file_path, "rb") as f:
            for byte_block in iter(lambda: f.read(4096), b""):
                sha256_hash.update(byte_block)
        return sha256_hash.hexdigest()

    def _copy_file(self, source: Path, dest: Path) -> None:
        """Copy file with progress logging and verification."""
        try:
            self.logger.info(f"Starting file copy from {source} to {dest}")

            if not source.exists():
                raise NixBuildError(f"Source file not found: {source}")

            # Get source file details
            source_size = source.stat().st_size
            source_hash = self._calculate_file_hash(source)
            self.logger.info(f"Source file size: {source_size/1024/1024:.2f} MB")

            # Ensure destination directory exists
            dest.parent.mkdir(parents=True, exist_ok=True)

            # Copy the file
            shutil.copy2(source, dest)

            # Fix permissions on the destination file
            os.chmod(dest, 0o644)

            # Verify the copy
            if not dest.exists():
                raise NixBuildError(f"Destination file not found after copy: {dest}")

            dest_size = dest.stat().st_size
            dest_hash = self._calculate_file_hash(dest)

            if dest_size != source_size:
                raise NixBuildError(
                    f"File size mismatch after copy: source={source_size}, dest={dest_size}"
                )

            if dest_hash != source_hash:
                raise NixBuildError(
                    f"File hash mismatch after copy: source={source_hash}, dest={dest_hash}"
                )

            self.logger.info("File copy completed and verified successfully")

        except Exception as e:
            self.logger.error(f"File copy failed: {str(e)}")
            raise NixBuildError(f"Failed to copy file: {str(e)}") from e


    def build_image(self, nix_config_path: str) -> BuildResult:
        """
        Build the Nix image for GCE.
        """
        target_file = self.out_dir / "nixos-image-x86_64-linux.raw.tar.gz"

        try:
            self._setup_directories()
            self._cleanup_tmp()

            if target_file.exists():
                self.logger.info("Removing existing target file...")
                target_file.unlink()

            # Create a temporary Nix expression that properly sets up the module
            nix_expr = self.tmp_dir / "configuration.nix"
            nix_expr_content = '''
    { config, pkgs, lib, ... }:
    {
    imports = [ %s ];

    _module.args = {
        serviceAccountFile = "%s";
        hostname = "%s";
    };
    }
    ''' % (
                nix_config_path,
                self.extra_args.get('serviceAccountFile', ''),
                self.extra_args.get('hostname', '')
            )

            # Write the expression file
            nix_expr.write_text(nix_expr_content)
            self.logger.debug(f"Created Nix expression:\n{nix_expr_content}")

            # Build the nix-build command using the module
            build_cmd = [
                "nix-build",
                "<nixpkgs/nixos/lib/eval-config.nix>",
                "-A", "config.system.build.googleComputeImage",
                "--arg", "modules", f"[ {nix_expr} ]",
                "--argstr", "system", "x86_64-linux",
                "-o", str(self.tmp_dir / "result"),
                "-j", str(self.build_jobs),
                "--show-trace"
            ]

            self.logger.info(f"Building Nix image with command: {' '.join(build_cmd)}")
            result = self._run_command(build_cmd)

            nix_store_path = str(self.tmp_dir / "result")
            self.logger.info(f"Build completed, store path: {nix_store_path}")

            source_image = self._find_nix_store_image(nix_store_path)
            self.logger.info(f"Found built image at: {source_image}")

            self._copy_file(source_image, target_file)

            file_hash = self._calculate_file_hash(target_file)
            file_size = target_file.stat().st_size

            return BuildResult(
                image_path=target_file,
                image_name=target_file.name,
                file_size=file_size,
                file_hash=file_hash,
                hostname=self.extra_args.get('hostname', ''),
                success=True
            )

        except Exception as e:
            self.logger.error(f"Failed to build Nix image: {str(e)}")
            raise NixBuildError(f"Image build failed: {str(e)}") from e
