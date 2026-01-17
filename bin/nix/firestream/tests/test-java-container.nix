# Tests for the Java container module factory
# Copyright Firestream. Apache-2.0 License.
{ pkgs, firestream }:

let
  lib = pkgs.lib;

  # Test that mkJavaContainerModule is exported
  testJavaFactoryExists = pkgs.runCommand "test-java-factory-exists" {} ''
    echo "Testing mkJavaContainerModule is exported..."

    ${if firestream ? mkJavaContainerModule then ''
      echo "PASS: mkJavaContainerModule is exported"
    '' else ''
      echo "FAIL: mkJavaContainerModule is not exported"
      exit 1
    ''}

    ${if firestream.containers ? mkJavaContainerModule then ''
      echo "PASS: mkJavaContainerModule is in containers namespace"
    '' else ''
      echo "FAIL: mkJavaContainerModule is not in containers namespace"
      exit 1
    ''}

    # Check that mkJavaContainerModule is in meta.containerFactories
    ${if lib.elem "mkJavaContainerModule" firestream.meta.containerFactories then ''
      echo "PASS: mkJavaContainerModule is in meta.containerFactories"
    '' else ''
      echo "FAIL: mkJavaContainerModule is not in meta.containerFactories"
      exit 1
    ''}

    touch $out
  '';

  # Test that a basic Java container module can be created
  testBasicJavaContainerModule = let
    testModule = firestream.mkJavaContainerModule {
      name = "test-java-app";
      version = "1.0.0";
      heapOpts = "-Xmx512m -Xms256m";
      jarDirs = [ "/opt/app/lib" ];
      validateFn = ''
        info "Validating test-java-app..."
      '';
      runCmd = "java -jar app.jar";
      exposedPorts = [ 8080 ];
    };
  in pkgs.runCommand "test-basic-java-container-module" {} ''
    echo "Testing basic Java container module creation..."

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

    # Check Java-specific attributes
    ${if testModule ? jdk then ''
      echo "PASS: jdk attribute exists"
    '' else ''
      echo "FAIL: jdk attribute missing"
      exit 1
    ''}

    ${if testModule ? resolvedJavaHome then ''
      echo "PASS: resolvedJavaHome attribute exists"
    '' else ''
      echo "FAIL: resolvedJavaHome attribute missing"
      exit 1
    ''}

    ${if testModule ? javaOpts then ''
      echo "PASS: javaOpts attribute exists"
    '' else ''
      echo "FAIL: javaOpts attribute missing"
      exit 1
    ''}

    # Check meta values
    ${if testModule.meta.name == "test-java-app" then ''
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

    # Check Java-specific meta
    ${if testModule.meta ? jdkVersion then ''
      echo "PASS: meta.jdkVersion exists"
    '' else ''
      echo "FAIL: meta.jdkVersion missing"
      exit 1
    ''}

    ${if testModule.meta ? heapConfiguration then ''
      echo "PASS: meta.heapConfiguration exists"
    '' else ''
      echo "FAIL: meta.heapConfiguration missing"
      exit 1
    ''}

    ${if testModule.meta ? nssWrapperEnabled then ''
      echo "PASS: meta.nssWrapperEnabled exists"
    '' else ''
      echo "FAIL: meta.nssWrapperEnabled missing"
      exit 1
    ''}

    touch $out
  '';

  # Test JVM options building
  testJvmOptionsBuilding = let
    testModule = firestream.mkJavaContainerModule {
      name = "jvm-test";
      heapOpts = "-Xmx2g -Xms1g";
      gcOpts = "-XX:+UseZGC";
      jmxOpts = "-Dcom.sun.management.jmxremote";
      extraJavaOpts = "-Dfoo=bar";
      runCmd = "java Main";
    };
  in pkgs.runCommand "test-jvm-options-building" {} ''
    echo "Testing JVM options building..."

    # Check that javaOpts contains all options
    JAVA_OPTS="${testModule.javaOpts}"

    if [[ "$JAVA_OPTS" == *"-Xmx2g"* ]]; then
      echo "PASS: heapOpts included in javaOpts"
    else
      echo "FAIL: heapOpts not included in javaOpts"
      exit 1
    fi

    if [[ "$JAVA_OPTS" == *"-XX:+UseZGC"* ]]; then
      echo "PASS: gcOpts included in javaOpts"
    else
      echo "FAIL: gcOpts not included in javaOpts"
      exit 1
    fi

    if [[ "$JAVA_OPTS" == *"-Dcom.sun.management.jmxremote"* ]]; then
      echo "PASS: jmxOpts included in javaOpts"
    else
      echo "FAIL: jmxOpts not included in javaOpts"
      exit 1
    fi

    if [[ "$JAVA_OPTS" == *"-Dfoo=bar"* ]]; then
      echo "PASS: extraJavaOpts included in javaOpts"
    else
      echo "FAIL: extraJavaOpts not included in javaOpts"
      exit 1
    fi

    # Check config stores individual options
    ${if testModule.config.heapOpts == "-Xmx2g -Xms1g" then ''
      echo "PASS: config.heapOpts is correct"
    '' else ''
      echo "FAIL: config.heapOpts is incorrect"
      exit 1
    ''}

    ${if testModule.config.gcOpts == "-XX:+UseZGC" then ''
      echo "PASS: config.gcOpts is correct"
    '' else ''
      echo "FAIL: config.gcOpts is incorrect"
      exit 1
    ''}

    touch $out
  '';

  # Test classpath building
  testClasspathBuilding = let
    testModule = firestream.mkJavaContainerModule {
      name = "classpath-test";
      classPath = [ "/opt/app/classes" "/opt/app/config" ];
      jarDirs = [ "/opt/app/lib" "/opt/app/ext" ];
      runCmd = "java Main";
    };
  in pkgs.runCommand "test-classpath-building" {} ''
    echo "Testing classpath building..."

    CLASSPATH="${testModule.config.resolvedClassPath}"

    if [[ "$CLASSPATH" == *"/opt/app/classes"* ]]; then
      echo "PASS: explicit classPath included"
    else
      echo "FAIL: explicit classPath not included"
      exit 1
    fi

    if [[ "$CLASSPATH" == *"/opt/app/lib/*"* ]]; then
      echo "PASS: jarDirs expanded with wildcard"
    else
      echo "FAIL: jarDirs not expanded with wildcard"
      exit 1
    fi

    if [[ "$CLASSPATH" == *"/opt/app/ext/*"* ]]; then
      echo "PASS: multiple jarDirs handled"
    else
      echo "FAIL: multiple jarDirs not handled"
      exit 1
    fi

    touch $out
  '';

  # Test NSS wrapper configuration
  testNssWrapperConfig = let
    testModuleEnabled = firestream.mkJavaContainerModule {
      name = "nss-enabled";
      enableNssWrapper = true;
      runCmd = "java Main";
    };
    testModuleDisabled = firestream.mkJavaContainerModule {
      name = "nss-disabled";
      enableNssWrapper = false;
      runCmd = "java Main";
    };
  in pkgs.runCommand "test-nss-wrapper-config" {} ''
    echo "Testing NSS wrapper configuration..."

    ${if testModuleEnabled.config.enableNssWrapper == true then ''
      echo "PASS: enableNssWrapper=true is stored"
    '' else ''
      echo "FAIL: enableNssWrapper=true not stored correctly"
      exit 1
    ''}

    ${if testModuleEnabled.meta.nssWrapperEnabled == true then ''
      echo "PASS: meta.nssWrapperEnabled=true"
    '' else ''
      echo "FAIL: meta.nssWrapperEnabled not true"
      exit 1
    ''}

    ${if testModuleDisabled.config.enableNssWrapper == false then ''
      echo "PASS: enableNssWrapper=false is stored"
    '' else ''
      echo "FAIL: enableNssWrapper=false not stored correctly"
      exit 1
    ''}

    ${if testModuleDisabled.meta.nssWrapperEnabled == false then ''
      echo "PASS: meta.nssWrapperEnabled=false"
    '' else ''
      echo "FAIL: meta.nssWrapperEnabled not false"
      exit 1
    ''}

    touch $out
  '';

  # Test custom JDK configuration
  testCustomJdkConfig = let
    customJdk = pkgs.temurin-bin-21;
    testModule = firestream.mkJavaContainerModule {
      name = "custom-jdk";
      jdk = customJdk;
      runCmd = "java Main";
    };
  in pkgs.runCommand "test-custom-jdk-config" {} ''
    echo "Testing custom JDK configuration..."

    # Check that the custom JDK is used
    ${if testModule.jdk == customJdk then ''
      echo "PASS: custom JDK is used"
    '' else ''
      echo "FAIL: custom JDK not used"
      exit 1
    ''}

    # Check that resolvedJavaHome points to custom JDK
    if [[ "${testModule.resolvedJavaHome}" == "${customJdk}"* ]]; then
      echo "PASS: resolvedJavaHome points to custom JDK"
    else
      echo "FAIL: resolvedJavaHome does not point to custom JDK"
      exit 1
    fi

    touch $out
  '';

in pkgs.runCommand "java-container-tests" {
  buildInputs = [
    testJavaFactoryExists
    testBasicJavaContainerModule
    testJvmOptionsBuilding
    testClasspathBuilding
    testNssWrapperConfig
    testCustomJdkConfig
  ];
} ''
  echo "================================================"
  echo "Java container module tests completed successfully!"
  echo "================================================"
  touch $out
''
