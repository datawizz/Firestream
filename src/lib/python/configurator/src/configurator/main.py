# from pathlib import Path
# import shutil
# from jinja2 import Environment, FileSystemLoader
# from InquirerPy import prompt

# class TemplateProcessor:
#     def __init__(self, template_dir, output_dir):
#         self.template_dir = Path(template_dir)
#         self.output_dir = Path(output_dir)
#         self.env = Environment(loader=FileSystemLoader(str(self.template_dir)))

#     def get_user_input(self):
#         questions = [
#             {"type": "input", "message": "Project name:", "name": "project_name", "default": "my-project"},
#             {"type": "input", "message": "Project version:", "name": "version", "default": "0.1.0"},
#             {"type": "input", "message": "Project description:", "name": "description", "default": "A Helm chart for Kubernetes"},
#             {"type": "input", "message": "Docker image:", "name": "docker_image", "default": "python:3.9-slim"},
#             {"type": "number", "message": "Container port:", "name": "port", "default": 8080},
#             {"type": "number", "message": "Number of replicas:", "name": "replicas", "default": 1},
#             {"type": "input", "message": "CPU request:", "name": "cpu_request", "default": "100m"},
#             {"type": "input", "message": "Memory request:", "name": "memory_request", "default": "128Mi"},
#             {"type": "input", "message": "CPU limit:", "name": "cpu_limit", "default": "200m"},
#             {"type": "input", "message": "Memory limit:", "name": "memory_limit", "default": "256Mi"},
#             {"type": "confirm", "message": "Enable ingress?", "name": "enable_ingress", "default": False},
#             {
#                 "type": "input",
#                 "message": "Ingress host:",
#                 "name": "ingress_host",
#                 "default": "example.local",
#                 "when": lambda x: x["enable_ingress"]
#             },
#             {
#                 "type": "input",
#                 "message": "Ingress path:",
#                 "name": "ingress_path",
#                 "default": "/",
#                 "when": lambda x: x["enable_ingress"]
#             },
#             {"type": "confirm", "message": "Proceed with these settings?", "name": "confirm"}
#         ]
#         return prompt(questions)

#     def build_context(self, answers):
#         context = {
#             "chart": {
#                 "name": answers["project_name"],
#                 "version": answers["version"],
#                 "description": answers["description"]
#             },
#             "image": {
#                 "repository": answers["docker_image"].split(":")[0],
#                 "tag": answers["docker_image"].split(":")[1],
#                 "pullPolicy": "IfNotPresent"
#             },
#             "service": {
#                 "port": answers["port"]
#             },
#             "replicaCount": answers["replicas"],
#             "resources": {
#                 "requests": {
#                     "cpu": answers["cpu_request"],
#                     "memory": answers["memory_request"]
#                 },
#                 "limits": {
#                     "cpu": answers["cpu_limit"],
#                     "memory": answers["memory_limit"]
#                 }
#             },
#             "ingress": {
#                 "enabled": answers["enable_ingress"]
#             }
#         }

#         if answers["enable_ingress"]:
#             context["ingress"].update({
#                 "host": answers["ingress_host"],
#                 "path": answers["ingress_path"]
#             })

#         return context

#     def process_files(self, context):
#         rendered_dir = self.output_dir / context["chart"]["name"]

#         # Delete existing project directory if it exists, then copy fresh
#         if rendered_dir.exists():
#             shutil.rmtree(rendered_dir)
#         shutil.copytree(self.template_dir, rendered_dir)

#         # Relative to the template directory
#         _templates = ["helm/template/Chart.yaml", "helm/template/values.yaml"]

#         # Process only Chart.yaml and values.yaml through Jinja
#         for template_file in _templates:
#             template = self.env.get_template(template_file)
#             output_file = rendered_dir / template_file
#             output_file.write_text(template.render(**context))

#         # TODO: Add further per-file Jinja processing here if needed.

#         # Rename the docker directory to the project name
#         (rendered_dir / "docker/new_project_template").rename(rendered_dir / "docker" / context["chart"]["name"])

# def main():
#     processor = TemplateProcessor("/workspace/src/deploy/template", "/workspace/src/deploy/rendered")
#     answers = processor.get_user_input()

#     if not answers["confirm"]:
#         print("Operation cancelled.")
#         return

#     context = processor.build_context(answers)
#     processor.process_files(context)
#     print("✨ Templates processed successfully!")

# if __name__ == "__main__":
#     main()


