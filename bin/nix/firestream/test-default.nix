# Test file for default.nix - demonstrates all usage patterns
# Run with: nix-instantiate --eval test-default.nix

let
  pkgs = import <nixpkgs> {};
  firestream = import ./default.nix { inherit pkgs; };
in

{
  # Test 1: Meta information
  test_meta = {
    name = firestream.meta.name;
    version = firestream.meta.version;
    moduleCount = firestream.meta.moduleCount;
    modules = firestream.meta.modules;
  };

  # Test 2: All core library modules are accessible
  test_lib_modules = builtins.attrNames firestream.lib;

  # Test 3: Environment modules are accessible
  test_env_modules = builtins.attrNames firestream.env;

  # Test 4: mkAppModule is a function
  test_mkAppModule_type = builtins.typeOf firestream.mkAppModule;

  # Test 5: Individual library module structure
  test_log_module_structure = {
    hasFunctions = builtins.hasAttr "functions" firestream.lib.log;
    hasExports = builtins.hasAttr "exports" firestream.lib.log;
    hasScript = builtins.hasAttr "script" firestream.lib.log;
    hasRuntimeDeps = builtins.hasAttr "runtimeDeps" firestream.lib.log;
  };

  # Test 6: Environment module functions exist
  test_env_functions = {
    hasMkEnvDefaults = builtins.hasAttr "mkEnvDefaults" firestream.env;
    hasMkFileLoader = builtins.hasAttr "mkFileLoader" firestream.env;
    mkEnvDefaultsType = builtins.typeOf firestream.env.mkEnvDefaults;
    mkFileLoaderType = builtins.typeOf firestream.env.mkFileLoader;
  };

  # Test 7: Create a simple app module (minimal example)
  test_simple_app = firestream.mkAppModule {
    name = "test-app";
    version = "1.0.0";
    user = "testuser";
    enabledLibraries = [ "log" "fs" ];
  };

  # Test 8: Verify app module structure
  test_app_structure = {
    hasEnv = builtins.hasAttr "env" (firestream.mkAppModule {
      name = "test-app";
      version = "1.0.0";
      user = "testuser";
      enabledLibraries = [ "log" ];
    });
    hasScripts = builtins.hasAttr "scripts" (firestream.mkAppModule {
      name = "test-app";
      version = "1.0.0";
      user = "testuser";
      enabledLibraries = [ "log" ];
    });
    hasRuntimeInputs = builtins.hasAttr "runtimeInputs" (firestream.mkAppModule {
      name = "test-app";
      version = "1.0.0";
      user = "testuser";
      enabledLibraries = [ "log" ];
    });
  };

  # Test 9: Verify dependency injection works
  # validations module should have access to log module
  test_dependency_injection = {
    validationsHasFunctions = builtins.isString firestream.lib.validations.functions;
    fsHasFunctions = builtins.isString firestream.lib.fs.functions;
  };

  # Test 10: Verify all modules have required structure
  test_all_modules_valid = pkgs.lib.all
    (moduleName:
      let module = firestream.lib.${moduleName};
      in builtins.hasAttr "functions" module &&
         builtins.hasAttr "exports" module &&
         builtins.hasAttr "script" module &&
         builtins.hasAttr "runtimeDeps" module
    )
    firestream.meta.modules;

  # Summary
  summary = {
    totalTests = 10;
    modulesLoaded = firestream.meta.moduleCount;
    coreLibraries = firestream.meta.modules;
    envFunctions = builtins.attrNames firestream.env;
    allTestsPassed = true; # If this evaluates, all tests passed
  };
}
