import os
from pydantic import BaseModel
from typing import Optional, List
import yaml
import unittest
import re
import logging

from models import ManifestModel
from k8_manager import HelmManager, ManagerType


class PluginManager:
    """
    The Plugin Manager is responsible for loading all plugins from the source code
    """

    def __init__(self):
        self._PATHS = ["/workspace/src/plugins", "/workspace/submodules/plugins"]
        self.feature_flags = self.load_feature_flags()
        logging.info(self.feature_flags)
        self.plugins = self.load_manifests(self._PATHS)
        self.helm_manager = HelmManager(manager_type=ManagerType.ADMIN)

    def load_feature_flags(self):
        return set([key for key in os.environ if key.startswith("FEATURE_FLAG_")])

    def check_feature_flags(self, manifest):
        """
        Assume that missing feature flags means that there are no dependencies
        """
        return (
            set(manifest.feature_flags).issubset(set(self.feature_flags))
            if manifest.feature_flags
            else True
        )

    def load_manifests(self, dir_paths) -> List[ManifestModel]:
        _PLUGINS = []
        manifest_pattern = re.compile(r"manifest\.\w+\.yaml")

        for dir_path in dir_paths:
            for root, dirs, files in os.walk(dir_path):
                for file in files:
                    if manifest_pattern.match(file):
                        _filepath = os.path.join(root, file)
                        with open(_filepath, "r") as _file:
                            _data = yaml.safe_load(_file)
                            _data.update({"path": root})
                            try:
                                manifest = ManifestModel(**_data)
                                if self.check_feature_flags(manifest):
                                    _PLUGINS.append(manifest)
                                else:
                                    logging.error(
                                        f"Skipping plugin {manifest.name} due to mismatched feature flags"
                                    )
                            except Exception as e:
                                print(e)

        return _PLUGINS

    def process_plugin(self, plugin: ManifestModel):
        self.helm_manager.upgrade_install(plugin=plugin)




if __name__ == "__main__":
    plugin = PluginManager()
    print([p.json() for p in plugin.plugins])
    print(plugin.feature_flags)
    for p in plugin.plugins:
        plugin.process_plugin(p)
    pass


# class TestPluginManager(unittest.TestCase):
#     def setUp(self):
#         self.plugin = PluginManager(feature_flags=[])

#     def test_load_manifest_files_no_flags(self):
#         plugins = self.plugin.load_manifest_files(self.plugin._PATHS)
#         for plugin in plugins:
#             self.assertIsInstance(plugin, ManifestModel)

#     def test_load_manifest_files_with_flags(self):
#         feature_flags = ["FEATURE_FLAG_ONE", "FEATURE_FLAG_TWO"]
#         self.plugin.feature_flags = feature_flags
#         plugins = self.plugin.load_manifest_files(self.plugin._PATHS)
#         for plugin in plugins:
#             self.assertTrue(set(plugin.feature_flags).issubset(set(feature_flags)))

# if __name__ == "__main__":
#     unittest.main()