from pathlib import Path
import shutil
import json
from typing import Optional
from pydantic import BaseModel, Field, validator
from jinja2 import Environment, FileSystemLoader
from InquirerPy import prompt

class ImageConfig(BaseModel):
    repository: str
    tag: str
    pull_policy: str = Field(default="IfNotPresent", alias="pullPolicy")

    @validator('repository', 'tag', pre=True)
    def split_docker_image(cls, v, values, field):
        if field.name == 'repository' and ':' in v:
            # If full docker image is provided in repository field
            repository, tag = v.split(':', 1)
            if 'tag' not in values:
                values['tag'] = tag
            return repository
        return v

class ResourceRequirements(BaseModel):
    cpu: str
    memory: str

class Resources(BaseModel):
    requests: ResourceRequirements
    limits: ResourceRequirements

class IngressConfig(BaseModel):
    enabled: bool = False
    host: Optional[str] = None
    path: Optional[str] = "/"

    @validator('host', 'path')
    def validate_ingress_fields(cls, v, values):
        if values.get('enabled', False):
            if v is None:
                raise ValueError("host and path are required when ingress is enabled")
        return v

class ChartConfig(BaseModel):
    name: str = Field(..., alias="project_name")
    version: str
    description: str = Field(default="A Helm chart for Kubernetes")

class ServiceConfig(BaseModel):
    port: int = Field(default=8080, ge=1, le=65535)

class TemplateContext(BaseModel):
    chart: ChartConfig
    image: ImageConfig
    service: ServiceConfig
    replica_count: int = Field(default=1, alias="replicaCount", ge=1)
    resources: Resources
    ingress: IngressConfig

    @validator('image', pre=True)
    def validate_docker_image(cls, v):
        if isinstance(v, str):
            repository, tag = v.split(':', 1)
            return {"repository": repository, "tag": tag}
        return v

class TemplateProcessor:
    def __init__(self, template_dir: str, output_dir: str):
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
            {"type": "number", "message": "Number of replicas:", "name": "replica_count", "default": 1},
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
        answers = prompt(questions)
        if not answers["confirm"]:
            return None
        return self._convert_answers_to_context(answers)

    def _convert_answers_to_context(self, answers: dict) -> TemplateContext:
        context_dict = {
            "chart": {
                "project_name": answers["project_name"],
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
            "replicaCount": answers["replica_count"],
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
            context_dict["ingress"].update({
                "host": answers["ingress_host"],
                "path": answers["ingress_path"]
            })

        return TemplateContext.parse_obj(context_dict)

    def load_json_input(self, json_path: str) -> TemplateContext:
        with open(json_path, 'r') as f:
            data = json.load(f)
        return TemplateContext.parse_obj(data)

    def process_files(self, context: TemplateContext):
        rendered_dir = self.output_dir / context.chart.name

        # Delete existing project directory if it exists, then copy fresh
        if rendered_dir.exists():
            shutil.rmtree(rendered_dir)
        shutil.copytree(self.template_dir, rendered_dir)

        # Relative to the template directory
        _templates = ["helm/template/Chart.yaml", "helm/template/values.yaml"]

        # Convert Pydantic model to dict for Jinja2 templating
        context_dict = context.dict(by_alias=True)

        # Process only Chart.yaml and values.yaml through Jinja
        for template_file in _templates:
            template = self.env.get_template(template_file)
            output_file = rendered_dir / template_file
            output_file.write_text(template.render(**context_dict))

        # Rename the docker directory to the project name
        (rendered_dir / "docker/new_project_template").rename(rendered_dir / "docker" / context.chart.name)

def main():
    import argparse

    parser = argparse.ArgumentParser(description='Process Helm chart templates')
    parser.add_argument('--json', type=str, help='Path to JSON input file')
    parser.add_argument('--template-dir', type=str, default='/workspace/src/deploy/template',
                      help='Template directory path')
    parser.add_argument('--output-dir', type=str, default='/workspace/src/deploy/rendered',
                      help='Output directory path')

    args = parser.parse_args()

    processor = TemplateProcessor(args.template_dir, args.output_dir)

    if args.json:
        try:
            context = processor.load_json_input(args.json)
        except Exception as e:
            print(f"Error loading JSON file: {e}")
            return
    else:
        context = processor.get_user_input()
        if context is None:
            print("Operation cancelled.")
            return

    processor.process_files(context)
    print("✨ Templates processed successfully!")

if __name__ == "__main__":
    main()
