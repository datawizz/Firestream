# Ruby module for iOS development tooling
# Provides Ruby with xcodeproj gem for XcodeGen project generation
{ pkgs, system ? null, fenixPackages ? null, firestreamLib ? null, ... }:

let
  # Ruby with bundled gems for iOS development
  rubyWithPackages = pkgs.ruby_3_3.withPackages (ps: with ps; [
    # XcodeGen project file generation
    xcodeproj

    # Additional useful gems (bundler is included with Ruby 3.3)
    rake
  ]);

in
{
  # Packages to include in the environment
  packages = [
    rubyWithPackages
  ];

  # Shell hook for Ruby configuration
  shellHook = ''
    # Set GEM_HOME to local directory to avoid permission issues
    export GEM_HOME="$PWD/.gem"
    export GEM_PATH="$GEM_HOME:${rubyWithPackages}/lib/ruby/gems/3.3.0"
    export PATH="$GEM_HOME/bin:$PATH"
  '';

  # Apps for testing
  apps = {
    test-ruby = {
      type = "app";
      program = pkgs.writeShellScript "test-ruby" ''
        echo "Testing Ruby environment..."

        # Check Ruby version
        echo -n "Ruby version: "
        ${rubyWithPackages}/bin/ruby --version

        # Check xcodeproj gem
        echo -n "xcodeproj gem: "
        ${rubyWithPackages}/bin/ruby -e "require 'xcodeproj'; puts Xcodeproj::VERSION" 2>/dev/null || echo "not found"

        # Check bundler
        echo -n "Bundler version: "
        ${rubyWithPackages}/bin/bundler --version 2>/dev/null || echo "not found"

        echo "Ruby environment test complete!"
      '';
    };
  };
}
