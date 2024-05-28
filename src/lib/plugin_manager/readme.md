# Plugin Manager

The Plugin Manager is itself a Plugin for Fireworks. It's responsibility is to manage other plugins.
It also serves as a self documenting reference for plugins. Each plugin is required to implement the following:

1. A manifest.json file in the root directory of a folder.
    Each folder of the plugin directory is walked and is evualted as a potential plugin based on the presence of a manigest.json file
    The manifest.json file constains a JSON schema for each DataModel that is produced by the plugin.
    It also describes what the URI of the JSON schema of reference is.

The Plugin Manager contains a Dockerfile which allows it to run as a container init for upgrades of infrustructure.
It also enables the plugins to be mounted as DAGs in Airflow at runtime.
