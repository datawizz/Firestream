# Submodules

This folder contains the source code for projects which are used extensively.
The git feature of submodules is used as a shallow but absolute reference a commit.
The format is <organization> / <repo name> 



# FireStream Plugins

Plugins are self contained folders which add some sort of functionality to FireStream. They are almost always external git repos. They may be numerous and are left to a users choice, therefore they are not submodules of the project as there is no dependency except perhaps among plugins which is handled by the plugin system (it won't load a plugins unless all dependencies are also present).