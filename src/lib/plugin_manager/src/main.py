
# import importlib
# from typing import Callable
# from etl_lib import DataContext, DataModel
# import json

# import os.path


# SRC_PLUGIN_DIRS = ["/workspace/src/app/plugins", "/workspace/submodules/plugins"] #os.path.join(os.path.dirname(os.path.abspath(__file__)), 'plugins')

# for dir in SRC_PLUGIN_DIRS:

#     for path, directories, files in os.walk(dir):
#         if "manifest.json" in files:
#             print(path, directories, files)
#             # /workspace/src/app/plugins/plugin_manager ['api', 'src'] ['readme.md', 'manifest.json']
#             fullpath = os.path.join(path, "manifest.json")
#             print(fullpath)
#             with open(fullpath, 'r') as f:

#                 manifest = json.load(f)
#                 print(manifest)

# from os import path

# import yaml

# from kubernetes import client, config

# config.load_kube_config("~/.kube/confi")
# kubectl config current-context


# def main():
#     config.load_kube_config()

#     with open(path.join(path.dirname(__file__), "deployment-1.yaml")) as f:
#         dep = yaml.safe_load(f)
#         k8s_apps_v1 = client.AppsV1Api()
#         resp = k8s_apps_v1.create_namespaced_deployment(
#             body=dep, namespace="default")
#         print("Deployment created. status='%s'" % resp.metadata.name)


# if __name__ == '__main__':
#     main()


# class PluginManager():

#     """
#     The PluginManager is responsible for loading and initializing plugins

#     Plugins are self describing packages of code, usually hosted as a git repository,
#     that are loaded during the deployment phase to provide new data interfaces to Fireworks.

#     Data Sources and Data Sinks ('interfaces') are assumed to be updated more frequently than Fireworks itself.

#     #TODO create the ability for the user to add plugins via the UI and with LLM interface :)


#     Plugins are required to implement a manifest.json file that describes the plugin and its interfaces.


#     """


#     def __init__(self) -> None:
#         pass

    

#     def initialize(self) -> dict[str, Callable[..., DataContext]]: # type: ignore
#         """
#         This method is called by the DataFactory or DataContext to initialize the plugins

#         By default the internal plugins are loaded from /src/app/plugins
#         Then the user defined plugins are loaded by walking the /src/submodules/plugins directory
        
#         Broken plugins are logged and skipped.
#         """
#         # creation_function = dict[str, Callable[..., DataContext]] = {}
#         pass

#     def import_module(self, module_name: str) -> None: # Plugin
#         """
#         Import a module and return the PluginInterface
#         """
#         return importlib.import_module(module_name) # type: ignore

#     def load_plugins(self, plugins: list[str]) -> None:
#         """
#         Load all plugins in the given list
#         """
#         for plugin in plugins:
#             plugin = self.import_module(plugin)
#             plugin.initialize()



#     def register(self, name: str, func: Callable[..., DataContext]) -> Callable[..., DataContext]:
#         """
#         Registers a DataContext as eligible for creation by the factory
#         """
#         self.creation_function[name] = func
#         return func





# import os
# import yaml
# from kubernetes import client, config

# class PluginInManager:
#     def __init__(self):
#         config.load_kube_config()
#         self.v1 = client.CoreV1Api()
#         _PATHS = ["/workspace/src/plugins", "/workspace/submodules/plugins"]

#     def list_namespaces(self):
#         ret = self.v1.list_namespace()
#         return {i.metadata.name: i.status.phase for i in ret.items}

#     def load_manifest_files(self, dir_paths: list):
#         _PLUGINS = []
#         for dir_path in dir_paths:
#             for root, dirs, files in os.walk(dir_path):
#                 if '__manifest__.yaml' in files:
#                     _filepath = os.path.join(root, '__manifest__.yaml')
#                     with open(_filepath, 'r') as _file:
#                         _data = yaml.safe_load(_file)
#                         _data.update({'path': root})
#                         _PLUGINS.append(_data)
#         return _PLUGINS


# import unittest

# class TestPluginInManager(unittest.TestCase):
#     def setUp(self):
#         self.plugin = PluginInManager()

#     def test_list_namespaces(self):
#         namespaces = self.plugin.list_namespaces()
#         self.assertIsInstance(namespaces, dict)

# if __name__ == "__main__":
#     unittest.main()



from plugin_manager import PluginManager

plugin_manager = PluginManager()

for plugin in plugin_manager.plugins:
    print(plugin.name, plugin.path, plugin.feature_flags)
    plugin_manager.process_plugin(plugin)