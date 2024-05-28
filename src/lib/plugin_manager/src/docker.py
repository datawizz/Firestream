


import os
import subprocess

class DockerManager:
    def __init__(self):
        self.container_name = "dependencies"
        self.workspace = "/workspace"
        self.dockerfile = "/workspace/Dockerfile"
        self.compose_project_name = os.getenv("COMPOSE_PROJECT_NAME")
        self.container_registry_url = os.getenv("CONTAINER_REGISTRY_URL")
        self.container_registry_port = os.getenv("CONTAINER_REGISTRY_PORT")
        self.git_commit_hash = os.getenv("GIT_COMMIT_HASH")
        self.image_tag = f"k3d-{self.compose_project_name}.{self.container_registry_url}:{self.container_registry_port}/{self.container_name}:{self.git_commit_hash}"

    def build_image(self):
        subprocess.run([
            "docker", "build", "--target", self.container_name,
            "-f", self.dockerfile,
            "-t", self.image_tag,
            "."
        ], check=True, cwd=self.workspace)

    def push_image(self):
        subprocess.run(["docker", "push", self.image_tag], check=True)

    def build_and_push_image(self):
        self.build_image()
        self.push_image()


# Using the DockerManager class
if __name__ == "__main__":
    manager = DockerManager()
    manager.build_and_push_image()





