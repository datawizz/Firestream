from pathlib import Path
import shutil
from jinja2 import Environment, FileSystemLoader
from InquirerPy import prompt

class TemplateProcessor:
    def __init__(self, template_dir, output_dir):
        self.template_dir = Path(template_dir)
        self.output_dir = Path(output_dir)
        self.env = Environment(loader=FileSystemLoader(str(self.template_dir)))

    def get_user_input(self):
        questions = [
            {"type": "input", "message": "Project name:", "name": "project_name", "default": "my-project"},
            {"type": "input", "message": "Project version:", "name": "version", "default": "0.1.0"},
            {"type": "input", "message": "Project description:", "name": "description", "default": "A Helm chart for Kubernetes"},
            {"type": "input", "message": "Docker image:", "name": "docker_image", "default": "python:3.9-slim"},
            {"type": "number", "message": "Container port:", "name": "port", "default": 8080},
            {"type": "number", "message": "Number of replicas:", "name": "replicas", "default": 1},
            {"type": "input", "message": "CPU request:", "name": "cpu_request", "default": "100m"},
            {"type": "input", "message": "Memory request:", "name": "memory_request", "default": "128Mi"},
            {"type": "input", "message": "CPU limit:", "name": "cpu_limit", "default": "200m"},
            {"type": "input", "message": "Memory limit:", "name": "memory_limit", "default": "256Mi"},
            {"type": "confirm", "message": "Enable ingress?", "name": "enable_ingress", "default": False},
            {
                "type": "input",
                "message": "Ingress host:",
                "name": "ingress_host",
                "default": "example.local",
                "when": lambda x: x["enable_ingress"]
            },
            {
                "type": "input",
                "message": "Ingress path:",
                "name": "ingress_path",
                "default": "/",
                "when": lambda x: x["enable_ingress"]
            },
            {"type": "confirm", "message": "Proceed with these settings?", "name": "confirm"}
        ]
        return prompt(questions)

    def build_context(self, answers):
        context = {
            "chart": {
                "name": answers["project_name"],
                "version": answers["version"],
                "description": answers["description"]
            },
            "image": {
                "repository": answers["docker_image"].split(":")[0],
                "tag": answers["docker_image"].split(":")[1],
                "pullPolicy": "IfNotPresent"
            },
            "service": {
                "port": answers["port"]
            },
            "replicaCount": answers["replicas"],
            "resources": {
                "requests": {
                    "cpu": answers["cpu_request"],
                    "memory": answers["memory_request"]
                },
                "limits": {
                    "cpu": answers["cpu_limit"],
                    "memory": answers["memory_limit"]
                }
            },
            "ingress": {
                "enabled": answers["enable_ingress"]
            }
        }

        if answers["enable_ingress"]:
            context["ingress"].update({
                "host": answers["ingress_host"],
                "path": answers["ingress_path"]
            })

        return context

    def process_files(self, context):
        rendered_dir = self.output_dir / context["chart"]["name"]

        # Delete existing project directory if it exists, then copy fresh
        if rendered_dir.exists():
            shutil.rmtree(rendered_dir)
        shutil.copytree(self.template_dir, rendered_dir)

        # Relative to the template directory
        _templates = ["helm/template/Chart.yaml", "helm/template/values.yaml"]

        # Process only Chart.yaml and values.yaml through Jinja
        for template_file in _templates:
            template = self.env.get_template(template_file)
            output_file = rendered_dir / template_file
            output_file.write_text(template.render(**context))

        # TODO: Add further per-file Jinja processing here if needed.

        # Rename the docker directory to the project name
        (rendered_dir / "docker/new_project_template").rename(rendered_dir / "docker" / context["chart"]["name"])

def main():
    processor = TemplateProcessor("/workspace/src/deploy/template", "/workspace/src/deploy/rendered")
    answers = processor.get_user_input()

    if not answers["confirm"]:
        print("Operation cancelled.")
        return

    context = processor.build_context(answers)
    processor.process_files(context)
    print("✨ Templates processed successfully!")

if __name__ == "__main__":
    main()
