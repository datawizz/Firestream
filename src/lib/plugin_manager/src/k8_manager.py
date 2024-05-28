import os
from enum import Enum
import subprocess
from jinja2 import Environment, FileSystemLoader
import logging
import tempfile
from kubernetes import client, config
from kubernetes.client import configuration
from jinja2 import Template
from models import ManifestModel


class ManagerType(Enum):
    ADMIN = 'admin'
    RBAC = 'RBAC'

class HelmManager:
    def __init__(self, manager_type: ManagerType):
        self.manager_type = manager_type

        if self.manager_type == ManagerType.ADMIN:
            self._init_admin()
        elif self.manager_type == ManagerType.RBAC:
            self._init_RBAC()
        else:
            raise ValueError(f"Invalid manager type: {manager_type}")

        self.v1 = client.CoreV1Api()

    def _init_admin(self):
        self.kube_config_path = os.path.expanduser("~/.kube/config")
        assert os.path.exists(self.kube_config_path), f"KUBECONFIG {self.kube_config_path} does not exist"
        config.load_kube_config(config_file=self.kube_config_path)

    def _init_RBAC(self):
        configuration.api_key['authorization'] = "Your Service Account Token"
        configuration.ssl_ca_cert = "Path to the Certificate File"
        configuration.api_key_prefix['authorization'] = 'Bearer'
        config.load_kube_config(client_configuration=configuration)


    def list_pods(self):
        print("\nList of pods:")
        for i in self.v1.list_pod_for_all_namespaces().items:
            print("%s\t%s\t%s" % (i.status.pod_ip, i.metadata.namespace, i.metadata.name))


    def list_namespaces(self):
        ret = self.v1.list_namespace()
        return {i.metadata.name: i.status.phase for i in ret.items}

    def upgrade_install(self, plugin: ManifestModel):
        print(F"Upgrading or installing plugin {plugin.name}")


        if not plugin.chart_dir:
            logging.error(f"Invalid chart directory: {plugin.chart_dir}")
            return False

        # Set environment variables
        env_vars = {k: v for k, v in os.environ.items()}

        # Read the Jinja2 template
        with open(plugin.template_values, 'r') as f:
            template_str = f.read()
        

        # Render template
        template = Template(template_str)
        rendered_template = template.render(env_vars)

        # Create temporary rendered template file, then apply it
        with tempfile.NamedTemporaryFile(delete=True) as tmp:
            # Write rendered template to temp file
            tmp.write(rendered_template.encode())
            tmp.flush()
            print(tmp)
            # Use temp file in helm upgrade/install command
            subprocess.run(["helm", "upgrade", plugin.name, plugin.path, "--install", "--values", tmp.name], check=True)



        # subprocess.run(["helm", "upgrade", plugin.name, plugin.path, "--install", "--values", plugin.values], check=True)

    def helm_delete(self, plugin: ManifestModel):
        subprocess.run(["helm", "delete", plugin.name], check=True)

    # def apply_chart(self, plugin: ManifestModel, values: str):
    #     self.helm_install_or_upgrade(plugin, values)

    def _validate(self, chart_dir: str, templated_value: str):
        if not os.path.isdir(chart_dir):
            logging.error(f"Invalid chart directory: {chart_dir}")
            return False

        if not os.path.isfile(templated_value):
            logging.error(f"Invalid file path: {templated_value}")
            return False

        return True

    def patch_values(self, template_directory: str, template_file: str, variables: dict) -> str:
        file_loader = FileSystemLoader(template_directory)
        env = Environment(loader=file_loader)
        template = env.get_template(template_file)
        output = template.render(variables)

        with tempfile.NamedTemporaryFile(delete=False, mode='w') as f:
            f.write(output)
            temp_file_name = f.name

        return temp_file_name

    def apply_overrides(self, chart_dir: str, templated_value: str):
        if not self._validate(chart_dir, templated_value):
            logging.error("Invalid inputs. Aborting operation.")
            return

        os.system(f"helm upgrade --install {chart_dir} -f {templated_value}")
        os.remove(templated_value)


import pytest
from models import ManifestModel

@pytest.fixture
def setup():
    manager = HelmManager(ManagerType.ADMIN)
    sample_plugin = ManifestModel(
        name="testplugin",
        path="/workspace/src/api/plugin_manager/tests/test_plugin",
        description="Sample plugin for testing",
        version="0.0.1",
        author="Sample Author",
        chart_dir="/workspace/src/api/plugin_manager/tests/test_plugin/test_chart",
        template_values="/workspace/src/api/plugin_manager/tests/test_plugin/template.values.yaml",
        feature_flags=["FEATURE_FLAG_TEST"]
    )
    return manager, sample_plugin

# def test_list_pods(setup):
#     manager, _ = setup
#     manager.list_pods()

def test_helm_connectivity(setup):
    manager, _ = setup
    assert manager.test_helm_connectivity()

def test_validate(setup):
    manager, sample_plugin = setup
    assert manager._validate(sample_plugin.chart_dir, sample_plugin.template_values)

# def test_invalid_chart_dir(setup):
#     manager, sample_plugin = setup
#     sample_plugin.chart_dir = "/invalid_dir"
#     assert not manager._validate(sample_plugin.chart_dir, sample_plugin.template_values)

# def test_invalid_file_path(setup):
#     manager, sample_plugin = setup
#     sample_plugin.template_values = "/invalid_file"
#     assert not manager._validate(sample_plugin.chart_dir, sample_plugin.template_values)



if __name__ == "__main__":
    pytest.main(["-v", "-s", f"{__file__}"])


if __name__ == "__main__":
    
    c = HelmManager(ManagerType.ADMIN)
    print(c.test_helm_connectivity())
    print(c.list_namespaces())


if __name__ == '__main__':
    manager = HelmManager(ManagerType.ADMIN)
    manager.list_pods()
