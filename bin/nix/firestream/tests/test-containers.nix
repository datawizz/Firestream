# Tests for the container module factories
# Copyright Firestream. Apache-2.0 License.
{ pkgs, firestream }:

let
  lib = pkgs.lib;

  # Test that container factories are exported
  testContainerFactoriesExist = pkgs.runCommand "test-container-factories-exist" {} ''
    echo "Testing container factories are exported..."

    # Check that mkContainerModule exists
    ${if firestream ? mkContainerModule then ''
      echo "PASS: mkContainerModule is exported"
    '' else ''
      echo "FAIL: mkContainerModule is not exported"
      exit 1
    ''}

    # Check that mkPythonContainerModule exists
    ${if firestream ? mkPythonContainerModule then ''
      echo "PASS: mkPythonContainerModule is exported"
    '' else ''
      echo "FAIL: mkPythonContainerModule is not exported"
      exit 1
    ''}

    # Check that containers namespace exists
    ${if firestream ? containers then ''
      echo "PASS: containers namespace is exported"
    '' else ''
      echo "FAIL: containers namespace is not exported"
      exit 1
    ''}

    touch $out
  '';

  # Test that a basic container module can be created
  testBasicContainerModule = let
    testModule = firestream.mkContainerModule {
      name = "test-app";
      version = "1.0.0";
      validateFn = ''
        info "Validating test-app..."
      '';
      configFn = ''
        info "Configuring test-app..."
      '';
      initFn = ''
        info "Initializing test-app..."
      '';
      runCmd = "echo 'test-app running'";
      systemDeps = [ pkgs.coreutils ];
      runtimeBinDeps = [ pkgs.curl ];
      exposedPorts = [ 8080 ];
      volumes = [ "/data" ];
    };
  in pkgs.runCommand "test-basic-container-module" {} ''
    echo "Testing basic container module creation..."

    # Check that required outputs exist
    ${if testModule ? meta then ''
      echo "PASS: meta attribute exists"
    '' else ''
      echo "FAIL: meta attribute missing"
      exit 1
    ''}

    ${if testModule ? runtimeDeps then ''
      echo "PASS: runtimeDeps attribute exists"
    '' else ''
      echo "FAIL: runtimeDeps attribute missing"
      exit 1
    ''}

    ${if testModule ? scripts then ''
      echo "PASS: scripts attribute exists"
    '' else ''
      echo "FAIL: scripts attribute missing"
      exit 1
    ''}

    ${if testModule ? runtimeEnv then ''
      echo "PASS: runtimeEnv attribute exists"
    '' else ''
      echo "FAIL: runtimeEnv attribute missing"
      exit 1
    ''}

    ${if testModule ? dockerImage then ''
      echo "PASS: dockerImage attribute exists"
    '' else ''
      echo "FAIL: dockerImage attribute missing"
      exit 1
    ''}

    ${if testModule ? devShell then ''
      echo "PASS: devShell attribute exists"
    '' else ''
      echo "FAIL: devShell attribute missing"
      exit 1
    ''}

    ${if testModule ? config then ''
      echo "PASS: config attribute exists"
    '' else ''
      echo "FAIL: config attribute missing"
      exit 1
    ''}

    # Check meta values
    ${if testModule.meta.name == "test-app" then ''
      echo "PASS: meta.name is correct"
    '' else ''
      echo "FAIL: meta.name is incorrect"
      exit 1
    ''}

    ${if testModule.meta.version == "1.0.0" then ''
      echo "PASS: meta.version is correct"
    '' else ''
      echo "FAIL: meta.version is incorrect"
      exit 1
    ''}

    touch $out
  '';

  # Test that scripts are properly generated
  testScriptGeneration = let
    testModule = firestream.mkContainerModule {
      name = "script-test";
      version = "2.0.0";
      runCmd = "echo 'running'";
    };
  in pkgs.runCommand "test-script-generation" {} ''
    echo "Testing script generation..."

    # Check that scripts.entrypoint exists and is a derivation
    ${if testModule.scripts ? entrypoint then ''
      echo "PASS: entrypoint script exists"
      # Check the entrypoint is actually a path
      if [ -d "${testModule.scripts.entrypoint}" ]; then
        echo "PASS: entrypoint is a valid derivation"
      else
        echo "FAIL: entrypoint is not a valid derivation"
        exit 1
      fi
    '' else ''
      echo "FAIL: entrypoint script missing"
      exit 1
    ''}

    ${if testModule.scripts ? envDefaults then ''
      echo "PASS: envDefaults script exists"
    '' else ''
      echo "FAIL: envDefaults script missing"
      exit 1
    ''}

    ${if testModule.scripts ? fileLoader then ''
      echo "PASS: fileLoader script exists"
    '' else ''
      echo "FAIL: fileLoader script missing"
      exit 1
    ''}

    ${if testModule.scripts ? lib then ''
      echo "PASS: lib script exists"
    '' else ''
      echo "FAIL: lib script missing"
      exit 1
    ''}

    touch $out
  '';

  # Test that the config module is included in coreLibs
  testConfigModuleInCoreLibs = pkgs.runCommand "test-config-module-in-core-libs" {} ''
    echo "Testing config module is in coreLibs..."

    ${if firestream.lib ? config then ''
      echo "PASS: config module is in lib"
    '' else ''
      echo "FAIL: config module is not in lib"
      exit 1
    ''}

    ${if firestream.lib.config ? functions then ''
      echo "PASS: config module has functions"
    '' else ''
      echo "FAIL: config module is missing functions"
      exit 1
    ''}

    touch $out
  '';

  # Test meta information
  testMetaInformation = pkgs.runCommand "test-meta-information" {} ''
    echo "Testing meta information..."

    # Check that containerFactories is in meta
    ${if lib.elem "mkContainerModule" firestream.meta.containerFactories then ''
      echo "PASS: mkContainerModule is in meta.containerFactories"
    '' else ''
      echo "FAIL: mkContainerModule is not in meta.containerFactories"
      exit 1
    ''}

    ${if lib.elem "mkPythonContainerModule" firestream.meta.containerFactories then ''
      echo "PASS: mkPythonContainerModule is in meta.containerFactories"
    '' else ''
      echo "FAIL: mkPythonContainerModule is not in meta.containerFactories"
      exit 1
    ''}

    # Check that config is in modules list
    ${if lib.elem "config" firestream.meta.modules then ''
      echo "PASS: config is in meta.modules"
    '' else ''
      echo "FAIL: config is not in meta.modules"
      exit 1
    ''}

    touch $out
  '';

in pkgs.runCommand "container-tests" {
  buildInputs = [
    testContainerFactoriesExist
    testBasicContainerModule
    testScriptGeneration
    testConfigModuleInCoreLibs
    testMetaInformation
  ];
} ''
  echo "================================================"
  echo "Container module tests completed successfully!"
  echo "================================================"
  touch $out
''
